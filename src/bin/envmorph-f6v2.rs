//! F6-v2: prototipo experimental separado de la interfaz contractual F6-v1.
//! Las inferencias del modelo no constituyen garantías ambientales universales.

#[path = "../hash.rs"]
mod hash;
#[path = "../json.rs"]
mod json;

use json::Json;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process;

const MAX_DIMENSION: usize = 8;
const MAX_STDOUT_BYTES: u64 = 1_048_576;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    experiment: Experiment,
    #[serde(default)]
    run: Vec<RunRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Experiment {
    schema_version: u32,
    dimension: usize,
    oracle: String,
    confirmations: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunRecord {
    assignment: String,
    run_id: String,
    status: String,
    stdout_file: Option<String>,
    stdout_sha256: Option<String>,
}

#[derive(Debug, Clone)]
struct Proof {
    label: bool,
    run_ids: Vec<String>,
    stdout_hashes: Vec<String>,
}

#[derive(Clone, Debug)]
enum Hypothesis {
    ConstantFalse,
    Conjunction(Vec<(usize, bool)>),
}

impl Hypothesis {
    fn predicts(&self, assignment: usize, dimension: usize) -> bool {
        match self {
            Self::ConstantFalse => false,
            Self::Conjunction(literals) => literals.iter().all(|(index, value)| {
                let bit = (assignment >> (dimension - 1 - index)) & 1;
                (bit == 1) == *value
            }),
        }
    }
}

fn hypothesis_family(dimension: usize) -> Vec<Hypothesis> {
    let mut result = vec![
        Hypothesis::ConstantFalse,
        Hypothesis::Conjunction(Vec::new()),
    ];
    for i in 0..dimension {
        for value in [false, true] {
            result.push(Hypothesis::Conjunction(vec![(i, value)]));
        }
    }
    for i in 0..dimension {
        for j in (i + 1)..dimension {
            for first in [false, true] {
                for second in [false, true] {
                    result.push(Hypothesis::Conjunction(vec![(i, first), (j, second)]));
                }
            }
        }
    }
    result
}

fn version_space(dimension: usize, evidence: &BTreeMap<usize, Proof>) -> Vec<Hypothesis> {
    hypothesis_family(dimension)
        .into_iter()
        .filter(|hypothesis| {
            evidence.iter().all(|(assignment, proof)| {
                hypothesis.predicts(*assignment, dimension) == proof.label
            })
        })
        .collect()
}

fn parse_assignment(value: &str, dimension: usize) -> Result<usize, String> {
    if value.len() != dimension || !value.bytes().all(|byte| byte == b'0' || byte == b'1') {
        return Err(format!(
            "asignación '{}' inválida para {} factores binarios",
            value, dimension
        ));
    }
    usize::from_str_radix(value, 2).map_err(|error| error.to_string())
}

fn assignment_string(value: usize, dimension: usize) -> String {
    format!("{:0width$b}", value, width = dimension)
}

fn check_sha256(value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("se requiere un SHA-256 hexadecimal de 64 caracteres en minúsculas".into());
    }
    Ok(())
}

fn verify_stdout(root: &Path, name: &str, expected: &str) -> Result<Vec<u8>, String> {
    check_sha256(expected)?;
    let relative = Path::new(name);
    if relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "ruta de evidencia no relativa o no normalizada: {name}"
        ));
    }
    let resolved = root
        .join(relative)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !resolved.starts_with(root) || !resolved.is_file() {
        return Err(format!(
            "la evidencia '{name}' queda fuera del directorio autorizado"
        ));
    }
    let metadata = fs::metadata(&resolved).map_err(|error| error.to_string())?;
    if metadata.len() > MAX_STDOUT_BYTES {
        return Err(format!("la evidencia '{name}' excede el tamaño permitido"));
    }
    let contents = fs::read(&resolved).map_err(|error| error.to_string())?;
    if hash::hash_bytes(&contents) != expected {
        return Err(format!("SHA-256 de evidencia no coincide: {name}"));
    }
    Ok(contents)
}

fn json_number(number: usize) -> Json {
    Json::Num(number as i64)
}

