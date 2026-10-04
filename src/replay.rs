//! Aislamiento de etapas mediante replay.
//!
//! Una etapa aguas abajo cuyas entradas y ejecutable cambiaron queda confounded:
//! el diff no puede determinar si su cambio propio habría alterado la salida con
//! entradas idénticas. `isolate` responde repitiendo la etapa registrada con sus
//! snapshots de entrada, una vez con su herramienta y otra con la herramienta de
//! la otra ejecución, y compara las salidas con las registradas.
//!
//! Las etiquetas de veredicto se conservan como identificadores técnicos estables:
//! sensitive, insensitive, not-reproducible y not-replayable.

use crate::model::{Manifest, Stage};
use crate::{hash, store};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

const ENV_NAMES: &[&str] = &[
    "LANG",
    "LC_ALL",
    "LC_COLLATE",
    "LC_CTYPE",
    "LC_NUMERIC",
    "LC_TIME",
    "TZ",
    "POSIXLY_CORRECT",
    "SOURCE_DATE_EPOCH",
];

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Sensitive,
    Insensitive,
    NotReproducible,
    NotReplayable(String),
}

impl Verdict {
    pub fn label(&self) -> String {
        match self {
            Verdict::Sensitive => "sensitive".into(),
            Verdict::Insensitive => "insensitive".into(),
            Verdict::NotReproducible => "not-reproducible".into(),
            Verdict::NotReplayable(why) => format!("not-replayable ({})", why),
        }
    }
}

fn to_sandbox(sandbox: &Path, logical: &str) -> Result<PathBuf, Verdict> {
    if logical == "." {
        return Ok(sandbox.join("_cwd"));
    }
    if logical.is_empty() {
        return Err(Verdict::NotReplayable("ruta lógica vacía".into()));
    }

    let path = Path::new(logical);
    if path.is_absolute() {
        return Err(Verdict::NotReplayable(format!(
            "ruta absoluta no mapeada: {}",
            logical
        )));
    }

    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(Verdict::NotReplayable(format!(
                    "ruta con '..' fuera del sandbox: {}",
                    logical
                )));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(Verdict::NotReplayable(format!(
                    "ruta no confinable al sandbox: {}",
                    logical
                )));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err(Verdict::NotReplayable("ruta lógica vacía".into()));
    }
    Ok(sandbox.join(relative))
}

