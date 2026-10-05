use crate::oracle::{
    ArtifactRef, ByteOracle, CsvOracle, EquivalenceOracle, EquivalenceResult, EquivalenceStatus,
    JsonOracle, TextOptions, TextOracle,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropagationStatus {
    Stable,
    DivergenceStart,
    Propagated,
    Absorbed,
    Unresolved,
}

impl PropagationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::DivergenceStart => "divergence_start",
            Self::Propagated => "propagated",
            Self::Absorbed => "absorbed",
            Self::Unresolved => "unresolved",
        }
    }

    fn reason(self) -> &'static str {
        match self {
            Self::Stable => "no hay una diferencia observable activa en este punto",
            Self::DivergenceStart => {
                "aparece una diferencia observable, sin afirmar todavía una causa"
            }
            Self::Propagated => "la diferencia observable continúa sin una brecha no resuelta",
            Self::Absorbed => "la diferencia observable deja de observarse en este punto",
            Self::Unresolved => {
                "la equivalencia no pudo resolverse y se interrumpe la continuidad del trazado"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceStatus {
    Stable,
    Absorbed,
    Persistent,
    Unresolved,
}

impl TraceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Absorbed => "absorbed",
            Self::Persistent => "persistent",
            Self::Unresolved => "unresolved",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Stable | Self::Absorbed | Self::Persistent => 0,
            Self::Unresolved => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactObservation {
    label: String,
    oracle: String,
    left_artifact: String,
    right_artifact: String,
    equivalence: EquivalenceResult,
}

impl ArtifactObservation {
    pub fn new(
        label: impl Into<String>,
        oracle: impl Into<String>,
        left: &ArtifactRef,
        right: &ArtifactRef,
        equivalence: EquivalenceResult,
    ) -> Self {
        Self {
            label: label.into(),
            oracle: oracle.into(),
            left_artifact: left.path().display().to_string(),
            right_artifact: right.path().display().to_string(),
            equivalence,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn oracle(&self) -> &str {
        &self.oracle
    }

    pub fn left_artifact(&self) -> &str {
        &self.left_artifact
    }

    pub fn right_artifact(&self) -> &str {
        &self.right_artifact
    }

    pub fn equivalence(&self) -> &EquivalenceResult {
        &self.equivalence
    }
}

#[derive(Debug, Clone)]
pub struct PropagationObservation {
    position: usize,
    artifact: ArtifactObservation,
    status: PropagationStatus,
}

impl PropagationObservation {
    pub fn position(&self) -> usize {
        self.position
    }

    pub fn artifact(&self) -> &ArtifactObservation {
        &self.artifact
    }

    pub fn status(&self) -> PropagationStatus {
        self.status
    }
}

#[derive(Debug, Clone)]
pub struct PropagationTrace {
    status: TraceStatus,
    observations: Vec<PropagationObservation>,
    first_observed_divergence: Option<String>,
    absorption_boundaries: Vec<String>,
    unresolved_count: usize,
}

impl PropagationTrace {
    pub fn status(&self) -> TraceStatus {
        self.status
    }

    pub fn observations(&self) -> &[PropagationObservation] {
        &self.observations
    }

    pub fn first_observed_divergence(&self) -> Option<&str> {
        self.first_observed_divergence.as_deref()
    }

    pub fn absorption_boundaries(&self) -> &[String] {
        &self.absorption_boundaries
    }

    pub fn unresolved_count(&self) -> usize {
        self.unresolved_count
    }
}

pub struct PropagationAnalyzer;

impl PropagationAnalyzer {
    pub fn analyze(observations: Vec<ArtifactObservation>) -> PropagationTrace {
        let mut active_difference = false;
        let mut saw_difference = false;
        let mut first_observed_divergence = None;
        let mut absorption_boundaries = Vec::new();
        let mut unresolved_count = 0usize;
        let mut classified = Vec::with_capacity(observations.len());

        for (index, observation) in observations.into_iter().enumerate() {
            let propagation_status = match observation.equivalence().status() {
                EquivalenceStatus::Identical | EquivalenceStatus::Equivalent => {
                    if active_difference {
                        active_difference = false;
                        absorption_boundaries.push(observation.label().to_string());
                        PropagationStatus::Absorbed
                    } else {
                        PropagationStatus::Stable
                    }
                }
                EquivalenceStatus::Different => {
                    saw_difference = true;
                    if active_difference {
                        PropagationStatus::Propagated
                    } else {
                        active_difference = true;
                        if first_observed_divergence.is_none() {
                            first_observed_divergence = Some(observation.label().to_string());
                        }
                        PropagationStatus::DivergenceStart
                    }
                }
                EquivalenceStatus::Missing | EquivalenceStatus::Error => {
                    unresolved_count += 1;
                    active_difference = false;
                    PropagationStatus::Unresolved
                }
            };

            classified.push(PropagationObservation {
                position: index + 1,
                artifact: observation,
                status: propagation_status,
            });
        }

        let status = if unresolved_count > 0 {
            TraceStatus::Unresolved
        } else if active_difference {
            TraceStatus::Persistent
        } else if saw_difference {
            TraceStatus::Absorbed
        } else {
            TraceStatus::Stable
        };

        PropagationTrace {
            status,
            observations: classified,
            first_observed_divergence,
            absorption_boundaries,
            unresolved_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OracleConfig {
    Byte,
    Text(TextOptions),
    Json,
    Csv,
}

impl OracleConfig {
    fn name(self) -> &'static str {
        match self {
            Self::Byte => "byte",
            Self::Text(_) => "text",
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObservationSpec {
    label: String,
    oracle: OracleConfig,
    left_artifact: PathBuf,
    right_artifact: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TraceSpec {
    observations: Vec<ObservationSpec>,
}

impl TraceSpec {
    pub fn from_tsv_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|error| {
            format!("no se pudo leer el trazado '{}': {}", path.display(), error)
        })?;
        Self::from_tsv_str(&text)
    }

    pub fn from_tsv_str(text: &str) -> Result<Self, String> {
        let mut lines = text
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty() && !line.trim_start().starts_with('#'));

        let Some((header_index, header)) = lines.next() else {
            return Err("el trazado TSV está vacío".into());
        };

        if header != "label\toracle\tleft_artifact\tright_artifact\toptions" {
            return Err(format!(
                "encabezado TSV no válido en la línea {}",
                header_index + 1
            ));
        }

        let mut observations = Vec::new();
        let mut labels = BTreeSet::new();

        for (line_index, line) in lines {
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() != 5 {
                return Err(format!(
                    "la línea {} debe tener cinco columnas TSV",
                    line_index + 1
                ));
            }

            let label = fields[0].trim();
            let oracle_name = fields[1].trim();
            let left = fields[2].trim();
            let right = fields[3].trim();
            let options = fields[4].trim();

            if label.is_empty() {
                return Err(format!("label vacío en la línea {}", line_index + 1));
            }
            if left.is_empty() || right.is_empty() {
                return Err(format!(
                    "ruta de artefacto vacía en la línea {}",
                    line_index + 1
                ));
            }
            if !labels.insert(label.to_string()) {
                return Err(format!("label duplicado: {}", label));
            }

            let oracle = parse_oracle_config(oracle_name, options)
                .map_err(|error| format!("línea {}: {}", line_index + 1, error))?;

            observations.push(ObservationSpec {
                label: label.to_string(),
                oracle,
                left_artifact: PathBuf::from(left),
                right_artifact: PathBuf::from(right),
            });
        }

        if observations.is_empty() {
            return Err("el trazado debe contener al menos una observación".into());
        }

        Ok(Self { observations })
    }

    pub fn len(&self) -> usize {
        self.observations.len()
    }
}

pub fn evaluate_trace(
    spec: &TraceSpec,
    left_root: Option<&Path>,
    right_root: Option<&Path>,
) -> Vec<ArtifactObservation> {
    spec.observations
        .iter()
        .map(|observation| {
            let left_path = resolve_artifact(left_root, &observation.left_artifact);
            let right_path = resolve_artifact(right_root, &observation.right_artifact);
            let left = ArtifactRef::new(left_path);
            let right = ArtifactRef::new(right_path);

            let result = match observation.oracle {
                OracleConfig::Byte => ByteOracle.compare(&left, &right),
                OracleConfig::Text(options) => TextOracle::new(options).compare(&left, &right),
                OracleConfig::Json => JsonOracle.compare(&left, &right),
                OracleConfig::Csv => CsvOracle.compare(&left, &right),
            };

            ArtifactObservation::new(
                observation.label.clone(),
                observation.oracle.name(),
                &left,
                &right,
                result,
            )
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct PropagationReport {
    trace: PropagationTrace,
}

impl PropagationReport {
    pub fn new(trace: PropagationTrace) -> Self {
        Self { trace }
    }

    pub fn trace(&self) -> &PropagationTrace {
        &self.trace
    }

    pub fn render_human(&self) -> String {
        let first = self.trace.first_observed_divergence().unwrap_or("ninguna");
        let boundaries = if self.trace.absorption_boundaries().is_empty() {
            "ninguna".to_string()
        } else {
            self.trace.absorption_boundaries().join(", ")
        };

        let mut out = String::new();
        out.push_str(&format!(
            "Estado del trazado: {}\n",
            self.trace.status().as_str()
        ));
        out.push_str(&format!(
            "Observaciones: {}\n",
            self.trace.observations().len()
        ));
        out.push_str(&format!("Primera divergencia observada: {}\n", first));
        out.push_str(&format!("Fronteras de absorción: {}\n", boundaries));
        out.push_str(&format!(
            "Observaciones no resueltas: {}\n",
            self.trace.unresolved_count()
        ));

        for observation in self.trace.observations() {
            let artifact = observation.artifact();
            out.push_str(&format!(
                "\n[{}] {}\n",
                observation.position(),
                artifact.label()
            ));
            out.push_str(&format!("  Oráculo: {}\n", artifact.oracle()));
            out.push_str(&format!(
                "  Equivalencia: {}\n",
                artifact.equivalence().status().as_str()
            ));
            out.push_str(&format!(
                "  Propagación: {}\n",
                observation.status().as_str()
            ));
            out.push_str(&format!(
                "  Artefacto izquierdo: {}\n",
                artifact.left_artifact()
            ));
            out.push_str(&format!(
                "  Artefacto derecho: {}\n",
                artifact.right_artifact()
            ));
            out.push_str(&format!(
                "  Razón F3: {}\n",
                artifact.equivalence().reason()
            ));
            out.push_str(&format!("  Razón F4: {}\n", observation.status().reason()));
        }

        out
    }

    pub fn render_json(&self) -> String {
        let first = self
            .trace
            .first_observed_divergence()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_string());

        let boundaries = self
            .trace
            .absorption_boundaries()
            .iter()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .collect::<Vec<_>>()
            .join(",");

        let observations = self
            .trace
            .observations()
            .iter()
            .map(render_observation_json)
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"trace_status\":\"{}\",\"first_observed_divergence\":{},\"absorption_boundaries\":[{}],\"unresolved_count\":{},\"observations\":[{}]}}\n",
            self.trace.status().as_str(),
            first,
            boundaries,
            self.trace.unresolved_count(),
            observations
        )
    }
}

fn parse_oracle_config(name: &str, options: &str) -> Result<OracleConfig, String> {
    let normalized_options = options.trim();

    match name {
        "byte" => {
            reject_options(name, normalized_options)?;
            Ok(OracleConfig::Byte)
        }
        "json" => {
            reject_options(name, normalized_options)?;
            Ok(OracleConfig::Json)
        }
        "csv" => {
            reject_options(name, normalized_options)?;
            Ok(OracleConfig::Csv)
        }
        "text" => Ok(OracleConfig::Text(parse_text_options(normalized_options)?)),
        other => Err(format!("oráculo no soportado: {}", other)),
    }
}

fn reject_options(oracle: &str, options: &str) -> Result<(), String> {
    if options.is_empty() || options == "-" {
        Ok(())
    } else {
        Err(format!(
            "las opciones '{}' solo son válidas para el oráculo text, no para {}",
            options, oracle
        ))
    }
}

fn parse_text_options(options: &str) -> Result<TextOptions, String> {
    if options.is_empty() || options == "-" {
        return Ok(TextOptions::default());
    }

    let mut parsed = TextOptions::default();
    let mut seen = BTreeSet::new();

    for option in options
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !seen.insert(option.to_string()) {
            return Err(format!("opción de texto duplicada: {}", option));
        }

        match option {
            "normalize_line_endings" => parsed.normalize_line_endings = true,
            "trim_trailing_whitespace" => parsed.trim_trailing_whitespace = true,
            "ignore_final_newline" => parsed.ignore_final_newline = true,
            other => return Err(format!("opción de texto no soportada: {}", other)),
        }
    }

    Ok(parsed)
}

fn resolve_artifact(root: Option<&Path>, artifact: &Path) -> PathBuf {
    if artifact.is_absolute() {
        artifact.to_path_buf()
    } else if let Some(root) = root {
        root.join(artifact)
    } else {
        artifact.to_path_buf()
    }
}

fn render_observation_json(observation: &PropagationObservation) -> String {
    let artifact = observation.artifact();
    let metadata = render_metadata_json(artifact.equivalence().metadata());

    format!(
        "{{\"position\":{},\"label\":\"{}\",\"oracle\":\"{}\",\"left_artifact\":\"{}\",\"right_artifact\":\"{}\",\"equivalence\":\"{}\",\"equivalence_reason\":\"{}\",\"equivalence_metadata\":{},\"propagation\":\"{}\",\"propagation_reason\":\"{}\"}}",
        observation.position(),
        json_escape(artifact.label()),
        json_escape(artifact.oracle()),
        json_escape(artifact.left_artifact()),
        json_escape(artifact.right_artifact()),
        artifact.equivalence().status().as_str(),
        json_escape(artifact.equivalence().reason()),
        metadata,
        observation.status().as_str(),
        json_escape(observation.status().reason())
    )
}

fn render_metadata_json(metadata: &BTreeMap<String, String>) -> String {
    let entries = metadata
        .iter()
        .map(|(key, value)| format!("\"{}\":\"{}\"", json_escape(key), json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");

    format!("{{{}}}", entries)
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!(
            "envmorph-propagation-test-{}-{}",
            std::process::id(),
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn byte_observation(label: &str, left: &[u8], right: &[u8]) -> ArtifactObservation {
        let left_ref = ArtifactRef::new(temp_file("left.bin", left));
        let right_ref = ArtifactRef::new(temp_file("right.bin", right));
        let result = ByteOracle.compare(&left_ref, &right_ref);
        ArtifactObservation::new(label, "byte", &left_ref, &right_ref, result)
    }

    #[test]
    fn stable_trace_has_no_divergence() {
        let trace = PropagationAnalyzer::analyze(vec![
            byte_observation("a", b"x", b"x"),
            byte_observation("b", b"y", b"y"),
        ]);

        assert_eq!(trace.status(), TraceStatus::Stable);
        assert_eq!(trace.first_observed_divergence(), None);
        assert!(trace.absorption_boundaries().is_empty());
        assert!(trace
            .observations()
            .iter()
            .all(|observation| observation.status() == PropagationStatus::Stable));
    }

    #[test]
    fn difference_can_propagate_and_be_absorbed() {
        let trace = PropagationAnalyzer::analyze(vec![
            byte_observation("a", b"x", b"x"),
            byte_observation("b", b"left", b"right"),
            byte_observation("c", b"left-2", b"right-2"),
            byte_observation("d", b"same", b"same"),
        ]);

        assert_eq!(trace.status(), TraceStatus::Absorbed);
        assert_eq!(trace.first_observed_divergence(), Some("b"));
        assert_eq!(trace.absorption_boundaries(), &["d".to_string()]);
        assert_eq!(
            trace.observations()[1].status(),
            PropagationStatus::DivergenceStart
        );
        assert_eq!(
            trace.observations()[2].status(),
            PropagationStatus::Propagated
        );
        assert_eq!(
            trace.observations()[3].status(),
            PropagationStatus::Absorbed
        );
    }

    #[test]
    fn persistent_difference_is_not_an_error() {
        let trace = PropagationAnalyzer::analyze(vec![
            byte_observation("a", b"left", b"right"),
            byte_observation("b", b"left-2", b"right-2"),
        ]);

        assert_eq!(trace.status(), TraceStatus::Persistent);
        assert_eq!(trace.status().exit_code(), 0);
    }

    #[test]
    fn unresolved_observation_breaks_continuity() {
        let left = ArtifactRef::new(temp_file("left.bin", b"x"));
        let missing =
            ArtifactRef::new(env::temp_dir().join("envmorph-propagation-definitely-missing"));
        let missing_result = ByteOracle.compare(&left, &missing);
        let unresolved = ArtifactObservation::new("gap", "byte", &left, &missing, missing_result);

        let trace = PropagationAnalyzer::analyze(vec![
            byte_observation("a", b"left", b"right"),
            unresolved,
            byte_observation("b", b"left-2", b"right-2"),
        ]);

        assert_eq!(trace.status(), TraceStatus::Unresolved);
        assert_eq!(trace.unresolved_count(), 1);
        assert_eq!(
            trace.observations()[1].status(),
            PropagationStatus::Unresolved
        );
        assert_eq!(
            trace.observations()[2].status(),
            PropagationStatus::DivergenceStart
        );
    }

    #[test]
    fn second_divergence_after_absorption_starts_a_new_segment() {
        let trace = PropagationAnalyzer::analyze(vec![
            byte_observation("a", b"1", b"2"),
            byte_observation("b", b"same", b"same"),
            byte_observation("c", b"3", b"4"),
            byte_observation("d", b"same-2", b"same-2"),
        ]);

        assert_eq!(trace.status(), TraceStatus::Absorbed);
        assert_eq!(trace.absorption_boundaries().len(), 2);
        assert_eq!(
            trace.observations()[2].status(),
            PropagationStatus::DivergenceStart
        );
    }

    #[test]
    fn trace_spec_parses_explicit_text_options() {
        let spec = TraceSpec::from_tsv_str(
            "label\toracle\tleft_artifact\tright_artifact\toptions\n\
             text\ttext\ta.txt\tb.txt\tnormalize_line_endings,ignore_final_newline\n",
        )
        .unwrap();

        assert_eq!(spec.len(), 1);
    }

    #[test]
    fn trace_spec_rejects_options_for_non_text_oracle() {
        let result = TraceSpec::from_tsv_str(
            "label\toracle\tleft_artifact\tright_artifact\toptions\n\
             data\tjson\ta.json\tb.json\tnormalize_line_endings\n",
        );

        assert!(result.is_err());
    }

    #[test]
    fn json_report_is_deterministic() {
        let trace = PropagationAnalyzer::analyze(vec![
            byte_observation("a", b"left", b"right"),
            byte_observation("b", b"same", b"same"),
        ]);
        let report = PropagationReport::new(trace);

        assert_eq!(report.render_json(), report.render_json());
        assert!(report
            .render_json()
            .contains("\"first_observed_divergence\":\"a\""));
        assert!(report
            .render_json()
            .contains("\"absorption_boundaries\":[\"b\"]"));
    }
}