fn render_report(
    dimension: usize,
    manifest_sha256: &str,
    attempted: usize,
    observed: &BTreeMap<usize, Proof>,
    unresolved_runs: &[usize],
) -> Json {
    let remaining = version_space(dimension, observed);
    let consistent = !remaining.is_empty();
    let mut verified = 0usize;
    let mut inferred = 0usize;
    let mut undecided = 0usize;
    let mut candidate: Option<(usize, usize)> = None;
    let mut cells = Vec::new();
    for assignment in 0..(1usize << dimension) {
        let key = assignment_string(assignment, dimension);
        let (status, label, run_ids, hashes) = if let Some(proof) = observed.get(&assignment) {
            verified += 1;
            (
                "evidence_verified",
                Some(proof.label),
                proof
                    .run_ids
                    .iter()
                    .map(|value| Json::str(value.clone()))
                    .collect(),
                proof
                    .stdout_hashes
                    .iter()
                    .map(|value| Json::str(value.clone()))
                    .collect(),
            )
        } else if unresolved_runs.contains(&assignment) {
            // Una intervención fallida, incompleta o inestable no se sustituye por una predicción.
            undecided += 1;
            ("unresolved", None, Vec::new(), Vec::new())
        } else if consistent {
            let positive = remaining
                .iter()
                .filter(|hypothesis| hypothesis.predicts(assignment, dimension))
                .count();
            let negative = remaining.len() - positive;
            if positive == 0 || negative == 0 {
                inferred += 1;
            } else {
                undecided += 1;
            }
            let score = positive.min(negative);
            if candidate.map(|(_, best)| score > best).unwrap_or(true) {
                candidate = Some((assignment, score));
            }
            if positive == 0 || negative == 0 {
                (
                    "model_supported",
                    Some(positive > 0),
                    Vec::new(),
                    Vec::new(),
                )
            } else {
                ("unresolved", None, Vec::new(), Vec::new())
            }
        } else {
            undecided += 1;
            ("unresolved", None, Vec::new(), Vec::new())
        };
        cells.push(Json::obj(vec![
            ("assignment", Json::str(key)),
            ("status", Json::str(status)),
            ("label", label.map(Json::Bool).unwrap_or(Json::Null)),
            ("run_ids", Json::Arr(run_ids)),
            ("stdout_sha256", Json::Arr(hashes)),
            (
                "invalid_or_incomplete_run_set",
                Json::Bool(unresolved_runs.contains(&assignment)),
            ),
        ]));
    }
    Json::obj(vec![
        ("schema_version", json_number(1)),
        ("method", Json::str("F6-v2-experimental")),
        ("manifest_sha256", Json::str(manifest_sha256.to_string())),
        ("oracle", Json::str("byte_stdout_vs_baseline_v1")),
        ("dimension", json_number(dimension)),
        ("primitive_run_records", json_number(attempted)),
        ("verified_direct", json_number(verified)),
        ("inferred_under_model", json_number(inferred)),
        ("unresolved", json_number(undecided)),
        ("remaining_hypotheses", json_number(remaining.len())),
        ("model_inconsistent", Json::Bool(!consistent)),
        ("model_assumption", Json::str("constantes y conjunciones de hasta dos literales firmados")),
        ("model_warning", Json::str("model_supported es condicional a la realizabilidad de H; no es evidencia de ejecución ni garantía universal")),
        ("trust_warning", Json::str("el hash externo ancla los bytes, pero no prueba la autenticidad de la ejecución ni la independencia física de las repeticiones")),
        ("suggested_next_assignment", candidate.map(|(index, _)| Json::str(assignment_string(index, dimension))).unwrap_or(Json::Null)),
        ("cells", Json::Arr(cells)),
    ])
}