fn rewrite_arg(sandbox: &Path, maps: &[(String, String)], arg: &str) -> String {
    let mut cur = arg.to_string();
    let mut names: Vec<&String> = maps.iter().map(|(_, n)| n).collect();
    names.sort_by(|a, b| b.len().cmp(&a.len()));
    for n in names {
        let target = sandbox.join(n).to_string_lossy().to_string();
        let mut out = String::new();
        let mut i = 0;
        while let Some(pos) = cur[i..].find(n.as_str()) {
            let start = i + pos;
            let end = start + n.len();
            let boundary = match cur[end..].chars().next() {
                None => true,
                Some(c) => {
                    c == '/' || !(c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
                }
            };
            out.push_str(&cur[i..start]);
            out.push_str(if boundary { &target } else { n });
            i = end;
        }
        out.push_str(&cur[i..]);
        cur = out;
    }
    cur
}

/// Ejecuta `st` en un sandbox nuevo usando `tool_path` como ejecutable. Devuelve
/// las salidas registradas que difieren del replay, o una lista vacía si reproduce.
fn replay_stage(m: &Manifest, st: &Stage, tool_path: &str) -> Result<Vec<String>, Verdict> {
    let sandbox = store::home().join(format!("replay-{}-{}", std::process::id(), store::now_ms()));
    let _ = fs::remove_dir_all(&sandbox);
    fs::create_dir_all(&sandbox).map_err(|e| Verdict::NotReplayable(e.to_string()))?;
    let result = (|| {
        for a in &st.inputs {
            match a.kind.as_str() {
                "file" => {
                    let sha = a
                        .sha256
                        .as_ref()
                        .ok_or_else(|| Verdict::NotReplayable("sin digest".into()))?;
                    let data = store::read_object(sha).ok_or_else(|| {
                        Verdict::NotReplayable(format!("sin snapshot para {}", a.path))
                    })?;
                    let dest = to_sandbox(&sandbox, &a.path)?;
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| Verdict::NotReplayable(e.to_string()))?;
                    }
                    fs::write(&dest, data).map_err(|e| Verdict::NotReplayable(e.to_string()))?;
                }
                "dir" => {
                    return Err(Verdict::NotReplayable(format!(
                        "entrada de directorio {}",
                        a.path
                    )))
                }
                _ => {}
            }
        }
        let cwd = to_sandbox(&sandbox, &st.cwd)?;
        fs::create_dir_all(&cwd).map_err(|e| Verdict::NotReplayable(e.to_string()))?;
        for o in &st.outputs {
            if let Some(parent) = to_sandbox(&sandbox, &o.path)?.parent() {
                let _ = fs::create_dir_all(parent);
            }
        }
        let mut args: Vec<String> = st
            .command
            .iter()
            .map(|c| rewrite_arg(&sandbox, &m.maps, c))
            .collect();
        if args.is_empty() {
            return Err(Verdict::NotReplayable("orden vacía".into()));
        }
        args.remove(0);
        let mut cmd = Command::new(tool_path);
        cmd.args(&args).current_dir(&cwd).stdin(Stdio::null());
        for n in ENV_NAMES {
            match m.env.vars.iter().find(|(k, _)| k == n) {
                Some((_, v)) => cmd.env(n, v),
                None => cmd.env_remove(n),
            };
        }
        let open = |logical: &str| -> Result<fs::File, Verdict> {
            let p = to_sandbox(&sandbox, logical)?;
            if let Some(parent) = p.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::File::create(&p).map_err(|e| Verdict::NotReplayable(e.to_string()))
        };
        let same = st.stdout.is_some() && st.stdout == st.stderr;
        if let Some(p) = &st.stdout {
            let f = open(p)?;
            if same {
                cmd.stderr(Stdio::from(
                    f.try_clone()
                        .map_err(|e| Verdict::NotReplayable(e.to_string()))?,
                ));
            }
            cmd.stdout(Stdio::from(f));
        } else {
            cmd.stdout(Stdio::null());
        }
        match (&st.stderr, same) {
            (Some(p), false) => {
                cmd.stderr(Stdio::from(open(p)?));
            }
            (None, _) => {
                cmd.stderr(Stdio::null());
            }
            _ => {}
        }
        let status = cmd.status().map_err(|e| {
            Verdict::NotReplayable(format!("no se puede ejecutar {}: {}", tool_path, e))
        })?;
        let code = status.code().unwrap_or(-1);
        let mut diffs = Vec::new();
        for o in &st.outputs {
            let now = store::hash_artifact(&to_sandbox(&sandbox, &o.path)?, o.path.clone(), "data");
            if now.sha256 != o.sha256 {
                diffs.push(o.path.clone());
            }
        }
        if code != st.exit_code && diffs.is_empty() {
            diffs.push("<código de salida>".to_string());
        }
        Ok(diffs)
    })();
    let _ = fs::remove_dir_all(&sandbox);
    result
}

#[derive(Debug, Clone)]
pub struct Isolation {
    pub position: usize,
    pub name: String,
    pub verdict: Verdict,
    pub detail: String,
}

/// Para cada etapa cuyo ejecutable difiere entre `a` y `b`, ejecuta replay en
/// ambas direcciones con entradas idénticas.
pub fn isolate(a: &Manifest, b: &Manifest) -> Vec<Isolation> {
    let mut out = Vec::new();
    let n = a.stages.len().min(b.stages.len());
    for i in 0..n {
        let (sa, sb) = (&a.stages[i], &b.stages[i]);
        let (Some(ta), Some(tb)) = (&sa.tool, &sb.tool) else {
            continue;
        };
        if ta.sha256 == tb.sha256 {
            continue;
        }
        let verdict_detail = (|| -> Result<(Verdict, String), Verdict> {
            let own_a = replay_stage(a, sa, &ta.path)?;
            if !own_a.is_empty() {
                return Ok((
                    Verdict::NotReproducible,
                    format!("la ejecución A no reproduce {}", own_a.join(", ")),
                ));
            }
            let own_b = replay_stage(b, sb, &tb.path)?;
            if !own_b.is_empty() {
                return Ok((
                    Verdict::NotReproducible,
                    format!("la ejecución B no reproduce {}", own_b.join(", ")),
                ));
            }
            let swap_a = replay_stage(a, sa, &tb.path)?;
            let swap_b = replay_stage(b, sb, &ta.path)?;
            if swap_a.is_empty() && swap_b.is_empty() {
                Ok((Verdict::Insensitive, "misma salida con cualquiera de los ejecutables sobre las entradas de ambas ejecuciones".into()))
            } else {
                let mut d = Vec::new();
                if !swap_a.is_empty() {
                    d.push(format!(
                        "entradas A con herramienta B cambian {}",
                        swap_a.join(", ")
                    ));
                }
                if !swap_b.is_empty() {
                    d.push(format!(
                        "entradas B con herramienta A cambian {}",
                        swap_b.join(", ")
                    ));
                }
                Ok((Verdict::Sensitive, d.join("; ")))
            }
        })();
        let (verdict, detail) = match verdict_detail {
            Ok(x) => x,
            Err(v) => (v, String::new()),
        };
        out.push(Isolation {
            position: i + 1,
            name: sa.name.clone(),
            verdict,
            detail,
        });
    }
    out
}

