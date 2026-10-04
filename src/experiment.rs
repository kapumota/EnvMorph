use std::collections::HashSet;
use std::fmt;

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
}
