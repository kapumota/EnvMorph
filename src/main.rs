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
  envmorph plan    EXPERIMENT.toml                        genera un plan ambiental determinista
  envmorph capabilities EXPERIMENT.toml                  inspecciona capacidades ambientales
  envmorph explore EXPERIMENT.toml [--output DIR] [--workdir DIR] -- CMD [ARGS...]
  envmorph compare-artifacts --oracle byte|text|json|csv --left FILE --right FILE [--format human|json] [--output FILE]
  envmorph analyze-propagation --trace TRACE.tsv [--left-root DIR] [--right-root DIR] [--format human|json] [--output FILE]
  envmorph minimize-environment CAUSAL.toml --trace TRACE.tsv --workdir DIR --output DIR [--format human|json] -- CMD [ARGS...]
  envmorph derive-contract CAUSAL.toml --trace TRACE.tsv --workdir DIR --evidence DIR --output CONTRACT.toml [--format human|toml] -- CMD [ARGS...]
                                                          deriva un contrato ambiental desde evidencia F5
  envmorph check-contract CONTRACT.toml --environment ENVIRONMENT.toml [--format human|json] [--output FILE]
                                                          evalúa una configuración contra evidencia contractual
                                                          minimiza factores ambientales por suficiencia observable
                                                          analiza propagación y absorción
                                                          compara artefactos sin decidir durante la ejecución
                                                          ejecuta variantes y crea un bundle
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

fn cmd_derive_contract(args: &[String]) {
    if args.is_empty() {
        fail("derive-contract requiere un archivo TOML causal");
    }

    let spec_path = std::path::PathBuf::from(&args[0]);
    let mut trace_path: Option<std::path::PathBuf> = None;
    let mut evidence_dir: Option<std::path::PathBuf> = None;
    let mut output_path: Option<std::path::PathBuf> = None;
    let mut workdir = std::env::current_dir().unwrap_or_else(|error| {
        fail(&format!(
            "no se pudo obtener el directorio actual: {}",
            error
        ))
    });
    let mut format = "human".to_string();
    let mut command = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--" => {
                command = args[i + 1..].to_vec();
                break;
            }
            "--trace" => {
                trace_path = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--trace",
                )))
            }
            "--evidence" => {
                evidence_dir = Some(std::path::PathBuf::from(take_value(
                    args,
                    &mut i,
                    "--evidence",
                )))
            }
            "--output" => {
                output_path = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--output",
                )))
            }
            "--workdir" => {
                workdir = std::path::PathBuf::from(take_value(args, &mut i, "--workdir"))
            }
            "--format" => format = take_value(args, &mut i, "--format"),
            other => fail(&format!(
                "opción desconocida '{}' para derive-contract",
                other
            )),
        }
        i += 1;
    }

    if command.is_empty() {
        fail("derive-contract requiere una orden después de --");
    }
    if format != "human" && format != "toml" {
        fail("--format debe ser human o toml");
    }

    let trace_path =
        trace_path.unwrap_or_else(|| fail("derive-contract requiere --trace TRACE.tsv"));
    let evidence_dir =
        evidence_dir.unwrap_or_else(|| fail("derive-contract requiere --evidence DIR"));
    let output_path =
        output_path.unwrap_or_else(|| fail("derive-contract requiere --output CONTRACT.toml"));
    if output_path.exists() {
        fail(&format!(
            "el archivo de contrato ya existe: {}",
            output_path.display()
        ));
    }

    let spec = causal::CausalSpec::from_toml_path(&spec_path).unwrap_or_else(|error| fail(&error));
    let trace_spec =
        propagation::TraceSpec::from_tsv_path(&trace_path).unwrap_or_else(|error| fail(&error));
    let report = causal::minimize_environment(
        &spec,
        &trace_spec,
        &causal::MinimizeOptions {
            output_dir: evidence_dir.clone(),
            workdir,
            command,
        },
    )
    .unwrap_or_else(|error| fail(&error));

    std::fs::write(
        evidence_dir.join("mce-result.json"),
        report.render_json().as_bytes(),
    )
    .unwrap_or_else(|error| fail(&format!("no se pudo guardar evidencia F5: {}", error)));

    let contract = contract::EnvironmentalContract::derive(&spec, &report)
        .unwrap_or_else(|error| fail(&error));
    let toml = contract.render_toml();
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).unwrap_or_else(|error| {
                fail(&format!(
                    "no se pudo crear '{}': {}",
                    parent.display(),
                    error
                ))
            });
        }
    }
    std::fs::write(&output_path, toml.as_bytes()).unwrap_or_else(|error| {
        fail(&format!(
            "no se pudo guardar '{}': {}",
            output_path.display(),
            error
        ))
    });

    if format == "toml" {
        print!("{}", toml);
    } else {
        print!("{}", contract.render_human());
        println!("Contrato guardado: {}", output_path.display());
        println!("Evidencia F5: {}", evidence_dir.display());
    }
}