pub fn render(a: &Manifest, b: &Manifest, iso: &[Isolation]) -> String {
    let mut o = String::new();
    o.push_str(&format!("AISLAMIENTO  A={}  B={}  (etapas con ejecutable diferente, repetidas con entradas idénticas)\n\n", a.run_id, b.run_id));
    for i in iso {
        o.push_str(&format!(
            "  {:03} {:<24} {}\n",
            i.position,
            i.name,
            i.verdict.label()
        ));
        if !i.detail.is_empty() {
            o.push_str(&format!("      {}\n", i.detail));
        }
    }
    let sens: Vec<&Isolation> = iso
        .iter()
        .filter(|i| i.verdict == Verdict::Sensitive)
        .collect();
    let unknown = iso
        .iter()
        .filter(|i| {
            matches!(
                i.verdict,
                Verdict::NotReproducible | Verdict::NotReplayable(_)
            )
        })
        .count();
    o.push('\n');
    match sens.first() {
        Some(f) => {
            o.push_str(&format!(
                "PRIMERA CAUSA CONFIRMADA  etapa {:03} {}\n",
                f.position, f.name
            ));
            o.push_str(&format!("  {} de {} etapa(s) repetidas son intrínsecamente sensibles al cambio de ejecutable\n", sens.len(), iso.len()));
        }
        None => o.push_str("Ninguna etapa repetida es sensible al cambio de ejecutable.\n"),
    }
    if unknown > 0 {
        o.push_str(&format!("  {} etapa(s) no pudieron aislarse\n", unknown));
    }
    o
}

#[allow(dead_code)]
fn _unused(_: &str) -> String {
    hash::hash_bytes(b"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_replaces_logical_prefixes() {
        let maps = vec![
            ("/a/out".to_string(), "@out".to_string()),
            ("/a/src".to_string(), "@src".to_string()),
        ];
        let sb = Path::new("/sb");
        assert_eq!(rewrite_arg(sb, &maps, "@out/x.tsv"), "/sb/@out/x.tsv");
        assert_eq!(
            rewrite_arg(sb, &maps, "-v f=@src/s.awk"),
            "-v f=/sb/@src/s.awk"
        );
        assert_eq!(rewrite_arg(sb, &maps, "a@outer"), "a@outer");
        assert_eq!(rewrite_arg(sb, &maps, "plain"), "plain");
    }

    #[test]
    fn sandbox_mapping() {
        let sb = Path::new("/sb");
        assert_eq!(to_sandbox(sb, ".").unwrap(), PathBuf::from("/sb/_cwd"));
        assert_eq!(
            to_sandbox(sb, "@out/a").unwrap(),
            PathBuf::from("/sb/@out/a")
        );
        assert_eq!(to_sandbox(sb, "a/./b").unwrap(), PathBuf::from("/sb/a/b"));
    }

    #[test]
    fn sandbox_rejects_absolute_paths() {
        let sb = Path::new("/sb");
        assert!(matches!(
            to_sandbox(sb, "/dev/null"),
            Err(Verdict::NotReplayable(_))
        ));
    }

    #[test]
    fn sandbox_rejects_parent_traversal() {
        let sb = Path::new("/sb");
        assert!(matches!(
            to_sandbox(sb, "../outside"),
            Err(Verdict::NotReplayable(_))
        ));
        assert!(matches!(
            to_sandbox(sb, "@out/../../outside"),
            Err(Verdict::NotReplayable(_))
        ));
    }
}
