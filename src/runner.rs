//! `envmorph stage`: ejecuta una orden, calcula hashes de sus entradas y salidas
//! declaradas y agrega un registro de etapa a la ejecución activa.

use crate::model::Stage;
use crate::{environment, store};
use std::fs::File;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Default)]
pub struct StageArgs {
    pub name: String,
    pub inputs: Vec<String>,
    pub code: Vec<String>,
    pub outputs: Vec<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub cwd: Option<String>,
    pub no_snapshot: bool,
    pub quiet: bool,
    pub command: Vec<String>,
}

const AWK_FAMILY: &[&str] = &["awk", "gawk", "mawk", "nawk"];
const SHELL_FAMILY: &[&str] = &["bash", "sh", "dash", "zsh"];

fn basename(s: &str) -> &str {
    s.rsplit('/').next().unwrap_or(s)
}

/// Los scripts ejecutados por la orden son entradas de código: `awk -f FILE`, `bash FILE`.
fn detect_code(cmd: &[String], base: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let Some(first) = cmd.first() else {
        return found;
    };
    let b = basename(first);
    if AWK_FAMILY.contains(&b) {
        let mut i = 1;
        while i < cmd.len() {
            if cmd[i] == "-f" && i + 1 < cmd.len() {
                found.push(cmd[i + 1].clone());
                i += 1;
            } else if let Some(f) = cmd[i].strip_prefix("--file=") {
                found.push(f.to_string());
            }
            i += 1;
        }
    } else if SHELL_FAMILY.contains(&b) {
        if let Some(script) = cmd.iter().skip(1).find(|a| !a.starts_with('-')) {
            if resolve(base, script).is_file() {
                found.push(script.clone());
            }
        }
    }
    found
}

fn resolve(base: &Path, p: &str) -> PathBuf {
    let pb = Path::new(p);
    if pb.is_absolute() {
        store::normalize(pb)
    } else {
        store::normalize(&base.join(pb))
    }
}

pub fn run(a: StageArgs) -> Result<i32, String> {
    if a.name.is_empty() {
        return Err("--name es obligatorio".into());
    }
    if a.command.is_empty() {
        return Err("falta la orden, usa `-- cmd args...`".into());
    }
    let run_id = store::current_run_id()?;
    let header = store::load_header(&run_id)?;
    if header.status != "open" {
        return Err(format!("{} está cerrada", run_id));
    }
    let run_dir = store::run_dir(&run_id);
    let mapper = store::PathMapper::new(&header.maps, &store::cwd());
    let base = match &a.cwd {
        Some(d) => resolve(&store::cwd(), d),
        None => store::cwd(),
    };

    // Entradas declaradas más entradas de código detectadas, sin duplicados por ruta real.
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut inputs = Vec::new();
    let no_snapshot = a.no_snapshot;
    let mut add_input = |p: &str, role: &str, inputs: &mut Vec<crate::model::Artifact>| {
        let real = resolve(&base, p);
        if seen.contains(&real) {
            return;
        }
        seen.push(real.clone());
        let art = store::hash_artifact(&real, mapper.logical(&real), role);
        if !no_snapshot {
            store::snapshot(&real, &art);
        }
        inputs.push(art);
    };
    for p in &a.inputs {
        add_input(p, "data", &mut inputs);
    }
    for p in a.code.iter().cloned().chain(detect_code(&a.command, &base)) {
        add_input(&p, "code", &mut inputs);
    }

    let started_ms = store::now_ms();
    let t0 = Instant::now();
    let mut cmd = Command::new(&a.command[0]);
    cmd.args(&a.command[1..]).current_dir(&base);
    // Semántica de `> f 2>&1`: si ambos streams apuntan al mismo archivo comparten un descriptor.
    let same_stream_file = match (&a.stdout, &a.stderr) {
        (Some(x), Some(y)) => resolve(&base, x) == resolve(&base, y),
        _ => false,
    };
    if let Some(p) = &a.stdout {
        let f = File::create(resolve(&base, p))
            .map_err(|e| format!("no se puede crear el archivo stdout {}: {}", p, e))?;
        if same_stream_file {
            cmd.stderr(Stdio::from(f.try_clone().map_err(|e| e.to_string())?));
        }
        cmd.stdout(Stdio::from(f));
    }
    if let (Some(p), false) = (&a.stderr, same_stream_file) {
        let f = File::create(resolve(&base, p))
            .map_err(|e| format!("no se puede crear el archivo stderr {}: {}", p, e))?;
        cmd.stderr(Stdio::from(f));
    }
    let exit_code = match cmd.status() {
        Ok(st) => st.code().unwrap_or_else(|| 128 + st.signal().unwrap_or(0)),
        Err(e) => {
            eprintln!("envmorph: no se puede ejecutar '{}': {}", a.command[0], e);
            127
        }
    };
    let duration_ms = t0.elapsed().as_millis() as u64;

    // Salidas declaradas más archivos usados para redirigir stdout y stderr.
    let mut out_paths: Vec<String> = a.outputs.clone();
    for p in [&a.stdout, &a.stderr].into_iter().flatten() {
        if !out_paths
            .iter()
            .any(|q| resolve(&base, q) == resolve(&base, p))
        {
            out_paths.push(p.clone());
        }
    }
    let mut outputs = Vec::new();
    for p in &out_paths {
        let real = resolve(&base, p);
        let art = store::hash_artifact(&real, mapper.logical(&real), "data");
        if !a.no_snapshot {
            store::snapshot(&real, &art);
        }
        outputs.push(art);
    }

    let tool = environment::tool_info(&a.command[0], &run_dir);
    let stage = Stage {
        index: 0,
        name: a.name.clone(),
        command: a.command.iter().map(|s| mapper.rewrite(s)).collect(),
        cwd: mapper.logical(&base),
        tool,
        inputs,
        outputs,
        exit_code,
        duration_ms,
        started_ms,
        stdout: a
            .stdout
            .as_ref()
            .map(|p| mapper.logical(&resolve(&base, p))),
        stderr: a
            .stderr
            .as_ref()
            .map(|p| mapper.logical(&resolve(&base, p))),
    };
    let (n_in, n_out) = (stage.inputs.len(), stage.outputs.len());
    let written = store::write_stage(&run_id, stage)?;
    if !a.quiet {
        eprintln!(
            "[envmorph] {} {:03} {} salida={} {}ms ent={} sal={}",
            run_id, written.index, a.name, exit_code, duration_ms, n_in, n_out
        );
    }
    Ok(exit_code)
}
