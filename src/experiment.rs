use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactorValue(String);

impl FactorValue {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.is_empty() {
            return Err("el valor de un factor no puede estar vacío".into());
        }
        if value.trim() != value {
            return Err("el valor de un factor no puede tener espacios externos".into());
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FactorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentalFactor {
    name: String,
    values: Vec<FactorValue>,
}

impl EnvironmentalFactor {
    pub fn new(name: impl Into<String>, values: Vec<FactorValue>) -> Result<Self, String> {
        let name = name.into();
        validate_factor_name(&name)?;
        if values.len() < 2 {
            return Err(format!(
                "el factor '{}' debe declarar al menos dos valores",
                name
            ));
        }

        let mut seen = HashSet::new();
        for value in &values {
            if !seen.insert(value.as_str()) {
                return Err(format!(
                    "el factor '{}' contiene el valor duplicado '{}'",
                    name, value
                ));
            }
        }

        Ok(Self { name, values })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> &[FactorValue] {
        &self.values
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentVariant {
    id: String,
    assignments: Vec<(String, FactorValue)>,
}

impl EnvironmentVariant {
    pub fn new(index: usize, assignments: Vec<(String, FactorValue)>) -> Result<Self, String> {
        if index == 0 {
            return Err("el índice de una variante debe empezar en 1".into());
        }
        if assignments.is_empty() {
            return Err("una variante debe contener al menos una asignación".into());
        }

        let mut seen = HashSet::new();
        for (name, _) in &assignments {
            validate_factor_name(name)?;
            if !seen.insert(name.as_str()) {
                return Err(format!("la variante repite el factor '{}'", name));
            }
        }

        Ok(Self {
            id: format!("variant-{index:04}"),
            assignments,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn assignments(&self) -> &[(String, FactorValue)] {
        &self.assignments
    }

    pub fn value(&self, factor_name: &str) -> Option<&FactorValue> {
        self.assignments
            .iter()
            .find(|(name, _)| name == factor_name)
            .map(|(_, value)| value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentSpec {
    name: String,
    schema_version: u32,
    factors: Vec<EnvironmentalFactor>,
}

impl ExperimentSpec {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        let raw: RawExperimentDocument =
            toml::from_str(input).map_err(|e| format!("TOML experimental inválido: {e}"))?;
        Self::from_raw(raw)
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let input = fs::read_to_string(path)
            .map_err(|e| format!("no se pudo leer '{}': {e}", path.display()))?;
        Self::from_toml_str(&input)
    }

    fn from_raw(raw: RawExperimentDocument) -> Result<Self, String> {
        let name = raw.experiment.name;
        if name.is_empty() || name.trim() != name {
            return Err(
                "el nombre del experimento no puede estar vacío ni tener espacios externos".into(),
            );
        }
        if raw.experiment.schema_version != 1 {
            return Err(format!(
                "versión de esquema experimental no soportada: {}",
                raw.experiment.schema_version
            ));
        }
        if raw.factor.is_empty() {
            return Err("el experimento debe declarar al menos un factor".into());
        }

        let mut factors = Vec::with_capacity(raw.factor.len());
        let mut names = HashSet::new();
        for factor in raw.factor {
            if !names.insert(factor.name.clone()) {
                return Err(format!("el experimento repite el factor '{}'", factor.name));
            }
            let values = factor
                .values
                .into_iter()
                .map(FactorValue::new)
                .collect::<Result<Vec<_>, _>>()?;
            factors.push(EnvironmentalFactor::new(factor.name, values)?);
        }

        Ok(Self {
            name,
            schema_version: raw.experiment.schema_version,
            factors,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn factors(&self) -> &[EnvironmentalFactor] {
        &self.factors
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawExperimentDocument {
    experiment: RawExperiment,
    #[serde(default)]
    factor: Vec<RawFactor>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawExperiment {
    name: String,
    schema_version: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFactor {
    name: String,
    values: Vec<String>,
}

pub const MAX_PLAN_VARIANTS: usize = 4096;

pub fn plan_variants(spec: &ExperimentSpec) -> Result<Vec<EnvironmentVariant>, String> {
    let count = variant_count(spec)?;
    let mut variants = Vec::with_capacity(count);
    let mut assignments = Vec::with_capacity(spec.factors().len());
    expand_variants(spec.factors(), 0, &mut assignments, &mut variants)?;
    Ok(variants)
}

pub fn render_plan(spec: &ExperimentSpec, variants: &[EnvironmentVariant]) -> String {
    let mut out = String::new();
    out.push_str(&format!("Experimento: {}\n", spec.name()));
    out.push_str(&format!("Versión de esquema: {}\n", spec.schema_version()));
    out.push_str(&format!("Factores: {}\n", spec.factors().len()));
    out.push_str(&format!("Variantes: {}\n\n", variants.len()));

    for variant in variants {
        out.push_str(variant.id());
        for (name, value) in variant.assignments() {
            out.push_str(&format!("  {}={}", name, value));
        }
        out.push('\n');
    }

    out
}

fn variant_count(spec: &ExperimentSpec) -> Result<usize, String> {
    let mut count = 1usize;
    for factor in spec.factors() {
        count = count
            .checked_mul(factor.values().len())
            .ok_or_else(|| "la cantidad de variantes excede el límite numérico".to_string())?;
        if count > MAX_PLAN_VARIANTS {
            return Err(format!(
                "el plan contiene {} variantes y excede el límite de {}",
                count, MAX_PLAN_VARIANTS
            ));
        }
    }
    Ok(count)
}

fn expand_variants(
    factors: &[EnvironmentalFactor],
    factor_index: usize,
    assignments: &mut Vec<(String, FactorValue)>,
    variants: &mut Vec<EnvironmentVariant>,
) -> Result<(), String> {
    if factor_index == factors.len() {
        variants.push(EnvironmentVariant::new(
            variants.len() + 1,
            assignments.clone(),
        )?);
        return Ok(());
    }

    let factor = &factors[factor_index];
    for value in factor.values() {
        assignments.push((factor.name().to_string(), value.clone()));
        expand_variants(factors, factor_index + 1, assignments, variants)?;
        assignments.pop();
    }

    Ok(())
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

    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(format!(
            "el nombre de factor '{}' solo puede usar letras minúsculas ASCII, dígitos y '_'",
            name
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(text: &str) -> FactorValue {
        FactorValue::new(text).unwrap()
    }

    #[test]
    fn factor_value_rejects_empty_and_outer_whitespace() {
        assert!(FactorValue::new("").is_err());
        assert!(FactorValue::new(" C").is_err());
        assert!(FactorValue::new("C ").is_err());
        assert_eq!(FactorValue::new("C").unwrap().as_str(), "C");
    }

    #[test]
    fn environmental_factor_preserves_declared_order() {
        let factor =
            EnvironmentalFactor::new("locale", vec![value("C"), value("en_US.UTF-8")]).unwrap();
        assert_eq!(factor.name(), "locale");
        assert_eq!(factor.values()[0].as_str(), "C");
        assert_eq!(factor.values()[1].as_str(), "en_US.UTF-8");
    }

    #[test]
    fn environmental_factor_rejects_invalid_name() {
        assert!(EnvironmentalFactor::new("Locale", vec![value("C"), value("POSIX")]).is_err());
        assert!(EnvironmentalFactor::new("awk-impl", vec![value("mawk"), value("gawk")]).is_err());
    }

    #[test]
    fn environmental_factor_rejects_duplicate_values() {
        assert!(EnvironmentalFactor::new("timezone", vec![value("UTC"), value("UTC")]).is_err());
    }

    #[test]
    fn environment_variant_has_stable_id_and_lookup() {
        let variant = EnvironmentVariant::new(
            7,
            vec![
                ("locale".into(), value("C")),
                ("timezone".into(), value("UTC")),
            ],
        )
        .unwrap();

        assert_eq!(variant.id(), "variant-0007");
        assert_eq!(variant.assignments()[0].0, "locale");
        assert_eq!(variant.value("timezone").unwrap().as_str(), "UTC");
        assert!(variant.value("awk_implementation").is_none());
    }

    #[test]
    fn environment_variant_rejects_duplicate_factors() {
        let result = EnvironmentVariant::new(
            1,
            vec![
                ("locale".into(), value("C")),
                ("locale".into(), value("POSIX")),
            ],
        );
        assert!(result.is_err());
    }

    #[test]
    fn experiment_spec_parses_valid_toml() {
        let spec = ExperimentSpec::from_toml_str(
            r#"
[experiment]
name = "smoke"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "en_US.UTF-8"]

[[factor]]
name = "timezone"
values = ["UTC", "America/Lima"]
"#,
        )
        .unwrap();

        assert_eq!(spec.name(), "smoke");
        assert_eq!(spec.schema_version(), 1);
        assert_eq!(spec.factors().len(), 2);
        assert_eq!(spec.factors()[0].name(), "locale");
    }

    #[test]
    fn experiment_spec_rejects_duplicate_factor_names() {
        let input = r#"
[experiment]
name = "duplicates"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "POSIX"]

[[factor]]
name = "locale"
values = ["C", "en_US.UTF-8"]
"#;
        assert!(ExperimentSpec::from_toml_str(input).is_err());
    }

    #[test]
    fn experiment_spec_rejects_unsupported_schema_version() {
        let input = r#"
[experiment]
name = "future"
schema_version = 2

[[factor]]
name = "locale"
values = ["C", "POSIX"]
"#;
        assert!(ExperimentSpec::from_toml_str(input).is_err());
    }

    #[test]
    fn experiment_spec_rejects_unknown_fields() {
        let input = r#"
[experiment]
name = "typo"
schema_version = 1
unexpected = true

[[factor]]
name = "locale"
values = ["C", "POSIX"]
"#;
        assert!(ExperimentSpec::from_toml_str(input).is_err());
    }

    #[test]
    fn experiment_spec_rejects_malformed_toml() {
        assert!(ExperimentSpec::from_toml_str("[experiment\nname = 1").is_err());
    }

    #[test]
    fn plan_variants_uses_cartesian_product() {
        let spec = ExperimentSpec::from_toml_str(
            r#"
[experiment]
name = "cartesian"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "POSIX"]

[[factor]]
name = "timezone"
values = ["UTC", "America/Lima"]
"#,
        )
        .unwrap();

        let variants = plan_variants(&spec).unwrap();
        assert_eq!(variants.len(), 4);
        assert_eq!(variants[0].id(), "variant-0001");
        assert_eq!(variants[0].value("locale").unwrap().as_str(), "C");
        assert_eq!(variants[0].value("timezone").unwrap().as_str(), "UTC");
        assert_eq!(variants[3].value("locale").unwrap().as_str(), "POSIX");
        assert_eq!(
            variants[3].value("timezone").unwrap().as_str(),
            "America/Lima"
        );
    }

    #[test]
    fn plan_variants_preserves_factor_and_value_order() {
        let spec = ExperimentSpec::from_toml_str(
            r#"
[experiment]
name = "order"
schema_version = 1

[[factor]]
name = "locale"
values = ["A", "B"]

[[factor]]
name = "timezone"
values = ["X", "Y"]
"#,
        )
        .unwrap();

        let variants = plan_variants(&spec).unwrap();
        let observed: Vec<String> = variants
            .iter()
            .map(|variant| {
                format!(
                    "{}:{}",
                    variant.value("locale").unwrap(),
                    variant.value("timezone").unwrap()
                )
            })
            .collect();

        assert_eq!(observed, vec!["A:X", "A:Y", "B:X", "B:Y"]);
    }

    #[test]
    fn plan_variants_rejects_excessive_matrix() {
        let factors = (0..13)
            .map(|index| {
                EnvironmentalFactor::new(format!("factor_{index}"), vec![value("a"), value("b")])
                    .unwrap()
            })
            .collect();

        let spec = ExperimentSpec {
            name: "large".into(),
            schema_version: 1,
            factors,
        };

        assert!(plan_variants(&spec).is_err());
    }

    #[test]
    fn render_plan_is_stable() {
        let spec = ExperimentSpec::from_toml_str(
            r#"
[experiment]
name = "stable"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "POSIX"]
"#,
        )
        .unwrap();

        let variants = plan_variants(&spec).unwrap();
        let rendered = render_plan(&spec, &variants);

        assert!(rendered.contains("Experimento: stable"));
        assert!(rendered.contains("Variantes: 2"));
        assert!(rendered.contains("variant-0001  locale=C"));
        assert!(rendered.contains("variant-0002  locale=POSIX"));
    }
}
