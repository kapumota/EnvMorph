//! Informes legibles y JSON para inspect, diff y verify.

use crate::compare::{Cause, RunDiff, Status};
use crate::json::Json;
use crate::model::{short, Manifest};
use crate::store;
use std::collections::BTreeMap;

pub fn iso(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m,
        d,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

pub fn inspect(m: &Manifest) -> String {
    let mut o = String::new();
    o.push_str(&format!("EnvMorph {} ({})\n", m.run_id, m.status));
    o.push_str(&format!("  inicio         {}\n", iso(m.started_ms)));
    let failed = m.stages.iter().filter(|s| s.exit_code != 0).count();
    let total: u64 = m.stages.iter().map(|s| s.duration_ms).sum();
    o.push_str(&format!(
        "  etapas         {} ({} fallaron)\n",
        m.stages.len(),
        failed
    ));
    o.push_str(&format!("  tiempo etapas  {} ms\n", total));

    let mut produced: BTreeMap<String, usize> = BTreeMap::new();
    let mut external: BTreeMap<String, String> = BTreeMap::new();
    let mut read_after: BTreeMap<String, bool> = BTreeMap::new();
    for (i, s) in m.stages.iter().enumerate() {
        for a in &s.inputs {
            if !produced.contains_key(&a.path) {
                external
                    .entry(a.path.clone())
                    .or_insert_with(|| format!("{} {}", short(&a.digest()), a.role));
            }
            read_after.insert(a.path.clone(), true);
        }
        for a in &s.outputs {
            produced.insert(a.path.clone(), i);
            read_after.insert(a.path.clone(), false);
        }
    }
    let finals: Vec<&String> = read_after
        .iter()
        .filter(|(_, r)| !**r)
        .map(|(p, _)| p)
        .collect();
    o.push_str(&format!("  entradas externas {}\n", external.len()));
    for (p, d) in &external {
        o.push_str(&format!("    {}  {}\n", p, d));
    }
    o.push_str(&format!(
        "  salidas terminales {} (ninguna etapa posterior las lee)\n",
        finals.len()
    ));

    o.push_str("  entorno\n");
    o.push_str(&format!(
        "    {} {} {}\n",
        m.env.os, m.env.kernel, m.env.arch
    ));
    for (k, v) in &m.env.vars {
        if k != "PATH" {
            o.push_str(&format!("    {}={}\n", k, v));
        }
    }
    if !m.env.git_commit.is_empty() {
        o.push_str(&format!(
            "    git {}{}\n",
            short(&m.env.git_commit),
            if m.env.git_dirty {
                " (con cambios)"
            } else {
                ""
            }
        ));
    }
    let mut tools: BTreeMap<String, String> = BTreeMap::new();
    for s in &m.stages {
        if let Some(t) = &s.tool {
            tools.insert(t.path.clone(), t.label());
        }
    }
    o.push_str("  herramientas\n");
    for t in tools.values() {
        o.push_str(&format!("    {}\n", t));
    }
    o.push_str("  etapas\n");
    for s in &m.stages {
        let tool = s
            .tool
            .as_ref()
            .map(|t| t.path.rsplit('/').next().unwrap_or("").to_string())
            .unwrap_or_else(|| "?".into());
        o.push_str(&format!(
            "    {:03} {:<24} salida={} {:>6}ms ent={} sal={} herramienta={}\n",
            s.index,
            s.name,
            s.exit_code,
            s.duration_ms,
            s.inputs.len(),
            s.outputs.len(),
            tool
        ));
    }
    o
}

fn first_line_diff(a_sha: &str, b_sha: &str) -> Option<String> {
    let (da, db) = (store::read_object(a_sha)?, store::read_object(b_sha)?);
    let la: Vec<&[u8]> = da.split(|c| *c == b'\n').collect();
    let lb: Vec<&[u8]> = db.split(|c| *c == b'\n').collect();
    let trunc = |b: &[u8]| -> String { String::from_utf8_lossy(b).chars().take(110).collect() };
    for i in 0..la.len().max(lb.len()) {
        let (x, y) = (la.get(i), lb.get(i));
        if x != y {
            return Some(format!(
                "primera línea diferente {}\n        A: {}\n        B: {}",
                i + 1,
                x.map(|b| trunc(b))
                    .unwrap_or_else(|| "<fin del archivo>".into()),
                y.map(|b| trunc(b))
                    .unwrap_or_else(|| "<fin del archivo>".into())
            ));
        }
    }
    None
}

/// Análisis estático de snapshots de scripts de la etapa para buscar primitivas
/// que hacen depender la salida del tiempo, aleatoriedad o host.
const NONDET_PATTERNS: &[(&str, &str)] = &[
    ("systime(", "tiempo de reloj de pared (awk systime)"),
    ("strftime(", "tiempo de reloj de pared (awk strftime)"),
    ("srand(", "semilla aleatoria (awk srand)"),
    ("rand()", "números aleatorios (awk rand)"),
    ("$RANDOM", "números aleatorios ($RANDOM)"),
    ("mktemp", "nombres de archivos temporales (mktemp)"),
    ("/dev/urandom", "bytes aleatorios (/dev/urandom)"),
    ("uuidgen", "identificadores aleatorios (uuidgen)"),
    ("$(date", "tiempo de reloj de pared (date)"),
    ("`date", "tiempo de reloj de pared (date)"),
    ("hostname", "nombre del host"),
];

pub fn nondeterminism_hints(st: &crate::model::Stage) -> Vec<String> {
    let mut hints = Vec::new();
    for a in st.inputs.iter().filter(|a| a.role == "code") {
        let Some(sha) = &a.sha256 else { continue };
        let Some(data) = store::read_object(sha) else {
            continue;
        };
        let text = String::from_utf8_lossy(&data);
        for (line_no, line) in text.lines().enumerate() {
            let code = line.split('#').next().unwrap_or("");
            for (pat, what) in NONDET_PATTERNS {
                if code.contains(pat) {
                    hints.push(format!("{}:{} usa {}", a.path, line_no + 1, what));
                }
            }
        }
    }
    hints
}

fn describe(c: &Cause) -> String {
    match c {
        Cause::CodeChanged(p) => format!("script modificado ({})", p.join(", ")),
        Cause::CommandChanged => "línea de órdenes modificada".to_string(),
        Cause::ToolChanged(a, b) => format!(
            "el ejecutable difiere\n          A {}\n          B {}",
            a, b
        ),
        Cause::ExternalInput(p) => format!(
            "datos de entrada modificados fuera del pipeline ({})",
            p.join(", ")
        ),
        Cause::Propagated(p) => format!(
            "entradas modificadas por una etapa aguas arriba ({})",
            p.join(", ")
        ),
        Cause::EnvVars(v) => format!(
            "el entorno de ejecución difiere ({}), solo candidato",
            v.join(", ")
        ),
        Cause::Unexplained => {
            "ningún elemento registrado difiere, posible no determinismo o dependencia no declarada"
                .to_string()
        }
    }
}

pub fn render_diff(a: &Manifest, b: &Manifest, d: &RunDiff) -> String {
    let mut o = String::new();
    o.push_str(&format!(
        "DIFERENCIA DE EJECUCIONES  A={}  B={}\n",
        a.run_id, b.run_id
    ));
    if let Some(s) = &d.structural {
        o.push_str(&format!("\nDIFERENCIA ESTRUCTURAL\n  {}\n", s));
    }
    if !d.env_diff.is_empty() || !d.notes.is_empty() {
        o.push_str("\nEntorno\n");
        for (k, x, y) in &d.env_diff {
            o.push_str(&format!("  {:<12} A={}  B={}\n", k, x, y));
        }
        for n in &d.notes {
            o.push_str(&format!("  nota: {}\n", n));
        }
    }
    o.push_str("\nEtapas\n");
    for s in &d.stages {
        let label = match s.status {
            Status::Identical => "identical",
            Status::Absorbed => "absorbed (entradas/orden/herramienta difieren, salidas idénticas)",
            Status::Propagated => "propagated",
            Status::Confounded => "confounded (cambio propio más divergencia aguas arriba)",
            Status::Root => "DIVERGIÓ",
        };
        o.push_str(&format!("  {:03} {:<24} {}\n", s.position, s.name, label));
    }
    match d.first_root() {
        None => {
            if d.identical() {
                o.push_str("\nRESULTADO  salidas idénticas en todas las etapas\n");
            } else {
                o.push_str("\nRESULTADO  no se encontró divergencia por etapa, revisa la diferencia estructural\n");
            }
        }
        Some(f) => {
            o.push_str(&format!(
                "\nPRIMERA DIVERGENCIA\n  etapa {:03} {}\n  causas, primero la más probable\n",
                f.position, f.name
            ));
            for (i, c) in f.causes.iter().enumerate() {
                o.push_str(&format!("    {}. [{}] {}\n", i + 1, c.kind(), describe(c)));
            }
            if let Some((x, y)) = f.exit_diff {
                o.push_str(&format!("  código de salida A={} B={}\n", x, y));
            }
            let (sa, sb) = (&a.stages[f.position - 1], &b.stages[f.position - 1]);
            if f.causes.iter().any(|c| matches!(c, Cause::Unexplained)) {
                let hints = nondeterminism_hints(sa);
                if !hints.is_empty() {
                    o.push_str("  indicios estáticos, los scripts de esta etapa contienen\n");
                    for h in hints {
                        o.push_str(&format!("    {}\n", h));
                    }
                }
            }
            for p in &f.outputs_diff {
                let find =
                    |s: &crate::model::Stage| s.outputs.iter().find(|x| &x.path == p).cloned();
                let (oa, ob) = (find(sa), find(sb));
                o.push_str(&format!(
                    "  salida {}\n    A sha256 {}\n    B sha256 {}\n",
                    p,
                    oa.as_ref()
                        .map(|x| short(&x.digest()))
                        .unwrap_or_else(|| "<ausente>".into()),
                    ob.as_ref()
                        .map(|x| short(&x.digest()))
                        .unwrap_or_else(|| "<ausente>".into())
                ));
                if let (Some(x), Some(y)) = (&oa, &ob) {
                    if let (Some(ha), Some(hb)) = (&x.sha256, &y.sha256) {
                        if let Some(t) = first_line_diff(ha, hb) {
                            o.push_str(&format!("    {}\n", t));
                        }
                    }
                }
            }
            let roots = d.roots();
            if roots.len() > 1 {
                o.push_str("  otras divergencias root independientes\n");
                for r in roots.iter().skip(1) {
                    o.push_str(&format!("    etapa {:03} {}\n", r.position, r.name));
                }
            }
            let total = d.stages.len().max(1);
            let candidates = roots.len() + d.count(Status::Confounded);
            o.push_str(&format!(
                "\nResumen  {} etapas: {} identical, {} absorbed, {} root, {} confounded, {} propagated\n",
                total,
                d.count(Status::Identical),
                d.count(Status::Absorbed),
                roots.len(),
                d.count(Status::Confounded),
                d.count(Status::Propagated)
            ));
            o.push_str(&format!(
                "Etapas candidatas a causa  {} de {} ({:.0}% descartadas)\n",
                candidates,
                total,
                100.0 * (1.0 - candidates as f64 / total as f64)
            ));
            if d.count(Status::Confounded) > 0 {
                o.push_str("Las etapas confounded necesitan replay con entradas idénticas para separar su cambio propio de la propagación aguas arriba.\n");
            }
        }
    }
    o
}

pub fn diff_json(a: &Manifest, b: &Manifest, d: &RunDiff) -> Json {
    let status = |s: Status| match s {
        Status::Identical => "identical",
        Status::Absorbed => "absorbed",
        Status::Propagated => "propagated",
        Status::Confounded => "confounded",
        Status::Root => "root",
    };
    let cause_json = |c: &Cause| {
        let detail = match c {
            Cause::CodeChanged(p)
            | Cause::ExternalInput(p)
            | Cause::Propagated(p)
            | Cause::EnvVars(p) => Json::Arr(p.iter().map(|x| Json::str(x.clone())).collect()),
            Cause::ToolChanged(x, y) => Json::Arr(vec![Json::str(x.clone()), Json::str(y.clone())]),
            _ => Json::Arr(vec![]),
        };
        Json::obj(vec![("kind", Json::str(c.kind())), ("detail", detail)])
    };
    let first = match d.first_root() {
        Some(f) => Json::obj(vec![
            ("position", Json::Num(f.position as i64)),
            ("name", Json::str(f.name.clone())),
            (
                "causes",
                Json::Arr(f.causes.iter().map(cause_json).collect()),
            ),
            (
                "outputs",
                Json::Arr(
                    f.outputs_diff
                        .iter()
                        .map(|p| Json::str(p.clone()))
                        .collect(),
                ),
            ),
        ]),
        None => Json::Null,
    };
    Json::obj(vec![
        ("a", Json::str(a.run_id.clone())),
        ("b", Json::str(b.run_id.clone())),
        ("identical", Json::Bool(d.identical())),
        (
            "structural",
            d.structural.clone().map(Json::Str).unwrap_or(Json::Null),
        ),
        ("stage_count", Json::Num(d.stages.len() as i64)),
        ("root_count", Json::Num(d.roots().len() as i64)),
        (
            "confounded_count",
            Json::Num(d.count(Status::Confounded) as i64),
        ),
        (
            "propagated_count",
            Json::Num(d.count(Status::Propagated) as i64),
        ),
        (
            "absorbed_count",
            Json::Num(d.count(Status::Absorbed) as i64),
        ),
        ("first_divergence", first),
        (
            "stages",
            Json::Arr(
                d.stages
                    .iter()
                    .map(|s| {
                        Json::obj(vec![
                            ("position", Json::Num(s.position as i64)),
                            ("name", Json::str(s.name.clone())),
                            ("status", Json::str(status(s.status))),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "environment_diff",
            Json::Arr(
                d.env_diff
                    .iter()
                    .map(|(k, x, y)| {
                        Json::obj(vec![
                            ("name", Json::str(k.clone())),
                            ("a", Json::str(x.clone())),
                            ("b", Json::str(y.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

/// Recalcula los hashes de artefactos en disco y los compara con lo registrado.
/// Detecta archivos modificados fuera de EnvMorph después de la ejecución y
/// escrituras sin etapa registrada cuando afectan una ruta previamente registrada.
pub fn verify(m: &Manifest) -> (String, bool) {
    let mapper = store::PathMapper::new(&m.maps, &store::cwd());
    let mut last_written: BTreeMap<String, (usize, crate::model::Artifact)> = BTreeMap::new();
    let mut first_seen_input: BTreeMap<String, crate::model::Artifact> = BTreeMap::new();
    for (i, s) in m.stages.iter().enumerate() {
        for a in &s.inputs {
            if !last_written.contains_key(&a.path) {
                first_seen_input
                    .entry(a.path.clone())
                    .or_insert_with(|| a.clone());
            }
        }
        for a in &s.outputs {
            last_written.insert(a.path.clone(), (i + 1, a.clone()));
        }
    }
    let mut o = String::new();
    let mut bad = 0;
    let check = |label: &str,
                 rec: &crate::model::Artifact,
                 stage: Option<usize>,
                 o: &mut String,
                 bad: &mut usize| {
        let real = mapper.real(&rec.path);
        let now = store::hash_artifact(&real, rec.path.clone(), &rec.role);
        let status = if now.sha256 == rec.sha256 {
            "ok"
        } else if now.sha256.is_none() {
            "FALTANTE"
        } else {
            "MODIFICADO"
        };
        if status != "ok" {
            *bad += 1;
        }
        o.push_str(&format!(
            "  {:<8} {:<9} {}{}\n",
            status,
            label,
            rec.path,
            stage
                .map(|n| format!("  (etapa {:03})", n))
                .unwrap_or_default()
        ));
    };
    for (p, a) in &first_seen_input {
        let _ = p;
        check("entrada", a, None, &mut o, &mut bad);
    }
    for (_, (n, a)) in &last_written {
        check("salida", a, Some(*n), &mut o, &mut bad);
    }
    let head = format!(
        "VERIFICACIÓN {}  {} artefacto(s) revisados, {} problema(s)\n",
        m.run_id,
        first_seen_input.len() + last_written.len(),
        bad
    );
    (format!("{}{}", head, o), bad == 0)
}
