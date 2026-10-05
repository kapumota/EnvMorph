use crate::causal::{CausalSpec, MinimalCausalEnvironment};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::Path;

pub const CONTRACT_SCHEMA_VERSION: u32 = 1;
const CONTRACT_POLICY: &str = "preserve_reference_behavior";
const CONTRACT_SCOPE: &str = "declared_contrasts_only";
const CONTRACT_CLAIM: &str = "evidence_bounded_contract_not_universal_portability";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractFactor {
    name: String,
    baseline: String,
    treatment: String,
}

impl ContractFactor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn baseline(&self) -> &str {
        &self.baseline
    }

    pub fn treatment(&self) -> &str {
        &self.treatment
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractEvidence {
    causal_status: String,
    confirmations: usize,
    target_trace_status: String,
    first_observed_divergence: Option<String>,
    absorption_boundaries: Vec<String>,
    minimal_sets: Vec<Vec<String>>,
    essential_factors: Vec<String>,
}

impl ContractEvidence {
    pub fn causal_status(&self) -> &str {
        &self.causal_status
    }

    pub fn confirmations(&self) -> usize {
        self.confirmations
    }

    pub fn target_trace_status(&self) -> &str {
        &self.target_trace_status
    }

    pub fn minimal_sets(&self) -> &[Vec<String>] {
        &self.minimal_sets
    }

    pub fn essential_factors(&self) -> &[String] {
        &self.essential_factors
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentalContract {
    name: String,
    source_causal_name: String,
    factors: Vec<ContractFactor>,
    evidence: ContractEvidence,
}

impl EnvironmentalContract {
    pub fn derive(spec: &CausalSpec, mce: &MinimalCausalEnvironment) -> Result<Self, String> {
        let factors = spec
            .factors()
            .iter()
            .map(|factor| ContractFactor {
                name: factor.name().to_string(),
                baseline: factor.baseline().as_str().to_string(),
                treatment: factor.treatment().as_str().to_string(),
            })
            .collect::<Vec<_>>();

        let target = mce.target_signature();
        let evidence = ContractEvidence {
            causal_status: mce.status().as_str().to_string(),
            confirmations: spec.confirmations(),
            target_trace_status: target.trace_status().to_string(),
            first_observed_divergence: target.first_observed_divergence().map(str::to_string),
            absorption_boundaries: target.absorption_boundaries().to_vec(),
            minimal_sets: mce.minimal_sets().to_vec(),
            essential_factors: mce.essential_factors().to_vec(),
        };

        let contract = Self {
            name: format!("{}-environmental-contract", spec.name()),
            source_causal_name: spec.name().to_string(),
            factors,
            evidence,
        };
        contract.validate()?;
        Ok(contract)
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let input = fs::read_to_string(path)
            .map_err(|error| format!("no se pudo leer '{}': {error}", path.display()))?;
        Self::from_toml_str(&input)
    }

    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        let raw: RawContractDocument =
            toml::from_str(input).map_err(|error| format!("TOML de contrato inválido: {error}"))?;

        if raw.contract.schema_version != CONTRACT_SCHEMA_VERSION {
            return Err(format!(
                "versión de esquema de contrato no soportada: {}",
                raw.contract.schema_version
            ));
        }
        if raw.contract.policy != CONTRACT_POLICY {
            return Err(format!(
                "política de contrato no soportada: {}",
                raw.contract.policy
            ));
        }
        if raw.contract.scope != CONTRACT_SCOPE {
            return Err(format!(
                "alcance de contrato no soportado: {}",
                raw.contract.scope
            ));
        }
        if raw.contract.claim != CONTRACT_CLAIM {
            return Err(format!(
                "afirmación de contrato no soportada: {}",
                raw.contract.claim
            ));
        }

        let factors = raw
            .factor
            .into_iter()
            .map(|factor| ContractFactor {
                name: factor.name,
                baseline: factor.baseline,
                treatment: factor.treatment,
            })
            .collect::<Vec<_>>();

        let evidence = ContractEvidence {
            causal_status: raw.evidence.causal_status,
            confirmations: raw.evidence.confirmations,
            target_trace_status: raw.evidence.target_trace_status,
            first_observed_divergence: raw.evidence.first_observed_divergence,
            absorption_boundaries: raw.evidence.absorption_boundaries,
            minimal_sets: raw.minimal_set.into_iter().map(|set| set.factors).collect(),
            essential_factors: raw.evidence.essential_factors,
        };

        let mut contract = Self {
            name: raw.contract.name,
            source_causal_name: raw.contract.source_causal_name,
            factors,
            evidence,
        };
        contract.normalize_sets()?;
        contract.validate()?;
        Ok(contract)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn factors(&self) -> &[ContractFactor] {
        &self.factors
    }

    pub fn evidence(&self) -> &ContractEvidence {
        &self.evidence
    }

    pub fn render_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Contrato: {}\n", self.name));
        out.push_str(&format!("Política: {}\n", CONTRACT_POLICY));
        out.push_str(&format!(
            "Experimento causal: {}\n",
            self.source_causal_name
        ));
        out.push_str(&format!("Factores declarados: {}\n", self.factors.len()));
        out.push_str(&format!(
            "Estado causal F5: {}\n",
            self.evidence.causal_status
        ));
        out.push_str(&format!(
            "Confirmaciones: {}\n",
            self.evidence.confirmations
        ));
        out.push_str(&format!(
            "Firma F4: {}\n",
            self.evidence.target_trace_status
        ));
        out.push_str(&format!(
            "Primera divergencia: {}\n",
            self.evidence
                .first_observed_divergence
                .as_deref()
                .unwrap_or("ninguna")
        ));
        out.push_str(&format!(
            "MCEs conocidos: {}\n",
            self.evidence.minimal_sets.len()
        ));
        for (index, set) in self.evidence.minimal_sets.iter().enumerate() {
            out.push_str(&format!("MCE {}: {}\n", index + 1, set.join(", ")));
        }
        let essential = if self.evidence.essential_factors.is_empty() {
            "ninguno".to_string()
        } else {
            self.evidence.essential_factors.join(", ")
        };
        out.push_str(&format!("Factores esenciales: {}\n", essential));
        out.push_str("Alcance: contrastes declarados y observados, no portabilidad universal.\n");
        out
    }

    pub fn render_toml(&self) -> String {
        let mut out = String::new();
        out.push_str("[contract]\n");
        out.push_str(&format!("name = \"{}\"\n", toml_escape(&self.name)));
        out.push_str(&format!("schema_version = {}\n", CONTRACT_SCHEMA_VERSION));
        out.push_str(&format!("policy = \"{}\"\n", CONTRACT_POLICY));
        out.push_str(&format!(
            "source_causal_name = \"{}\"\n",
            toml_escape(&self.source_causal_name)
        ));
        out.push_str(&format!("scope = \"{}\"\n", CONTRACT_SCOPE));
        out.push_str(&format!("claim = \"{}\"\n\n", CONTRACT_CLAIM));

        out.push_str("[evidence]\n");
        out.push_str(&format!(
            "causal_status = \"{}\"\n",
            toml_escape(&self.evidence.causal_status)
        ));
        out.push_str(&format!(
            "confirmations = {}\n",
            self.evidence.confirmations
        ));
        out.push_str(&format!(
            "target_trace_status = \"{}\"\n",
            toml_escape(&self.evidence.target_trace_status)
        ));
        if let Some(first) = &self.evidence.first_observed_divergence {
            out.push_str(&format!(
                "first_observed_divergence = \"{}\"\n",
                toml_escape(first)
            ));
        }
        out.push_str(&format!(
            "absorption_boundaries = {}\n",
            render_toml_array(&self.evidence.absorption_boundaries)
        ));
        out.push_str(&format!(
            "essential_factors = {}\n",
            render_toml_array(&self.evidence.essential_factors)
        ));

        for factor in &self.factors {
            out.push_str("\n[[factor]]\n");
            out.push_str(&format!("name = \"{}\"\n", toml_escape(&factor.name)));
            out.push_str(&format!(
                "baseline = \"{}\"\n",
                toml_escape(&factor.baseline)
            ));
            out.push_str(&format!(
                "treatment = \"{}\"\n",
                toml_escape(&factor.treatment)
            ));
        }

        for set in &self.evidence.minimal_sets {
            out.push_str("\n[[minimal_set]]\n");
            out.push_str(&format!("factors = {}\n", render_toml_array(set)));
        }

        out
    }

    pub fn evaluate(&self, environment: &EnvironmentSpec) -> ContractEvaluation {
        let values = environment
            .factors
            .iter()
            .map(|factor| (factor.name.as_str(), factor.value.as_str()))
            .collect::<BTreeMap<_, _>>();

        for factor in &self.factors {
            if !values.contains_key(factor.name.as_str()) {
                return ContractEvaluation::new(
                    ContractStatus::Invalid,
                    environment.name(),
                    format!("falta el factor requerido '{}'", factor.name),
                    Vec::new(),
                    None,
                );
            }
        }

        let declared = self
            .factors
            .iter()
            .map(|factor| factor.name.as_str())
            .collect::<BTreeSet<_>>();
        if let Some(extra) = environment
            .factors
            .iter()
            .find(|factor| !declared.contains(factor.name.as_str()))
        {
            return ContractEvaluation::new(
                ContractStatus::OutOfScope,
                environment.name(),
                format!("el factor '{}' no pertenece al contrato", extra.name),
                Vec::new(),
                None,
            );
        }

        let mut treated = Vec::new();
        for factor in &self.factors {
            let value = values[factor.name.as_str()];
            if value == factor.baseline.as_str() {
                continue;
            }
            if value == factor.treatment.as_str() {
                treated.push(factor.name.clone());
                continue;
            }
            return ContractEvaluation::new(
                ContractStatus::OutOfScope,
                environment.name(),
                format!(
                    "el valor '{}' de '{}' no pertenece al contraste baseline/treatment observado",
                    value, factor.name
                ),
                treated,
                None,
            );
        }

        if treated.is_empty() {
            return ContractEvaluation::new(
                ContractStatus::Satisfied,
                environment.name(),
                "la configuración coincide exactamente con el baseline observado".into(),
                treated,
                None,
            );
        }

        let all_treated = treated.len() == self.factors.len();

        if self.evidence.causal_status == "no_observable_effect" {
            if all_treated {
                return ContractEvaluation::new(
                    ContractStatus::Satisfied,
                    environment.name(),
                    "la intervención completa observada preservó el comportamiento de referencia"
                        .into(),
                    treated,
                    None,
                );
            }
            return ContractEvaluation::new(
                ContractStatus::OutOfScope,
                environment.name(),
                "la combinación parcial no fue establecida por la evidencia F5".into(),
                treated,
                None,
            );
        }

        if let Some(matched) = self
            .evidence
            .minimal_sets
            .iter()
            .find(|set| set.as_slice() == treated.as_slice())
            .cloned()
        {
            return ContractEvaluation::new(
                ContractStatus::Violated,
                environment.name(),
                "la configuración coincide con un Minimal Causal Environment observado".into(),
                treated,
                Some(matched),
            );
        }

        if all_treated {
            return ContractEvaluation::new(
                ContractStatus::Violated,
                environment.name(),
                "la configuración coincide con la intervención completa que produjo la firma F4 objetivo"
                    .into(),
                treated,
                None,
            );
        }

        ContractEvaluation::new(
            ContractStatus::OutOfScope,
            environment.name(),
            "la combinación pertenece al contraste declarado, pero F5 no estableció su semántica contractual"
                .into(),
            treated,
            None,
        )
    }

    fn normalize_sets(&mut self) -> Result<(), String> {
        let order = self
            .factors
            .iter()
            .enumerate()
            .map(|(index, factor)| (factor.name.clone(), index))
            .collect::<BTreeMap<_, _>>();

        for set in &mut self.evidence.minimal_sets {
            for name in set.iter() {
                if !order.contains_key(name) {
                    return Err(format!(
                        "el MCE referencia el factor desconocido '{}'",
                        name
                    ));
                }
            }
            set.sort_by_key(|name| order.get(name).copied().unwrap_or(usize::MAX));
        }
        self.evidence
            .minimal_sets
            .sort_by(|left, right| left.cmp(right));
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        validate_nonempty_trimmed(&self.name, "nombre del contrato")?;
        validate_nonempty_trimmed(&self.source_causal_name, "experimento causal de origen")?;

        if self.factors.is_empty() {
            return Err("el contrato debe declarar al menos un factor".into());
        }

        let mut names = HashSet::new();
        for factor in &self.factors {
            validate_factor_name(&factor.name)?;
            validate_nonempty_trimmed(&factor.baseline, "valor baseline")?;
            validate_nonempty_trimmed(&factor.treatment, "valor treatment")?;
            if factor.baseline == factor.treatment {
                return Err(format!(
                    "el factor '{}' no puede tener baseline y treatment iguales",
                    factor.name
                ));
            }
            if !names.insert(factor.name.as_str()) {
                return Err(format!("el contrato repite el factor '{}'", factor.name));
            }
        }

        if self.evidence.confirmations == 0 || self.evidence.confirmations > 5 {
            return Err("el contrato requiere entre 1 y 5 confirmaciones F5".into());
        }

        if !matches!(
            self.evidence.target_trace_status.as_str(),
            "stable" | "absorbed" | "persistent"
        ) {
            return Err(format!(
                "estado F4 objetivo no soportado por el contrato: {}",
                self.evidence.target_trace_status
            ));
        }

        match self.evidence.causal_status.as_str() {
            "minimal" => {
                if self.evidence.minimal_sets.is_empty() {
                    return Err("un contrato causal minimal debe contener al menos un MCE".into());
                }
                if self.evidence.target_trace_status == "stable" {
                    return Err("un contrato causal minimal no puede tener firma F4 stable".into());
                }
            }
            "no_observable_effect" => {
                if !self.evidence.minimal_sets.is_empty() {
                    return Err("un contrato sin efecto observable no puede declarar MCEs".into());
                }
                if !self.evidence.essential_factors.is_empty() {
                    return Err(
                        "un contrato sin efecto observable no puede declarar factores esenciales"
                            .into(),
                    );
                }
                if self.evidence.target_trace_status != "stable" {
                    return Err(
                        "un contrato sin efecto observable debe conservar firma F4 stable".into(),
                    );
                }
            }
            other => return Err(format!("estado causal F5 no soportado: {other}")),
        }

        let declared = self
            .factors
            .iter()
            .map(|factor| factor.name.as_str())
            .collect::<HashSet<_>>();

        let mut cardinality = None;
        let mut seen_sets = BTreeSet::new();
        for set in &self.evidence.minimal_sets {
            if set.is_empty() {
                return Err("un MCE contractual no puede estar vacío".into());
            }
            if let Some(expected) = cardinality {
                if expected != set.len() {
                    return Err("todos los MCEs deben tener la misma cardinalidad mínima".into());
                }
            } else {
                cardinality = Some(set.len());
            }

            let unique = set.iter().collect::<HashSet<_>>();
            if unique.len() != set.len() {
                return Err("un MCE contractual repite factores".into());
            }
            for name in set {
                if !declared.contains(name.as_str()) {
                    return Err(format!(
                        "el MCE referencia el factor desconocido '{}'",
                        name
                    ));
                }
            }
            if !seen_sets.insert(set.clone()) {
                return Err("el contrato repite un MCE".into());
            }
        }

        let expected_essential = essential_factors(&self.factors, &self.evidence.minimal_sets);
        if expected_essential != self.evidence.essential_factors {
            return Err(format!(
                "los factores esenciales no coinciden con la intersección de los MCEs: esperado {:?}",
                expected_essential
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentFactorValue {
    name: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentSpec {
    name: String,
    factors: Vec<EnvironmentFactorValue>,
}

impl EnvironmentSpec {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let input = fs::read_to_string(path)
            .map_err(|error| format!("no se pudo leer '{}': {error}", path.display()))?;
        Self::from_toml_str(&input)
    }

    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        let raw: RawEnvironmentDocument =
            toml::from_str(input).map_err(|error| format!("TOML de entorno inválido: {error}"))?;
        if raw.environment.schema_version != 1 {
            return Err(format!(
                "versión de esquema de entorno no soportada: {}",
                raw.environment.schema_version
            ));
        }
        validate_nonempty_trimmed(&raw.environment.name, "nombre del entorno")?;

        let mut seen = HashSet::new();
        let mut factors = Vec::with_capacity(raw.factor.len());
        for factor in raw.factor {
            validate_factor_name(&factor.name)?;
            validate_nonempty_trimmed(&factor.value, "valor ambiental")?;
            if !seen.insert(factor.name.clone()) {
                return Err(format!("el entorno repite el factor '{}'", factor.name));
            }
            factors.push(EnvironmentFactorValue {
                name: factor.name,
                value: factor.value,
            });
        }

        Ok(Self {
            name: raw.environment.name,
            factors,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractStatus {
    Satisfied,
    Violated,
    OutOfScope,
    Invalid,
}

impl ContractStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Violated => "violated",
            Self::OutOfScope => "out_of_scope",
            Self::Invalid => "invalid",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Satisfied => 0,
            Self::Violated => 1,
            Self::OutOfScope => 2,
            Self::Invalid => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractEvaluation {
    status: ContractStatus,
    environment_name: String,
    reason: String,
    treated_factors: Vec<String>,
    matched_mce: Option<Vec<String>>,
}

impl ContractEvaluation {
    fn new(
        status: ContractStatus,
        environment_name: &str,
        reason: String,
        treated_factors: Vec<String>,
        matched_mce: Option<Vec<String>>,
    ) -> Self {
        Self {
            status,
            environment_name: environment_name.to_string(),
            reason,
            treated_factors,
            matched_mce,
        }
    }

    pub fn status(&self) -> ContractStatus {
        self.status
    }

    pub fn render_human(&self) -> String {
        let treated = if self.treated_factors.is_empty() {
            "ninguno".to_string()
        } else {
            self.treated_factors.join(", ")
        };
        let matched = self
            .matched_mce
            .as_ref()
            .map(|set| set.join(", "))
            .unwrap_or_else(|| "ninguno".to_string());

        format!(
            "Entorno: {}\nEstado: {}\nRazón: {}\nFactores treatment: {}\nMCE coincidente: {}\n",
            self.environment_name,
            self.status.as_str(),
            self.reason,
            treated,
            matched
        )
    }

    pub fn render_json(&self) -> String {
        let treated = self
            .treated_factors
            .iter()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .collect::<Vec<_>>()
            .join(",");
        let matched = self
            .matched_mce
            .as_ref()
            .map(|set| {
                let values = set
                    .iter()
                    .map(|value| format!("\"{}\"", json_escape(value)))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("[{}]", values)
            })
            .unwrap_or_else(|| "null".to_string());

        format!(
            "{{\"environment\":\"{}\",\"status\":\"{}\",\"reason\":\"{}\",\"treated_factors\":[{}],\"matched_mce\":{},\"scope\":\"{}\",\"claim\":\"{}\"}}\n",
            json_escape(&self.environment_name),
            self.status.as_str(),
            json_escape(&self.reason),
            treated,
            matched,
            CONTRACT_SCOPE,
            CONTRACT_CLAIM
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContractDocument {
    contract: RawContractHeader,
    evidence: RawContractEvidence,
    #[serde(default)]
    factor: Vec<RawContractFactor>,
    #[serde(default)]
    minimal_set: Vec<RawMinimalSet>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContractHeader {
    name: String,
    schema_version: u32,
    policy: String,
    source_causal_name: String,
    scope: String,
    claim: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContractEvidence {
    causal_status: String,
    confirmations: usize,
    target_trace_status: String,
    #[serde(default)]
    first_observed_divergence: Option<String>,
    #[serde(default)]
    absorption_boundaries: Vec<String>,
    #[serde(default)]
    essential_factors: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContractFactor {
    name: String,
    baseline: String,
    treatment: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMinimalSet {
    factors: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnvironmentDocument {
    environment: RawEnvironmentHeader,
    #[serde(default)]
    factor: Vec<RawEnvironmentFactor>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnvironmentHeader {
    name: String,
    schema_version: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnvironmentFactor {
    name: String,
    value: String,
}

fn essential_factors(factors: &[ContractFactor], sets: &[Vec<String>]) -> Vec<String> {
    if sets.is_empty() {
        return Vec::new();
    }

    factors
        .iter()
        .filter(|factor| sets.iter().all(|set| set.contains(&factor.name)))
        .map(|factor| factor.name.clone())
        .collect()
}

fn validate_factor_name(name: &str) -> Result<(), String> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err("el nombre de un factor no puede estar vacío".into());
    };
    if !first.is_ascii_lowercase() {
        return Err(format!(
            "el nombre de factor '{}' debe comenzar con una letra minúscula ASCII",
            name
        ));
    }
    if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_') {
        return Err(format!(
            "el nombre de factor '{}' solo puede usar letras minúsculas ASCII, dígitos y '_'",
            name
        ));
    }
    Ok(())
}

fn validate_nonempty_trimmed(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty() || value.trim() != value {
        return Err(format!(
            "{} no puede estar vacío ni tener espacios externos",
            label
        ));
    }
    Ok(())
}

fn render_toml_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", toml_escape(value)))
            .collect::<Vec<_>>()
            .join(", ")
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
    use crate::causal::{minimize_environment, CausalSpec, CausalStatus, MinimizeOptions};
    use crate::propagation::TraceSpec;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    fn temp_dir(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "envmorph-contract-test-{}-{}-{}",
            std::process::id(),
            id,
            name
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn sample_contract() -> EnvironmentalContract {
        EnvironmentalContract::from_toml_str(
            r#"
[contract]
name = "sample"
schema_version = 1
policy = "preserve_reference_behavior"
source_causal_name = "causal-sample"
scope = "declared_contrasts_only"
claim = "evidence_bounded_contract_not_universal_portability"

[evidence]
causal_status = "minimal"
confirmations = 2
target_trace_status = "persistent"
first_observed_divergence = "result"
absorption_boundaries = []
essential_factors = ["locale"]

[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"

[[factor]]
name = "timezone"
baseline = "UTC"
treatment = "America/Lima"

[[minimal_set]]
factors = ["locale"]
"#,
        )
        .unwrap()
    }

    fn environment(input: &str) -> EnvironmentSpec {
        EnvironmentSpec::from_toml_str(input).unwrap()
    }

    #[test]
    fn contract_roundtrip_is_stable() {
        let contract = sample_contract();
        let text = contract.render_toml();
        let parsed = EnvironmentalContract::from_toml_str(&text).unwrap();
        assert_eq!(parsed, contract);
        assert_eq!(parsed.render_toml(), text);
    }

    #[test]
    fn baseline_satisfies_contract() {
        let contract = sample_contract();
        let env = environment(
            r#"
[environment]
name = "baseline"
schema_version = 1
[[factor]]
name = "locale"
value = "C"
[[factor]]
name = "timezone"
value = "UTC"
"#,
        );
        assert_eq!(contract.evaluate(&env).status(), ContractStatus::Satisfied);
    }

    #[test]
    fn exact_mce_violates_contract() {
        let contract = sample_contract();
        let env = environment(
            r#"
[environment]
name = "mce"
schema_version = 1
[[factor]]
name = "locale"
value = "POSIX"
[[factor]]
name = "timezone"
value = "UTC"
"#,
        );
        assert_eq!(contract.evaluate(&env).status(), ContractStatus::Violated);
    }

    #[test]
    fn untested_partial_combination_is_out_of_scope() {
        let contract = sample_contract();
        let env = environment(
            r#"
[environment]
name = "partial"
schema_version = 1
[[factor]]
name = "locale"
value = "C"
[[factor]]
name = "timezone"
value = "America/Lima"
"#,
        );
        assert_eq!(contract.evaluate(&env).status(), ContractStatus::OutOfScope);
    }

    #[test]
    fn unseen_value_is_out_of_scope() {
        let contract = sample_contract();
        let env = environment(
            r#"
[environment]
name = "unseen"
schema_version = 1
[[factor]]
name = "locale"
value = "en_US.UTF-8"
[[factor]]
name = "timezone"
value = "UTC"
"#,
        );
        assert_eq!(contract.evaluate(&env).status(), ContractStatus::OutOfScope);
    }

    #[test]
    fn missing_factor_is_invalid() {
        let contract = sample_contract();
        let env = environment(
            r#"
[environment]
name = "missing"
schema_version = 1
[[factor]]
name = "locale"
value = "C"
"#,
        );
        assert_eq!(contract.evaluate(&env).status(), ContractStatus::Invalid);
    }

    #[test]
    fn duplicate_environment_factor_is_rejected() {
        let result = EnvironmentSpec::from_toml_str(
            r#"
[environment]
name = "duplicate"
schema_version = 1
[[factor]]
name = "locale"
value = "C"
[[factor]]
name = "locale"
value = "POSIX"
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn contract_rejects_inconsistent_essential_factors() {
        let result = EnvironmentalContract::from_toml_str(
            r#"
[contract]
name = "bad"
schema_version = 1
policy = "preserve_reference_behavior"
source_causal_name = "causal"
scope = "declared_contrasts_only"
claim = "evidence_bounded_contract_not_universal_portability"
[evidence]
causal_status = "minimal"
confirmations = 1
target_trace_status = "persistent"
absorption_boundaries = []
essential_factors = []
[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"
[[minimal_set]]
factors = ["locale"]
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn no_effect_full_intervention_is_satisfied() {
        let contract = EnvironmentalContract::from_toml_str(
            r#"
[contract]
name = "stable"
schema_version = 1
policy = "preserve_reference_behavior"
source_causal_name = "causal-stable"
scope = "declared_contrasts_only"
claim = "evidence_bounded_contract_not_universal_portability"
[evidence]
causal_status = "no_observable_effect"
confirmations = 1
target_trace_status = "stable"
absorption_boundaries = []
essential_factors = []
[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"
"#,
        )
        .unwrap();
        let env = environment(
            r#"
[environment]
name = "full"
schema_version = 1
[[factor]]
name = "locale"
value = "POSIX"
"#,
        );
        assert_eq!(contract.evaluate(&env).status(), ContractStatus::Satisfied);
    }

    #[test]
    fn derived_contract_consumes_real_f5_result() {
        let workdir = temp_dir("derive-workdir");
        fs::write(
            workdir.join("workflow.sh"),
            "#!/bin/sh\nmkdir -p output\nif [ \"${LC_ALL:-}\" = POSIX ]; then printf 'changed\\n'; else printf 'same\\n'; fi > output/result.txt\n",
        )
        .unwrap();

        let spec = CausalSpec::from_toml_str(
            r#"
[causal]
name = "derive"
schema_version = 1
confirmations = 1
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
        let trace = TraceSpec::from_tsv_str(
            "label\toracle\tleft_artifact\tright_artifact\toptions\n\
             result\tbyte\tworkspace/output/result.txt\tworkspace/output/result.txt\t-\n",
        )
        .unwrap();
        let root = temp_dir("derive-output").join("mce");
        let report = minimize_environment(
            &spec,
            &trace,
            &MinimizeOptions {
                output_dir: root,
                workdir,
                command: vec!["sh".into(), "workflow.sh".into()],
            },
        )
        .unwrap();
        assert_eq!(report.status(), CausalStatus::Minimal);
        let contract = EnvironmentalContract::derive(&spec, &report).unwrap();
        assert_eq!(
            contract.evidence().minimal_sets(),
            &[vec!["locale".to_string()]]
        );
    }
}
