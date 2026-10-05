use crate::executor::{explore, ExecutionStatus, ExploreOptions};
use crate::experiment::{EnvironmentVariant, EnvironmentalFactor, ExperimentSpec, FactorValue};
use crate::propagation::{
    evaluate_trace, PropagationAnalyzer, PropagationTrace, TraceSpec, TraceStatus,
};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const MAX_EXACT_CAUSAL_FACTORS: usize = 10;
pub const MAX_CONFIRMATIONS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalFactor {
    name: String,
    baseline: FactorValue,
    treatment: FactorValue,
}

impl CausalFactor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn baseline(&self) -> &FactorValue {
        &self.baseline
    }

    pub fn treatment(&self) -> &FactorValue {
        &self.treatment
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalSpec {
    name: String,
    schema_version: u32,
    confirmations: usize,
    factors: Vec<CausalFactor>,
}

impl CausalSpec {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let input = fs::read_to_string(path)
            .map_err(|error| format!("no se pudo leer '{}': {error}", path.display()))?;
        Self::from_toml_str(&input)
    }

    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        let raw: RawCausalDocument =
            toml::from_str(input).map_err(|error| format!("TOML causal inválido: {error}"))?;

        if raw.causal.name.is_empty() || raw.causal.name.trim() != raw.causal.name {
            return Err(
                "el nombre del experimento causal no puede estar vacío ni tener espacios externos"
                    .into(),
            );
        }

        if raw.causal.schema_version != 1 {
            return Err(format!(
                "versión de esquema causal no soportada: {}",
                raw.causal.schema_version
            ));
        }

        if raw.causal.confirmations == 0 || raw.causal.confirmations > MAX_CONFIRMATIONS {
            return Err(format!(
                "confirmations debe estar entre 1 y {}",
                MAX_CONFIRMATIONS
            ));
        }

        if raw.factor.is_empty() {
            return Err("el experimento causal debe declarar al menos un factor".into());
        }

        if raw.factor.len() > MAX_EXACT_CAUSAL_FACTORS {
            return Err(format!(
                "F5 admite como máximo {} factores para búsqueda exacta",
                MAX_EXACT_CAUSAL_FACTORS
            ));
        }

        let mut names = BTreeSet::new();
        let mut factors = Vec::with_capacity(raw.factor.len());

        for raw_factor in raw.factor {
            if !names.insert(raw_factor.name.clone()) {
                return Err(format!(
                    "el experimento causal repite el factor '{}'",
                    raw_factor.name
                ));
            }

            let baseline = FactorValue::new(raw_factor.baseline)?;
            let treatment = FactorValue::new(raw_factor.treatment)?;

            EnvironmentalFactor::new(
                raw_factor.name.clone(),
                vec![baseline.clone(), treatment.clone()],
            )?;

            factors.push(CausalFactor {
                name: raw_factor.name,
                baseline,
                treatment,
            });
        }

        Ok(Self {
            name: raw.causal.name,
            schema_version: raw.causal.schema_version,
            confirmations: raw.causal.confirmations,
            factors,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn confirmations(&self) -> usize {
        self.confirmations
    }

    pub fn factors(&self) -> &[CausalFactor] {
        &self.factors
    }

    fn experiment_spec(&self) -> Result<ExperimentSpec, String> {
        let mut text = String::new();
        text.push_str("[experiment]\n");
        text.push_str(&format!(
            "name = \"{}\"\n",
            toml_escape(&format!("{}-f5", self.name))
        ));
        text.push_str("schema_version = 1\n");

        for factor in &self.factors {
            text.push_str("\n[[factor]]\n");
            text.push_str(&format!("name = \"{}\"\n", toml_escape(factor.name())));
            text.push_str(&format!(
                "values = [\"{}\", \"{}\"]\n",
                toml_escape(factor.baseline().as_str()),
                toml_escape(factor.treatment().as_str())
            ));
        }

        ExperimentSpec::from_toml_str(&text)
    }
}

fn default_confirmations() -> usize {
    2
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCausalDocument {
    causal: RawCausalHeader,
    #[serde(default)]
    factor: Vec<RawCausalFactor>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCausalHeader {
    name: String,
    schema_version: u32,
    #[serde(default = "default_confirmations")]
    confirmations: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCausalFactor {
    name: String,
    baseline: String,
    treatment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SignatureObservation {
    label: String,
    equivalence: String,
    propagation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceSignature {
    trace_status: String,
    first_observed_divergence: Option<String>,
    absorption_boundaries: Vec<String>,
    observations: Vec<SignatureObservation>,
}

impl TraceSignature {
    fn from_trace(trace: &PropagationTrace) -> Self {
        Self {
            trace_status: trace.status().as_str().to_string(),
            first_observed_divergence: trace.first_observed_divergence().map(str::to_string),
            absorption_boundaries: trace.absorption_boundaries().to_vec(),
            observations: trace
                .observations()
                .iter()
                .map(|observation| SignatureObservation {
                    label: observation.artifact().label().to_string(),
                    equivalence: observation
                        .artifact()
                        .equivalence()
                        .status()
                        .as_str()
                        .to_string(),
                    propagation: observation.status().as_str().to_string(),
                })
                .collect(),
        }
    }

    pub fn trace_status(&self) -> &str {
        &self.trace_status
    }

    pub fn first_observed_divergence(&self) -> Option<&str> {
        self.first_observed_divergence.as_deref()
    }

    pub fn absorption_boundaries(&self) -> &[String] {
        &self.absorption_boundaries
    }

    fn is_stable(&self) -> bool {
        self.trace_status == "stable"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CausalStatus {
    Minimal,
    NoObservableEffect,
}

impl CausalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::NoObservableEffect => "no_observable_effect",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Minimal => 0,
            Self::NoObservableEffect => 1,
        }
    }
}

#[derive(Debug, Clone)]
struct ExactSearchResult {
    minimal_subsets: Vec<Vec<usize>>,
    evaluated_subsets: usize,
}

#[derive(Debug, Clone)]
pub struct MinimalCausalEnvironment {
    status: CausalStatus,
    factor_count: usize,
    confirmations: usize,
    target_signature: TraceSignature,
    minimal_sets: Vec<Vec<String>>,
    essential_factors: Vec<String>,
    evaluated_subsets: usize,
}

impl MinimalCausalEnvironment {
    pub fn status(&self) -> CausalStatus {
        self.status
    }

    pub fn minimal_sets(&self) -> &[Vec<String>] {
        &self.minimal_sets
    }

    pub fn essential_factors(&self) -> &[String] {
        &self.essential_factors
    }

    pub fn minimal_cardinality(&self) -> Option<usize> {
        self.minimal_sets.first().map(Vec::len)
    }

    pub fn target_signature(&self) -> &TraceSignature {
        &self.target_signature
    }

    pub fn render_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Estado MCE: {}\n", self.status.as_str()));
        out.push_str("Método de búsqueda: exact_cardinality\n");
        out.push_str(&format!("Factores declarados: {}\n", self.factor_count));
        out.push_str(&format!("Confirmaciones: {}\n", self.confirmations));
        out.push_str(&format!(
            "Subconjuntos evaluados: {}\n",
            self.evaluated_subsets
        ));
        out.push_str(&format!(
            "Firma objetivo F4: {}\n",
            self.target_signature.trace_status()
        ));
        out.push_str(&format!(
            "Primera divergencia objetivo: {}\n",
            self.target_signature
                .first_observed_divergence()
                .unwrap_or("ninguna")
        ));

        let boundaries = if self.target_signature.absorption_boundaries().is_empty() {
            "ninguna".to_string()
        } else {
            self.target_signature.absorption_boundaries().join(", ")
        };
        out.push_str(&format!(
            "Fronteras de absorción objetivo: {}\n",
            boundaries
        ));

        match self.status {
            CausalStatus::Minimal => {
                out.push_str(&format!(
                    "Cardinalidad mínima: {}\n",
                    self.minimal_cardinality().unwrap_or(0)
                ));
                out.push_str(&format!("Conjuntos mínimos: {}\n", self.minimal_sets.len()));
                for (index, set) in self.minimal_sets.iter().enumerate() {
                    out.push_str(&format!("MCE {}: {}\n", index + 1, set.join(", ")));
                }
                let essential = if self.essential_factors.is_empty() {
                    "ninguno".to_string()
                } else {
                    self.essential_factors.join(", ")
                };
                out.push_str(&format!("Factores esenciales: {}\n", essential));
            }
            CausalStatus::NoObservableEffect => {
                out.push_str("Cardinalidad mínima: ninguna\n");
                out.push_str("Conjuntos mínimos: 0\n");
                out.push_str("La intervención completa no produce una diferencia observable.\n");
            }
        }

        out.push_str(
            "Alcance causal: suficiencia operacional exacta dentro de los factores declarados, no prueba de causalidad física.\n",
        );
        out
    }

    pub fn render_json(&self) -> String {
        let minimal_cardinality = self
            .minimal_cardinality()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_string());

        let minimal_sets = self
            .minimal_sets
            .iter()
            .map(|set| {
                let values = set
                    .iter()
                    .map(|value| format!("\"{}\"", json_escape(value)))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("[{}]", values)
            })
            .collect::<Vec<_>>()
            .join(",");

        let essential = self
            .essential_factors
            .iter()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"status\":\"{}\",\"search_method\":\"exact_cardinality\",\"factor_count\":{},\"confirmations\":{},\"evaluated_subsets\":{},\"minimal_cardinality\":{},\"minimal_sets\":[{}],\"essential_factors\":[{}],\"target_signature\":{},\"scope\":\"declared_factors_only\",\"causality_claim\":\"operational_sufficiency_not_physical_proof\"}}\n",
            self.status.as_str(),
            self.factor_count,
            self.confirmations,
            self.evaluated_subsets,
            minimal_cardinality,
            minimal_sets,
            essential,
            render_signature_json(&self.target_signature)
        )
    }
}

#[derive(Debug, Clone)]
pub struct MinimizeOptions {
    pub output_dir: PathBuf,
    pub workdir: PathBuf,
    pub command: Vec<String>,
}

pub fn minimize_environment(
    spec: &CausalSpec,
    trace_spec: &TraceSpec,
    options: &MinimizeOptions,
) -> Result<MinimalCausalEnvironment, String> {
    validate_options(options)?;
    let experiment_spec = spec.experiment_spec()?;

    fs::create_dir_all(options.output_dir.join("target")).map_err(|error| {
        format!(
            "no se pudo crear el directorio causal '{}': {}",
            options.output_dir.display(),
            error
        )
    })?;
    fs::create_dir_all(options.output_dir.join("search"))
        .map_err(|error| format!("no se pudo crear el directorio de búsqueda: {error}"))?;

    let baseline_indices: Vec<usize> = Vec::new();
    let full_indices = (0..spec.factors().len()).collect::<Vec<_>>();
    let mut baseline_roots = Vec::with_capacity(spec.confirmations());
    let mut target_signature: Option<TraceSignature> = None;

    for confirmation in 1..=spec.confirmations() {
        let baseline_dir = options
            .output_dir
            .join("target")
            .join(format!("baseline-{confirmation:03}"));
        let full_dir = options
            .output_dir
            .join("target")
            .join(format!("full-{confirmation:03}"));

        let baseline_root = execute_assignment(
            spec,
            &experiment_spec,
            &baseline_indices,
            options,
            &baseline_dir,
        )?;
        let full_root =
            execute_assignment(spec, &experiment_spec, &full_indices, options, &full_dir)?;

        let signature = evaluate_signature(trace_spec, &baseline_root, &full_root)?;

        if let Some(expected) = &target_signature {
            if expected != &signature {
                return Err(format!(
                    "la intervención completa no reproduce una firma F4 estable entre confirmaciones, divergencia en la confirmación {}",
                    confirmation
                ));
            }
        } else {
            target_signature = Some(signature);
        }

        baseline_roots.push(baseline_root);
    }

    let target_signature =
        target_signature.ok_or_else(|| "no se pudo construir la firma objetivo F4".to_string())?;

    if target_signature.is_stable() {
        return Ok(MinimalCausalEnvironment {
            status: CausalStatus::NoObservableEffect,
            factor_count: spec.factors().len(),
            confirmations: spec.confirmations(),
            target_signature,
            minimal_sets: Vec::new(),
            essential_factors: Vec::new(),
            evaluated_subsets: 1,
        });
    }

    let mut candidate_counter = 0usize;
    let search = exact_minimal_subsets(spec.factors().len(), |indices| {
        candidate_counter += 1;
        let set_dir = options
            .output_dir
            .join("search")
            .join(format!("set-{candidate_counter:04}-k{:02}", indices.len()));

        for confirmation in 1..=spec.confirmations() {
            let candidate_dir = set_dir.join(format!("confirmation-{confirmation:03}"));
            let candidate_root =
                execute_assignment(spec, &experiment_spec, indices, options, &candidate_dir)?;
            let signature = evaluate_signature(
                trace_spec,
                &baseline_roots[confirmation - 1],
                &candidate_root,
            )?;

            if signature != target_signature {
                return Ok(false);
            }
        }

        Ok(true)
    })?;

    let minimal_sets = search
        .minimal_subsets
        .iter()
        .map(|indices| {
            indices
                .iter()
                .map(|index| spec.factors()[*index].name().to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let essential_factors = essential_factors(&minimal_sets);

    Ok(MinimalCausalEnvironment {
        status: CausalStatus::Minimal,
        factor_count: spec.factors().len(),
        confirmations: spec.confirmations(),
        target_signature,
        minimal_sets,
        essential_factors,
        evaluated_subsets: search.evaluated_subsets + 1,
    })
}

fn validate_options(options: &MinimizeOptions) -> Result<(), String> {
    if options.command.is_empty() {
        return Err("minimize-environment requiere una orden después de --".into());
    }

    if !options.workdir.is_dir() {
        return Err(format!(
            "el directorio de trabajo no existe: {}",
            options.workdir.display()
        ));
    }

    if options.output_dir.exists() {
        return Err(format!(
            "el directorio de salida causal ya existe: {}",
            options.output_dir.display()
        ));
    }

    Ok(())
}

fn execute_assignment(
    causal_spec: &CausalSpec,
    experiment_spec: &ExperimentSpec,
    treated_indices: &[usize],
    options: &MinimizeOptions,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    let treated = treated_indices.iter().copied().collect::<BTreeSet<_>>();
    let assignments = causal_spec
        .factors()
        .iter()
        .enumerate()
        .map(|(index, factor)| {
            let value = if treated.contains(&index) {
                factor.treatment().clone()
            } else {
                factor.baseline().clone()
            };
            (factor.name().to_string(), value)
        })
        .collect::<Vec<_>>();

    let variant = EnvironmentVariant::new(1, assignments)?;
    let variants = vec![variant];
    let explore_options = ExploreOptions {
        output_dir: output_dir.to_path_buf(),
        workdir: options.workdir.clone(),
        command: options.command.clone(),
    };

    let bundle = explore(experiment_spec, &variants, &explore_options)?;
    let execution = bundle
        .executions()
        .first()
        .ok_or_else(|| "F2 no devolvió ejecución para la intervención".to_string())?;

    match execution.status() {
        ExecutionStatus::ExecutionSucceeded => Ok(execution.directory().to_path_buf()),
        ExecutionStatus::Unavailable => Err(format!(
            "la intervención '{}' no está disponible en el entorno actual",
            assignment_label(causal_spec, treated_indices)
        )),
        ExecutionStatus::ExecutionFailed => Err(format!(
            "la intervención '{}' terminó con execution_failed",
            assignment_label(causal_spec, treated_indices)
        )),
    }
}

fn evaluate_signature(
    trace_spec: &TraceSpec,
    baseline_root: &Path,
    candidate_root: &Path,
) -> Result<TraceSignature, String> {
    let observations = evaluate_trace(trace_spec, Some(baseline_root), Some(candidate_root));
    let trace = PropagationAnalyzer::analyze(observations);

    if trace.status() == TraceStatus::Unresolved {
        return Err(format!(
            "F4 produjo un trazado unresolved al comparar '{}' con '{}'",
            baseline_root.display(),
            candidate_root.display()
        ));
    }

    Ok(TraceSignature::from_trace(&trace))
}

fn exact_minimal_subsets<F>(
    factor_count: usize,
    mut reproduces: F,
) -> Result<ExactSearchResult, String>
where
    F: FnMut(&[usize]) -> Result<bool, String>,
{
    if factor_count == 0 {
        return Err("la búsqueda causal requiere al menos un factor".into());
    }

    if factor_count > MAX_EXACT_CAUSAL_FACTORS {
        return Err(format!(
            "la búsqueda exacta admite como máximo {} factores",
            MAX_EXACT_CAUSAL_FACTORS
        ));
    }

    let mut evaluated_subsets = 0usize;

    for cardinality in 1..factor_count {
        let combinations = combinations(factor_count, cardinality);
        let mut matching = Vec::new();

        for subset in combinations {
            evaluated_subsets += 1;
            if reproduces(&subset)? {
                matching.push(subset);
            }
        }

        if !matching.is_empty() {
            return Ok(ExactSearchResult {
                minimal_subsets: matching,
                evaluated_subsets,
            });
        }
    }

    Ok(ExactSearchResult {
        minimal_subsets: vec![(0..factor_count).collect()],
        evaluated_subsets,
    })
}

fn combinations(factor_count: usize, cardinality: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut current = Vec::with_capacity(cardinality);
    build_combinations(0, factor_count, cardinality, &mut current, &mut out);
    out
}

fn build_combinations(
    start: usize,
    factor_count: usize,
    remaining: usize,
    current: &mut Vec<usize>,
    out: &mut Vec<Vec<usize>>,
) {
    if remaining == 0 {
        out.push(current.clone());
        return;
    }

    if factor_count - start < remaining {
        return;
    }

    let last_start = factor_count - remaining;
    for index in start..=last_start {
        current.push(index);
        build_combinations(index + 1, factor_count, remaining - 1, current, out);
        current.pop();
    }
}

fn essential_factors(minimal_sets: &[Vec<String>]) -> Vec<String> {
    let Some(first) = minimal_sets.first() else {
        return Vec::new();
    };

    first
        .iter()
        .filter(|factor| minimal_sets.iter().all(|set| set.contains(*factor)))
        .cloned()
        .collect()
}

fn assignment_label(spec: &CausalSpec, treated_indices: &[usize]) -> String {
    if treated_indices.is_empty() {
        return "baseline".into();
    }

    let treated = treated_indices.iter().copied().collect::<BTreeSet<_>>();
    spec.factors()
        .iter()
        .enumerate()
        .filter(|(index, _)| treated.contains(index))
        .map(|(_, factor)| factor.name().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn render_signature_json(signature: &TraceSignature) -> String {
    let first = signature
        .first_observed_divergence()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_string());

    let boundaries = signature
        .absorption_boundaries()
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");

    let observations = signature
        .observations
        .iter()
        .map(|observation| {
            format!(
                "{{\"label\":\"{}\",\"equivalence\":\"{}\",\"propagation\":\"{}\"}}",
                json_escape(&observation.label),
                json_escape(&observation.equivalence),
                json_escape(&observation.propagation)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"trace_status\":\"{}\",\"first_observed_divergence\":{},\"absorption_boundaries\":[{}],\"observations\":[{}]}}",
        json_escape(signature.trace_status()),
        first,
        boundaries,
        observations
    )
}

fn toml_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
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

    #[test]
    fn causal_spec_parses_explicit_baseline_and_treatment() {
        let spec = CausalSpec::from_toml_str(
            r#"
[causal]
name = "smoke"
schema_version = 1
confirmations = 2

[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"

[[factor]]
name = "timezone"
baseline = "UTC"
treatment = "America/Lima"
"#,
        )
        .unwrap();

        assert_eq!(spec.name(), "smoke");
        assert_eq!(spec.confirmations(), 2);
        assert_eq!(spec.factors().len(), 2);
        assert_eq!(spec.factors()[0].baseline().as_str(), "C");
        assert_eq!(spec.factors()[1].treatment().as_str(), "America/Lima");
    }

    #[test]
    fn causal_spec_rejects_equal_baseline_and_treatment() {
        let result = CausalSpec::from_toml_str(
            r#"
[causal]
name = "invalid"
schema_version = 1

[[factor]]
name = "locale"
baseline = "C"
treatment = "C"
"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn causal_spec_rejects_duplicate_factors() {
        let result = CausalSpec::from_toml_str(
            r#"
[causal]
name = "duplicates"
schema_version = 1

[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"

[[factor]]
name = "locale"
baseline = "C"
treatment = "en_US.UTF-8"
"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn combinations_are_deterministic() {
        assert_eq!(
            combinations(4, 2),
            vec![
                vec![0, 1],
                vec![0, 2],
                vec![0, 3],
                vec![1, 2],
                vec![1, 3],
                vec![2, 3],
            ]
        );
    }

    #[test]
    fn exact_search_finds_single_minimum_factor() {
        let result = exact_minimal_subsets(3, |subset| Ok(subset == [1])).unwrap();
        assert_eq!(result.minimal_subsets, vec![vec![1]]);
        assert_eq!(result.evaluated_subsets, 3);
    }

    #[test]
    fn exact_search_preserves_interactions() {
        let result = exact_minimal_subsets(3, |subset| Ok(subset == [0, 2])).unwrap();
        assert_eq!(result.minimal_subsets, vec![vec![0, 2]]);
        assert!(result.evaluated_subsets > 3);
    }

    #[test]
    fn exact_search_reports_all_minimum_alternatives() {
        let result = exact_minimal_subsets(3, |subset| Ok(subset == [0] || subset == [2])).unwrap();
        assert_eq!(result.minimal_subsets, vec![vec![0], vec![2]]);
    }

    #[test]
    fn exact_search_falls_back_to_full_intervention() {
        let result = exact_minimal_subsets(2, |_subset| Ok(false)).unwrap();
        assert_eq!(result.minimal_subsets, vec![vec![0, 1]]);
        assert_eq!(result.evaluated_subsets, 2);
    }

    #[test]
    fn exact_search_does_not_hide_evaluation_errors() {
        let result = exact_minimal_subsets(2, |_subset| Err("unresolved".into()));
        assert!(result.is_err());
    }

    #[test]
    fn essential_factor_is_intersection_of_all_minimum_sets() {
        let sets = vec![
            vec!["locale".to_string(), "timezone".to_string()],
            vec!["locale".to_string(), "awk_implementation".to_string()],
        ];
        assert_eq!(essential_factors(&sets), vec!["locale".to_string()]);
    }
}