fn cmd_check_contract(args: &[String]) {
    if args.is_empty() {
        fail("check-contract requiere un archivo CONTRACT.toml");
    }

    let contract_path = std::path::PathBuf::from(&args[0]);
    let mut environment_path: Option<std::path::PathBuf> = None;
    let mut output_path: Option<std::path::PathBuf> = None;
    let mut format = "human".to_string();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--environment" => {
                environment_path = Some(std::path::PathBuf::from(take_value(
                    args,
                    &mut i,
                    "--environment",
                )))
            }
            "--output" => {
                output_path = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--output",
                )))
            }
            "--format" => format = take_value(args, &mut i, "--format"),
            other => fail(&format!(
                "opción desconocida '{}' para check-contract",
                other
            )),
        }
        i += 1;
    }

    if format != "human" && format != "json" {
        fail("--format debe ser human o json");
    }
    let environment_path = environment_path
        .unwrap_or_else(|| fail("check-contract requiere --environment ENVIRONMENT.toml"));

    let contract = contract::EnvironmentalContract::from_toml_path(&contract_path)
        .unwrap_or_else(|error| fail(&error));
    let environment = contract::EnvironmentSpec::from_toml_path(&environment_path)
        .unwrap_or_else(|error| fail(&error));
    let evaluation = contract.evaluate(&environment);
    let json = evaluation.render_json();

    if let Some(path) = output_path {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).unwrap_or_else(|error| {
                    fail(&format!(
                        "no se pudo crear '{}': {}",
                        parent.display(),
                        error
                    ))
                });
            }
        }
        std::fs::write(&path, json.as_bytes()).unwrap_or_else(|error| {
            fail(&format!(
                "no se pudo guardar '{}': {}",
                path.display(),
                error
            ))
        });
    }

    if format == "json" {
        print!("{}", json);
    } else {
        print!("{}", evaluation.render_human());
    }
    std::process::exit(evaluation.status().exit_code());
}

fn cmd_minimize_environment(args: &[String]) {
    if args.is_empty() {
        fail("minimize-environment requiere un archivo TOML causal");
    }

    let spec_path = std::path::PathBuf::from(&args[0]);
    let mut trace_path: Option<std::path::PathBuf> = None;
    let mut output_dir: Option<std::path::PathBuf> = None;
    let mut workdir = std::env::current_dir().unwrap_or_else(|error| {
        fail(&format!(
            "no se pudo obtener el directorio actual: {}",
            error
        ))
    });
    let mut format = "human".to_string();
    let mut command = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--" => {
                command = args[i + 1..].to_vec();
                break;
            }
            "--trace" => {
                trace_path = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--trace",
                )));
            }
            "--output" => {
                output_dir = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--output",
                )));
            }
            "--workdir" => {
                workdir = std::path::PathBuf::from(take_value(args, &mut i, "--workdir"));
            }
            "--format" => {
                format = take_value(args, &mut i, "--format");
            }
            other => fail(&format!(
                "opción desconocida '{}' para minimize-environment",
                other
            )),
        }
        i += 1;
    }

    let trace_path =
        trace_path.unwrap_or_else(|| fail("minimize-environment requiere --trace TRACE.tsv"));
    let output_dir =
        output_dir.unwrap_or_else(|| fail("minimize-environment requiere --output DIR"));

    if command.is_empty() {
        fail("minimize-environment requiere una orden después de --");
    }

    if format != "human" && format != "json" {
        fail("--format debe ser human o json");
    }

    let spec = causal::CausalSpec::from_toml_path(&spec_path).unwrap_or_else(|error| fail(&error));
    let trace_spec =
        propagation::TraceSpec::from_tsv_path(&trace_path).unwrap_or_else(|error| fail(&error));

    let options = causal::MinimizeOptions {
        output_dir: output_dir.clone(),
        workdir,
        command,
    };

    let report = causal::minimize_environment(&spec, &trace_spec, &options)
        .unwrap_or_else(|error| fail(&error));

    let json = report.render_json();
    std::fs::write(output_dir.join("mce-result.json"), json.as_bytes()).unwrap_or_else(|error| {
        fail(&format!(
            "no se pudo guardar el resultado causal en '{}': {}",
            output_dir.join("mce-result.json").display(),
            error
        ))
    });

    if format == "json" {
        print!("{}", json);
    } else {
        print!("{}", report.render_human());
    }

    std::process::exit(report.status().exit_code());
}

