//! EnvMorph: registro de procedencia por etapas y localización de la primera
//! divergencia en pipelines Unix.

mod compare;
mod environment;
mod graph;
mod hash;
mod json;
mod model;
mod replay;
mod report;
mod runner;
mod store;

use std::process::exit;

const USAGE: &str = "envmorph 0.1.0

USO
  envmorph begin  [--map PREFIJO=NOMBRE]... [--env VAR]... inicia una ejecución e imprime su id
  envmorph stage  --name N [--input P]... [--code P]... [--output P]...
                    [--stdout ARCHIVO] [--stderr ARCHIVO] [--cwd DIR]
                    [--no-snapshot] [--quiet] -- CMD [ARGS...]
  envmorph end                                            cierra la ejecución activa
  envmorph list                                           lista las ejecuciones registradas
  envmorph inspect RUN                                    resume una ejecución
  envmorph diff    A B [--json]                           localiza la primera divergencia
  envmorph isolate A B                                    repite etapas cuyo ejecutable difiere
  envmorph graph   RUN [--diff OTHER]                     genera el grafo de procedencia (DOT)
  envmorph verify  RUN                                    recalcula hashes de artefactos en disco
  envmorph export  RUN                                    exporta el manifest completo como JSON

RUN es un id (run-003), un número (3) o 'latest'.
El store se ubica en $ENVMORPH_HOME (por defecto ./.envmorph).
La ejecución activa es $ENVMORPH_RUN o la escrita por `begin`.
El código de salida de `stage` es el de la orden envuelta.
El código de salida de `diff` es 0 si las salidas son idénticas y 1 en otro caso.";

fn fail(msg: &str) -> ! {
    eprintln!("envmorph: {}", msg);
    exit(2);
}

fn take_value(args: &[String], i: &mut usize, flag: &str) -> String {
    *i += 1;
    match args.get(*i) {
        Some(v) => v.clone(),
        None => fail(&format!("{} requiere un valor", flag)),
    }
}

fn cmd_begin(args: &[String]) {
    let mut maps = Vec::new();
    let mut extra = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--map" => {
                let v = take_value(args, &mut i, "--map");
                match v.split_once('=') {
                    Some((p, n)) if !p.is_empty() && !n.is_empty() => maps.push((
                        store::absolutize(std::path::Path::new(p))
                            .to_string_lossy()
                            .to_string(),
                        n.to_string(),
                    )),
                    _ => fail("--map espera PREFIJO=NOMBRE"),
                }
            }
            "--env" => extra.push(take_value(args, &mut i, "--env")),
            other => fail(&format!("opción desconocida '{}' para begin", other)),
        }
        i += 1;
    }
    match store::create_run(maps, environment::capture(&extra)) {
        Ok(m) => println!("{}", m.run_id),
        Err(e) => fail(&e),
    }
}

fn cmd_stage(args: &[String]) {
    let mut a = runner::StageArgs::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--" => {
                a.command = args[i + 1..].to_vec();
                break;
            }
            "--name" => a.name = take_value(args, &mut i, "--name"),
            "--input" => a.inputs.push(take_value(args, &mut i, "--input")),
            "--code" => a.code.push(take_value(args, &mut i, "--code")),
            "--output" => a.outputs.push(take_value(args, &mut i, "--output")),
            "--stdout" => a.stdout = Some(take_value(args, &mut i, "--stdout")),
            "--stderr" => a.stderr = Some(take_value(args, &mut i, "--stderr")),
            "--cwd" => a.cwd = Some(take_value(args, &mut i, "--cwd")),
            "--no-snapshot" => a.no_snapshot = true,
            "--quiet" => a.quiet = true,
            other => fail(&format!("opción desconocida '{}' para stage", other)),
        }
        i += 1;
    }
    match runner::run(a) {
        Ok(code) => exit(code),
        Err(e) => fail(&e),
    }
}

fn load(arg: &str) -> model::Manifest {
    let id = store::resolve_run(arg).unwrap_or_else(|e| fail(&e));
    store::load_run(&id).unwrap_or_else(|e| fail(&e))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        println!("{}", USAGE);
        exit(2);
    };
    let rest = &args[1..];
    match cmd.as_str() {
        "begin" => cmd_begin(rest),
        "stage" => cmd_stage(rest),
        "end" => {
            let id = store::current_run_id().unwrap_or_else(|e| fail(&e));
            let mut m = store::load_run(&id).unwrap_or_else(|e| fail(&e));
            m.status = "closed".into();
            m.ended_ms = store::now_ms();
            store::save_header(&m).unwrap_or_else(|e| fail(&e));
            println!("{} cerrada con {} etapa(s)", id, m.stages.len());
        }
        "list" => {
            for id in store::list_runs() {
                match store::load_run(&id) {
                    Ok(m) => println!(
                        "{}  {}  {:<6}  etapas={}  git={}",
                        id,
                        report::iso(m.started_ms),
                        m.status,
                        m.stages.len(),
                        model::short(&m.env.git_commit)
                    ),
                    Err(e) => println!("{}  ilegible: {}", id, e),
                }
            }
        }
        "inspect" => {
            let r = rest
                .first()
                .unwrap_or_else(|| fail("inspect requiere una ejecución"));
            print!("{}", report::inspect(&load(r)));
        }
        "diff" => {
            let mut as_json = false;
            let mut runs = Vec::new();
            for a in rest {
                if a == "--json" {
                    as_json = true;
                } else {
                    runs.push(a.clone());
                }
            }
            if runs.len() != 2 {
                fail("diff requiere dos ejecuciones");
            }
            let (a, b) = (load(&runs[0]), load(&runs[1]));
            let d = compare::compare(&a, &b);
            if as_json {
                print!("{}", report::diff_json(&a, &b, &d).pretty());
            } else {
                print!("{}", report::render_diff(&a, &b, &d));
            }
            exit(if d.identical() { 0 } else { 1 });
        }
        "isolate" => {
            if rest.len() != 2 {
                fail("isolate requiere dos ejecuciones");
            }
            let (a, b) = (load(&rest[0]), load(&rest[1]));
            let iso = replay::isolate(&a, &b);
            print!("{}", replay::render(&a, &b, &iso));
            exit(
                if iso.iter().any(|i| i.verdict == replay::Verdict::Sensitive) {
                    1
                } else {
                    0
                },
            );
        }
        "graph" => {
            let r = rest
                .first()
                .unwrap_or_else(|| fail("graph requiere una ejecución"));
            let m = load(r);
            let other = rest
                .iter()
                .position(|x| x == "--diff")
                .and_then(|p| rest.get(p + 1));
            let d = other.map(|o| compare::compare(&m, &load(o)));
            print!("{}", graph::dot(&m, d.as_ref()));
        }
        "verify" => {
            let r = rest
                .first()
                .unwrap_or_else(|| fail("verify requiere una ejecución"));
            let (text, ok) = report::verify(&load(r));
            print!("{}", text);
            exit(if ok { 0 } else { 1 });
        }
        "export" => {
            let r = rest
                .first()
                .unwrap_or_else(|| fail("export requiere una ejecución"));
            print!("{}", load(r).full_json().pretty());
        }
        "-h" | "--help" | "help" => println!("{}", USAGE),
        "--version" | "version" => println!("envmorph {}", env!("CARGO_PKG_VERSION")),
        other => {
            eprintln!("envmorph: comando desconocido '{}'\n\n{}", other, USAGE);
            exit(2);
        }
    }
}

mod experiment;
