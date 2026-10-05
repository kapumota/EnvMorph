use crate::contract::{ContractStatus, EnvironmentSpec, EnvironmentalContract};

pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;
pub const MAX_ENVELOPE_FACTORS: usize = 10;
const ENVELOPE_SCOPE: &str = "declared_baseline_treatment_lattice";
const ENVELOPE_CLAIM: &str = "finite_observed_contrast_envelope_not_universal_portability";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeStatus {
    Complete,
    Partial,
}

impl EnvelopeStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeFactor {
    name: String,
    baseline: String,
    treatment: String,
}

impl EnvelopeFactor {
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
pub struct EnvelopeAssignment {
    factor: String,
    value: String,
    mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeCell {
    id: String,
    mask: usize,
    status: ContractStatus,
    assignment: Vec<EnvelopeAssignment>,
}

impl EnvelopeCell {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn status(&self) -> ContractStatus {
        self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeBoundary {
    left: String,
    right: String,
    factor: String,
    left_status: ContractStatus,
    right_status: ContractStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortabilityEnvelope {
    contract_name: String,
    status: EnvelopeStatus,
    factors: Vec<EnvelopeFactor>,
    cells: Vec<EnvelopeCell>,
    boundaries: Vec<EnvelopeBoundary>,
    satisfied_cells: usize,
    violated_cells: usize,
    out_of_scope_cells: usize,
}

impl PortabilityEnvelope {
    pub fn build(contract: &EnvironmentalContract) -> Result<Self, String> {
        let factor_count = contract.factors().len();
        if factor_count == 0 {
            return Err("el sobre de portabilidad requiere al menos un factor contractual".into());
        }
        if factor_count > MAX_ENVELOPE_FACTORS {
            return Err(format!(
                "F7 admite como máximo {} factores por sobre finito",
                MAX_ENVELOPE_FACTORS
            ));
        }

        let factors = contract
            .factors()
            .iter()
            .map(|factor| EnvelopeFactor {
                name: factor.name().to_string(),
                baseline: factor.baseline().to_string(),
                treatment: factor.treatment().to_string(),
            })
            .collect::<Vec<_>>();

        let total = 1usize << factor_count;
        let mut cells = Vec::with_capacity(total);
        let mut satisfied_cells = 0usize;
        let mut violated_cells = 0usize;
        let mut out_of_scope_cells = 0usize;

        for mask in 0..total {
            let environment = environment_for_mask(&factors, mask)?;
            let evaluation = contract.evaluate(&environment);
            let status = evaluation.status();

            match status {
                ContractStatus::Satisfied => satisfied_cells += 1,
                ContractStatus::Violated => violated_cells += 1,
                ContractStatus::OutOfScope => out_of_scope_cells += 1,
                ContractStatus::Invalid => {
                    return Err(format!(
                        "F6 produjo invalid para una celda completa generada por F7: máscara {}",
                        mask
                    ));
                }
            }

            let assignment = factors
                .iter()
                .enumerate()
                .map(|(index, factor)| {
                    let treated = mask & (1usize << index) != 0;
                    EnvelopeAssignment {
                        factor: factor.name.clone(),
                        value: if treated {
                            factor.treatment.clone()
                        } else {
                            factor.baseline.clone()
                        },
                        mode: if treated { "treatment" } else { "baseline" }.to_string(),
                    }
                })
                .collect::<Vec<_>>();

            cells.push(EnvelopeCell {
                id: format!("cell-{:04}", mask + 1),
                mask,
                status,
                assignment,
            });
        }

        let mut boundaries = Vec::new();
        for mask in 0..total {
            for factor_index in 0..factor_count {
                let neighbor = mask ^ (1usize << factor_index);
                if neighbor <= mask {
                    continue;
                }
                let left = &cells[mask];
                let right = &cells[neighbor];
                if left.status != right.status {
                    boundaries.push(EnvelopeBoundary {
                        left: left.id.clone(),
                        right: right.id.clone(),
                        factor: factors[factor_index].name.clone(),
                        left_status: left.status,
                        right_status: right.status,
                    });
                }
            }
        }

        let status = if out_of_scope_cells == 0 {
            EnvelopeStatus::Complete
        } else {
            EnvelopeStatus::Partial
        };

        Ok(Self {
            contract_name: contract.name().to_string(),
            status,
            factors,
            cells,
            boundaries,
            satisfied_cells,
            violated_cells,
            out_of_scope_cells,
        })
    }

    pub fn status(&self) -> EnvelopeStatus {
        self.status
    }

    pub fn cells(&self) -> &[EnvelopeCell] {
        &self.cells
    }

    pub fn boundaries(&self) -> &[EnvelopeBoundary] {
        &self.boundaries
    }

    pub fn satisfied_cells(&self) -> usize {
        self.satisfied_cells
    }

    pub fn violated_cells(&self) -> usize {
        self.violated_cells
    }

    pub fn out_of_scope_cells(&self) -> usize {
        self.out_of_scope_cells
    }

    pub fn decided_cells(&self) -> usize {
        self.satisfied_cells + self.violated_cells
    }

    pub fn render_human(&self) -> String {
        format!(
            "Sobre de portabilidad: {}\nEstado: {}\nFactores: {}\nCeldas totales: {}\nCeldas satisfied: {}\nCeldas violated: {}\nCeldas out_of_scope: {}\nCobertura decidida: {}/{}\nFronteras observadas: {}\nAlcance: lattice finito baseline/treatment declarado.\nAfirmación: no constituye portabilidad universal.\n",
            self.contract_name,
            self.status.as_str(),
            self.factors.len(),
            self.cells.len(),
            self.satisfied_cells,
            self.violated_cells,
            self.out_of_scope_cells,
            self.decided_cells(),
            self.cells.len(),
            self.boundaries.len(),
        )
    }

    pub fn render_json(&self) -> String {
        let factors = self
            .factors
            .iter()
            .map(|factor| {
                format!(
                    "{{\"name\":\"{}\",\"baseline\":\"{}\",\"treatment\":\"{}\"}}",
                    json_escape(&factor.name),
                    json_escape(&factor.baseline),
                    json_escape(&factor.treatment)
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let cells = self
            .cells
            .iter()
            .map(|cell| {
                let assignment = cell
                    .assignment
                    .iter()
                    .map(|value| {
                        format!(
                            "{{\"factor\":\"{}\",\"value\":\"{}\",\"mode\":\"{}\"}}",
                            json_escape(&value.factor),
                            json_escape(&value.value),
                            json_escape(&value.mode)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"id\":\"{}\",\"mask\":{},\"status\":\"{}\",\"assignment\":[{}]}}",
                    json_escape(&cell.id),
                    cell.mask,
                    cell.status.as_str(),
                    assignment
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let boundaries = self
            .boundaries
            .iter()
            .map(|boundary| {
                format!(
                    "{{\"left\":\"{}\",\"right\":\"{}\",\"factor\":\"{}\",\"left_status\":\"{}\",\"right_status\":\"{}\"}}",
                    json_escape(&boundary.left),
                    json_escape(&boundary.right),
                    json_escape(&boundary.factor),
                    boundary.left_status.as_str(),
                    boundary.right_status.as_str()
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"schema_version\":{},\"contract\":\"{}\",\"status\":\"{}\",\"scope\":\"{}\",\"claim\":\"{}\",\"factor_count\":{},\"total_cells\":{},\"satisfied_cells\":{},\"violated_cells\":{},\"out_of_scope_cells\":{},\"decided_cells\":{},\"coverage\":{{\"decided\":{},\"total\":{}}},\"boundary_count\":{},\"factors\":[{}],\"cells\":[{}],\"boundaries\":[{}]}}\n",
            ENVELOPE_SCHEMA_VERSION,
            json_escape(&self.contract_name),
            self.status.as_str(),
            ENVELOPE_SCOPE,
            ENVELOPE_CLAIM,
            self.factors.len(),
            self.cells.len(),
            self.satisfied_cells,
            self.violated_cells,
            self.out_of_scope_cells,
            self.decided_cells(),
            self.decided_cells(),
            self.cells.len(),
            self.boundaries.len(),
            factors,
            cells,
            boundaries
        )
    }
}

fn environment_for_mask(
    factors: &[EnvelopeFactor],
    mask: usize,
) -> Result<EnvironmentSpec, String> {
    let mut text = String::new();
    text.push_str("[environment]\n");
    text.push_str(&format!("name = \"cell-{:04}\"\n", mask + 1));
    text.push_str("schema_version = 1\n");

    for (index, factor) in factors.iter().enumerate() {
        let value = if mask & (1usize << index) != 0 {
            &factor.treatment
        } else {
            &factor.baseline
        };
        text.push_str("\n[[factor]]\n");
        text.push_str(&format!("name = \"{}\"\n", toml_escape(&factor.name)));
        text.push_str(&format!("value = \"{}\"\n", toml_escape(value)));
    }

    EnvironmentSpec::from_toml_str(&text)
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
    use crate::contract::EnvironmentalContract;

    fn contract_two_factors() -> EnvironmentalContract {
        EnvironmentalContract::from_toml_str(
            r#"
[contract]
name = "test-contract"
schema_version = 1
policy = "preserve_reference_behavior"
source_causal_name = "test"
scope = "declared_contrasts_only"
claim = "evidence_bounded_contract_not_universal_portability"

[evidence]
causal_status = "minimal"
confirmations = 1
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

    fn contract_one_factor() -> EnvironmentalContract {
        EnvironmentalContract::from_toml_str(
            r#"
[contract]
name = "single-contract"
schema_version = 1
policy = "preserve_reference_behavior"
source_causal_name = "single"
scope = "declared_contrasts_only"
claim = "evidence_bounded_contract_not_universal_portability"

[evidence]
causal_status = "minimal"
confirmations = 1
target_trace_status = "persistent"
first_observed_divergence = "result"
absorption_boundaries = []
essential_factors = ["locale"]

[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"

[[minimal_set]]
factors = ["locale"]
"#,
        )
        .unwrap()
    }

    #[test]
    fn envelope_enumerates_complete_binary_lattice() {
        let envelope = PortabilityEnvelope::build(&contract_two_factors()).unwrap();
        assert_eq!(envelope.cells().len(), 4);
    }

    #[test]
    fn envelope_preserves_contract_classification() {
        let envelope = PortabilityEnvelope::build(&contract_two_factors()).unwrap();
        assert_eq!(envelope.satisfied_cells(), 1);
        assert_eq!(envelope.violated_cells(), 2);
        assert_eq!(envelope.out_of_scope_cells(), 1);
        assert_eq!(envelope.status(), EnvelopeStatus::Partial);
    }

    #[test]
    fn envelope_records_neighbor_boundaries() {
        let envelope = PortabilityEnvelope::build(&contract_two_factors()).unwrap();
        assert_eq!(envelope.boundaries().len(), 3);
    }

    #[test]
    fn envelope_is_complete_when_every_declared_cell_is_decided() {
        let envelope = PortabilityEnvelope::build(&contract_one_factor()).unwrap();
        assert_eq!(envelope.status(), EnvelopeStatus::Complete);
        assert_eq!(envelope.cells().len(), 2);
        assert_eq!(envelope.out_of_scope_cells(), 0);
    }

    #[test]
    fn envelope_json_is_deterministic_and_bounded() {
        let envelope = PortabilityEnvelope::build(&contract_two_factors()).unwrap();
        assert_eq!(envelope.render_json(), envelope.render_json());
        assert!(envelope
            .render_json()
            .contains("finite_observed_contrast_envelope_not_universal_portability"));
    }
}