fn cmd_analyze_propagation(args: &[String]) {
    let mut trace_path: Option<std::path::PathBuf> = None;
    let mut left_root: Option<std::path::PathBuf> = None;
    let mut right_root: Option<std::path::PathBuf> = None;
    let mut output: Option<std::path::PathBuf> = None;
    let mut format = "human".to_string();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--trace" => {
                trace_path = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--trace",
                )));
            }
            "--left-root" => {
                left_root = Some(std::path::PathBuf::from(take_value(
                    args,
                    &mut i,
                    "--left-root",
                )));
            }
            "--right-root" => {
                right_root = Some(std::path::PathBuf::from(take_value(
                    args,
                    &mut i,
                    "--right-root",
                )));
            }
            "--format" => {
                format = take_value(args, &mut i, "--format");
            }
            "--output" => {
                output = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--output",
                )));
            }
            other => fail(&format!(
                "opción desconocida '{}' para analyze-propagation",
                other
            )),
        }

        i += 1;
    }

    let trace_path =
        trace_path.unwrap_or_else(|| fail("analyze-propagation requiere --trace ARCHIVO.tsv"));

    if format != "human" && format != "json" {
        fail("--format debe ser human o json");
    }

    let spec =
        propagation::TraceSpec::from_tsv_path(&trace_path).unwrap_or_else(|error| fail(&error));
    let observations =
        propagation::evaluate_trace(&spec, left_root.as_deref(), right_root.as_deref());
    let trace = propagation::PropagationAnalyzer::analyze(observations);
    let report = propagation::PropagationReport::new(trace);

    let rendered = if format == "json" {
        report.render_json()
    } else {
        report.render_human()
    };

    print!("{}", rendered);

    if let Some(path) = output {
        std::fs::write(&path, rendered.as_bytes()).unwrap_or_else(|error| {
            fail(&format!(
                "no se pudo guardar el trazado en '{}': {}",
                path.display(),
                error
            ))
        });
    }

    std::process::exit(report.trace().status().exit_code());
}

fn cmd_compare_artifacts(args: &[String]) {
    let mut oracle_name: Option<String> = None;
    let mut left: Option<std::path::PathBuf> = None;
    let mut right: Option<std::path::PathBuf> = None;
    let mut output: Option<std::path::PathBuf> = None;
    let mut format = "human".to_string();
    let mut text_options = oracle::TextOptions::default();
    let mut text_option_selected = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--oracle" => {
                oracle_name = Some(take_value(args, &mut i, "--oracle"));
            }
            "--left" => {
                left = Some(std::path::PathBuf::from(take_value(args, &mut i, "--left")));
            }
            "--right" => {
                right = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--right",
                )));
            }
            "--format" => {
                format = take_value(args, &mut i, "--format");
            }
            "--output" => {
                output = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--output",
                )));
            }
            "--normalize-line-endings" => {
                text_options.normalize_line_endings = true;
                text_option_selected = true;
            }
            "--trim-trailing-whitespace" => {
                text_options.trim_trailing_whitespace = true;
                text_option_selected = true;
            }
            "--ignore-final-newline" => {
                text_options.ignore_final_newline = true;
                text_option_selected = true;
            }
            other => fail(&format!(
                "opción desconocida '{}' para compare-artifacts",
                other
            )),
        }

        i += 1;
    }

    let oracle_name = oracle_name
        .unwrap_or_else(|| fail("compare-artifacts requiere --oracle byte|text|json|csv"));
    let left = left.unwrap_or_else(|| fail("compare-artifacts requiere --left ARCHIVO"));
    let right = right.unwrap_or_else(|| fail("compare-artifacts requiere --right ARCHIVO"));

    if format != "human" && format != "json" {
        fail("--format debe ser human o json");
    }

    if oracle_name != "text" && text_option_selected {
        fail("las normalizaciones de texto solo son válidas con --oracle text");
    }

    let left_ref = oracle::ArtifactRef::new(left);
    let right_ref = oracle::ArtifactRef::new(right);

    let implementation: Box<dyn oracle::EquivalenceOracle> = match oracle_name.as_str() {
        "byte" => Box::new(oracle::ByteOracle),
        "text" => Box::new(oracle::TextOracle::new(text_options)),
        "json" => Box::new(oracle::JsonOracle),
        "csv" => Box::new(oracle::CsvOracle),
        other => fail(&format!("oráculo no soportado: {}", other)),
    };

    let result = implementation.compare(&left_ref, &right_ref);
    let report =
        oracle::ComparisonReport::new(implementation.name(), &left_ref, &right_ref, result);

    let rendered = if format == "json" {
        report.render_json()
    } else {
        report.render_human()
    };

    print!("{}", rendered);

    if let Some(path) = output {
        std::fs::write(&path, rendered.as_bytes()).unwrap_or_else(|error| {
            fail(&format!(
                "no se pudo guardar el resultado en '{}': {}",
                path.display(),
                error
            ))
        });
    }

    std::process::exit(report.result().status().exit_code());
}