fn evaluate(path: &Path, expected_manifest_sha256: &str) -> Result<Json, String> {
    check_sha256(expected_manifest_sha256)?;
    let bytes =
        fs::read(path).map_err(|error| format!("no se pudo leer el manifiesto: {error}"))?;
    let manifest_sha256 = hash::hash_bytes(&bytes);
    if manifest_sha256 != expected_manifest_sha256 {
        return Err("la huella externa del manifiesto no coincide; no evaluar".into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
    let manifest: Manifest = toml::from_str(text).map_err(|error| error.to_string())?;
    let config = &manifest.experiment;
    if config.schema_version != 1
        || !(1..=MAX_DIMENSION).contains(&config.dimension)
        || config.confirmations != 2
        || config.oracle != "byte_stdout_vs_baseline_v1"
    {
        return Err("esquema o configuración no compatibles con el prototipo experimental".into());
    }
    if manifest.run.len() > (1usize << config.dimension) * 2 {
        return Err("se encontraron más de dos registros por configuración potencial".into());
    }
    let root = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let mut seen_run_ids = BTreeSet::new();
    let mut seen_file_paths = BTreeSet::new();
    let mut grouped: BTreeMap<usize, Vec<(String, Option<Vec<u8>>, Option<String>)>> =
        BTreeMap::new();
    for run in &manifest.run {
        if run.run_id.is_empty()
            || run.run_id.len() > 80
            || !run
                .run_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
            || !seen_run_ids.insert(run.run_id.clone())
        {
            return Err(format!(
                "identificador de ejecución inválido o repetido: {}",
                run.run_id
            ));
        }
        let assignment = parse_assignment(&run.assignment, config.dimension)?;
        let stdout = match (run.stdout_file.as_deref(), run.stdout_sha256.as_deref()) {
            (Some(name), Some(expected)) => {
                let candidate = root
                    .join(name)
                    .canonicalize()
                    .map_err(|error| error.to_string())?;
                if !seen_file_paths.insert(candidate) {
                    return Err("dos ejecuciones comparten el mismo archivo de evidencia".into());
                }
                Some(verify_stdout(&root, name, expected)?)
            }
            (None, None) => None,
            _ => return Err(format!("salida y SHA-256 incompletos para {}", run.run_id)),
        };
        match run.status.as_str() {
            "ok" if stdout.is_some() => {}
            "failed" | "blocked" | "unstable" => {}
            _ => return Err(format!("estado de ejecución inválido para {}", run.run_id)),
        }
        grouped.entry(assignment).or_default().push((
            run.run_id.clone(),
            if run.status == "ok" { stdout } else { None },
            run.stdout_sha256.clone(),
        ));
        if grouped[&assignment].len() > 2 {
            return Err(format!(
                "más de dos ejecuciones en la configuración {}",
                run.assignment
            ));
        }
    }
    let baseline = grouped
        .get(&0)
        .ok_or("faltan las dos observaciones del baseline")?;
    if baseline.len() != 2
        || baseline[0].1.is_none()
        || baseline[1].1.is_none()
        || baseline[0].1 != baseline[1].1
    {
        return Err("el baseline no tiene dos ejecuciones correctas y concordantes".into());
    }
    let reference = baseline[0].1.as_ref().expect("baseline verificado");
    let mut observed = BTreeMap::new();
    let mut unresolved_runs = Vec::new();
    for (&assignment, runs) in &grouped {
        if runs.len() != 2 || runs[0].1.is_none() || runs[1].1.is_none() || runs[0].1 != runs[1].1 {
            unresolved_runs.push(assignment);
            continue;
        }
        let first = runs[0].1.as_ref().expect("salida concordante");
        observed.insert(
            assignment,
            Proof {
                label: first != reference,
                run_ids: runs.iter().map(|record| record.0.clone()).collect(),
                stdout_hashes: runs.iter().filter_map(|record| record.2.clone()).collect(),
            },
        );
    }
    Ok(render_report(
        config.dimension,
        &manifest_sha256,
        manifest.run.len(),
        &observed,
        &unresolved_runs,
    ))
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 7
        || args[0] != "evaluate"
        || args[1] != "--manifest"
        || args[3] != "--expected-manifest-sha256"
        || args[5] != "--output"
    {
        return Err("uso: envmorph-f6v2 evaluate --manifest FILE --expected-manifest-sha256 HEX --output REPORT.json".into());
    }
    let output = PathBuf::from(&args[6]);
    if output.exists() {
        return Err("el archivo de salida ya existe; no se sobrescribirá".into());
    }
    let report = evaluate(Path::new(&args[2]), &args[4])?;
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
    }
    fs::write(&output, report.pretty()).map_err(|error| error.to_string())?;
    println!("M2_F6V2_EVALUATION=PASS");
    println!("M2_F6V2_REPORT={}", output.display());
    println!("M2_SCIENTIFIC_EVIDENCE=NONE_PROTOTYPE_ONLY");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("M2_F6V2_ERROR={error}");
        process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proof(label: bool) -> Proof {
        Proof {
            label,
            run_ids: vec!["first".into(), "second".into()],
            stdout_hashes: vec!["a".into(), "b".into()],
        }
    }

    #[test]
    fn family_size_matches_m1() {
        assert_eq!(hypothesis_family(4).len(), 2 + 8 + 24);
    }

    #[test]
    fn conditional_inference_is_separate_from_direct_evidence() {
        let mut evidence = BTreeMap::new();
        for assignment in 0..8 {
            if assignment != 7 {
                evidence.insert(assignment, proof((assignment & 4) != 0));
            }
        }
        let remaining = version_space(3, &evidence);
        assert!(!remaining.is_empty());
        assert!(remaining.iter().all(|h| h.predicts(7, 3)));
        let report = render_report(3, &"a".repeat(64), 14, &evidence, &[]);
        let cells = report.get("cells").unwrap().as_arr().unwrap();
        assert_eq!(
            cells[0].get("status").unwrap().as_str(),
            Some("evidence_verified")
        );
        assert_eq!(
            cells[7].get("status").unwrap().as_str(),
            Some("model_supported")
        );
    }

    #[test]
    fn xor_three_detected_as_misspecification() {
        let mut evidence = BTreeMap::new();
        for assignment in 0usize..8 {
            evidence.insert(assignment, proof((assignment.count_ones() % 2) == 1));
        }
        assert!(version_space(3, &evidence).is_empty());
        let report = render_report(3, &"a".repeat(64), 16, &evidence, &[]);
        assert_eq!(
            report.get("model_inconsistent").unwrap().as_bool(),
            Some(true)
        );
        assert_eq!(
            report.get("inferred_under_model").unwrap().as_i64(),
            Some(0)
        );
    }

    #[test]
    fn false_model_can_predict_wrong_before_mismatch_is_detected() {
        // Tras observar dos puntos XOR, H puede predecir erróneamente otros.
        // El reporte debe reconocer que model_supported es condicional.
        let mut evidence = BTreeMap::new();
        evidence.insert(0, proof(false));
        evidence.insert(1, proof(true));
        let remaining = version_space(3, &evidence);
        assert!(!remaining.is_empty());
        assert!(remaining.iter().all(|h| !h.predicts(2, 3)));
        assert_eq!((2usize.count_ones() % 2) == 1, true);
        let report = render_report(3, &"a".repeat(64), 4, &evidence, &[]);
        let cells = report.get("cells").unwrap().as_arr().unwrap();
        assert_eq!(
            cells[2].get("status").unwrap().as_str(),
            Some("model_supported")
        );
        assert_eq!(cells[2].get("label").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn invalid_run_cell_must_abstain_even_when_model_is_unanimous() {
        let mut evidence = BTreeMap::new();
        for assignment in 0..8 {
            if assignment != 7 {
                evidence.insert(assignment, proof((assignment & 4) != 0));
            }
        }
        let report = render_report(3, &"a".repeat(64), 14, &evidence, &[7]);
        let cells = report.get("cells").unwrap().as_arr().unwrap();
        assert_eq!(cells[7].get("status").unwrap().as_str(), Some("unresolved"));
        assert_eq!(
            cells[7]
                .get("invalid_or_incomplete_run_set")
                .unwrap()
                .as_bool(),
            Some(true)
        );
    }

    #[test]
    fn invalid_assignment_rejected() {
        assert!(parse_assignment("101", 2).is_err());
        assert!(parse_assignment("1a", 2).is_err());
        assert_eq!(parse_assignment("101", 3).unwrap(), 5);
    }

    #[test]
    fn signed_literals_obey_declared_bit_order() {
        let h = Hypothesis::Conjunction(vec![(0, true), (2, false)]);
        assert!(h.predicts(4, 3));
        assert!(!h.predicts(5, 3));
        assert!(!h.predicts(0, 3));
    }
}