fn cmd_explore(args: &[String]) {
    if args.is_empty() {
        fail("explore requiere un archivo TOML experimental");
    }

    let spec_path = std::path::PathBuf::from(&args[0]);
    let mut output_dir: Option<std::path::PathBuf> = None;
    let mut workdir = std::env::current_dir()
        .unwrap_or_else(|e| fail(&format!("no se pudo obtener el directorio actual: {}", e)));
    let mut command = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--" => {
                command = args[i + 1..].to_vec();
                break;
            }
            "--output" => {
                output_dir = Some(std::path::PathBuf::from(take_value(
                    args, &mut i, "--output",
                )));
            }
            "--workdir" => {
                workdir = std::path::PathBuf::from(take_value(args, &mut i, "--workdir"));
            }
            other => fail(&format!("opción desconocida '{}' para explore", other)),
        }
        i += 1;
    }

    if command.is_empty() {
        fail("explore requiere una orden después de --");
    }

    let spec = experiment::ExperimentSpec::from_toml_path(&spec_path).unwrap_or_else(|e| fail(&e));
    let variants = experiment::plan_variants(&spec).unwrap_or_else(|e| fail(&e));

    let output_dir = output_dir.unwrap_or_else(|| {
        std::path::PathBuf::from(".envmorph")
            .join("experiments")
            .join(spec.name())
    });

    let options = executor::ExploreOptions {
        output_dir,
        workdir,
        command,
    };

    let bundle = executor::explore(&spec, &variants, &options).unwrap_or_else(|e| fail(&e));
    print!("{}", executor::render_explore(&spec, &bundle));

    if bundle.has_failures() {
        std::process::exit(1);
    }
}

fn cmd_capabilities(args: &[String]) {
    if args.len() != 1 {
        fail("capabilities requiere exactamente un archivo TOML experimental");
    }

    let path = std::path::Path::new(&args[0]);
    let spec = experiment::ExperimentSpec::from_toml_path(path).unwrap_or_else(|e| fail(&e));
    let capabilities = capability::probe_spec(&spec).unwrap_or_else(|e| fail(&e));
    print!("{}", capability::render_capabilities(&spec, &capabilities));
}

fn cmd_plan(args: &[String]) {
    if args.len() != 1 {
        fail("plan requiere exactamente un archivo TOML experimental");
    }

    let path = std::path::Path::new(&args[0]);
    let spec = experiment::ExperimentSpec::from_toml_path(path).unwrap_or_else(|e| fail(&e));
    let variants = experiment::plan_variants(&spec).unwrap_or_else(|e| fail(&e));
    print!("{}", experiment::render_plan(&spec, &variants));
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
        "derive-contract" => cmd_derive_contract(rest),
        "check-contract" => cmd_check_contract(rest),
        "minimize-environment" => cmd_minimize_environment(rest),
        "analyze-propagation" => cmd_analyze_propagation(rest),
        "compare-artifacts" => cmd_compare_artifacts(rest),
        "explore" => cmd_explore(rest),
        "capabilities" => cmd_capabilities(rest),
        "plan" => cmd_plan(rest),
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

mod capability;
mod causal;
mod contract;
mod executor;
mod experiment;
mod oracle;
mod propagation;
