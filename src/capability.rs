use crate::experiment::{EnvironmentVariant, ExperimentSpec, FactorValue};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityStatus {
    Available,
    Unavailable,
}

impl CapabilityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Available => "disponible",
            Self::Unavailable => "no_disponible",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentCapability {
    factor: String,
    value: FactorValue,
    status: CapabilityStatus,
    resolved: Option<String>,
    detail: Option<String>,
}

impl EnvironmentCapability {
    fn available(
        factor: &str,
        value: &FactorValue,
        resolved: Option<String>,
        detail: Option<String>,
    ) -> Self {
        Self {
            factor: factor.to_string(),
            value: value.clone(),
            status: CapabilityStatus::Available,
            resolved,
            detail,
        }
    }

    fn unavailable(factor: &str, value: &FactorValue, detail: impl Into<String>) -> Self {
        Self {
            factor: factor.to_string(),
            value: value.clone(),
            status: CapabilityStatus::Unavailable,
            resolved: None,
            detail: Some(detail.into()),
        }
    }

    pub fn factor(&self) -> &str {
        &self.factor
    }

    pub fn value(&self) -> &FactorValue {
        &self.value
    }

    pub fn status(&self) -> &CapabilityStatus {
        &self.status
    }

    pub fn resolved(&self) -> Option<&str> {
        self.resolved.as_deref()
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn is_available(&self) -> bool {
        self.status == CapabilityStatus::Available
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTool {
    requested: String,
    path: PathBuf,
    version: Option<String>,
}

impl ResolvedTool {
    pub fn requested(&self) -> &str {
        &self.requested
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEnvironment {
    variables: Vec<(String, String)>,
    path_prefixes: Vec<PathBuf>,
    tools: Vec<ResolvedTool>,
    capabilities: Vec<EnvironmentCapability>,
}

impl ResolvedEnvironment {
    pub fn variables(&self) -> &[(String, String)] {
        &self.variables
    }

    pub fn path_prefixes(&self) -> &[PathBuf] {
        &self.path_prefixes
    }

    pub fn tools(&self) -> &[ResolvedTool] {
        &self.tools
    }

    pub fn capabilities(&self) -> &[EnvironmentCapability] {
        &self.capabilities
    }

    pub fn is_available(&self) -> bool {
        self.capabilities
            .iter()
            .all(EnvironmentCapability::is_available)
    }
}

pub fn probe_spec(spec: &ExperimentSpec) -> Result<Vec<EnvironmentCapability>, String> {
    let mut capabilities = Vec::new();

    for factor in spec.factors() {
        for value in factor.values() {
            capabilities.push(probe_factor(factor.name(), value)?);
        }
    }

    Ok(capabilities)
}

pub fn resolve_variant(variant: &EnvironmentVariant) -> Result<ResolvedEnvironment, String> {
    let mut variables = Vec::new();
    let path_prefixes = Vec::new();
    let mut tools = Vec::new();
    let mut capabilities = Vec::new();

    for (factor, value) in variant.assignments() {
        let capability = probe_factor(factor, value)?;

        match factor.as_str() {
            "locale" => {
                variables.push(("LC_ALL".to_string(), value.as_str().to_string()));
                variables.push(("LANG".to_string(), value.as_str().to_string()));
            }
            "timezone" => {
                variables.push(("TZ".to_string(), value.as_str().to_string()));
            }
            "awk_implementation" => {
                if capability.is_available() {
                    if let Some(path) = capability.resolved() {
                        tools.push(ResolvedTool {
                            requested: value.as_str().to_string(),
                            path: PathBuf::from(path),
                            version: tool_version(value.as_str(), Path::new(path)),
                        });
                    }
                }
            }
            _ => {
                return Err(format!(
                    "factor ambiental no soportado por F2.1: {}",
                    factor
                ));
            }
        }

        capabilities.push(capability);
    }

    Ok(ResolvedEnvironment {
        variables,
        path_prefixes,
        tools,
        capabilities,
    })
}

pub fn render_capabilities(
    spec: &ExperimentSpec,
    capabilities: &[EnvironmentCapability],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("Experimento: {}\n", spec.name()));
    out.push_str(&format!("Capacidades: {}\n\n", capabilities.len()));

    for capability in capabilities {
        out.push_str(capability.factor());
        out.push('=');
        out.push_str(capability.value().as_str());
        out.push_str("  ");
        out.push_str(capability.status().as_str());

        if let Some(resolved) = capability.resolved() {
            out.push_str("  resuelto=");
            out.push_str(resolved);
        }

        if let Some(detail) = capability.detail() {
            out.push_str("  detalle=");
            out.push_str(&detail.replace('\n', " "));
        }

        out.push('\n');
    }

    out
}

fn probe_factor(factor: &str, value: &FactorValue) -> Result<EnvironmentCapability, String> {
    match factor {
        "locale" => Ok(probe_locale(value)),
        "timezone" => Ok(probe_timezone(value)),
        "awk_implementation" => Ok(probe_awk(value)),
        _ => Err(format!(
            "factor ambiental no soportado por F2.1: {}",
            factor
        )),
    }
}

fn probe_locale(value: &FactorValue) -> EnvironmentCapability {
    let output = match Command::new("locale").arg("-a").output() {
        Ok(output) => output,
        Err(error) => {
            return EnvironmentCapability::unavailable(
                "locale",
                value,
                format!("no se pudo ejecutar locale -a: {}", error),
            );
        }
    };

    if !output.status.success() {
        return EnvironmentCapability::unavailable(
            "locale",
            value,
            format!("locale -a terminó con estado {}", output.status),
        );
    }

    let requested = normalize_locale(value.as_str());
    let stdout = String::from_utf8_lossy(&output.stdout);

    for available in stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if normalize_locale(available) == requested {
            return EnvironmentCapability::available(
                "locale",
                value,
                Some(available.to_string()),
                None,
            );
        }
    }

    EnvironmentCapability::unavailable(
        "locale",
        value,
        "el locale solicitado no aparece en locale -a",
    )
}

fn probe_timezone(value: &FactorValue) -> EnvironmentCapability {
    if !valid_timezone_id(value.as_str()) {
        return EnvironmentCapability::unavailable(
            "timezone",
            value,
            "identificador de zona horaria no válido",
        );
    }

    let tzdir = env::var_os("TZDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/share/zoneinfo"));

    let candidate = tzdir.join(value.as_str());

    match fs::metadata(&candidate) {
        Ok(metadata) if metadata.is_file() => EnvironmentCapability::available(
            "timezone",
            value,
            Some(candidate.display().to_string()),
            None,
        ),
        Ok(_) => EnvironmentCapability::unavailable(
            "timezone",
            value,
            "la ruta de zona horaria no es un archivo",
        ),
        Err(error) => EnvironmentCapability::unavailable(
            "timezone",
            value,
            format!("zona horaria no disponible: {}", error),
        ),
    }
}

fn probe_awk(value: &FactorValue) -> EnvironmentCapability {
    if !valid_tool_name(value.as_str()) {
        return EnvironmentCapability::unavailable(
            "awk_implementation",
            value,
            "nombre de herramienta no válido",
        );
    }

    let Some(path) = resolve_executable(value.as_str()) else {
        return EnvironmentCapability::unavailable(
            "awk_implementation",
            value,
            "ejecutable no encontrado en PATH",
        );
    };

    let version = tool_version(value.as_str(), &path);

    EnvironmentCapability::available(
        "awk_implementation",
        value,
        Some(path.display().to_string()),
        version,
    )
}

fn normalize_locale(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn valid_timezone_id(value: &str) -> bool {
    let path = Path::new(value);

    if path.is_absolute() || value.is_empty() {
        return false;
    }

    path.components()
        .all(|component| matches!(component, Component::Normal(_)))
}

fn valid_tool_name(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('/')
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '+'))
}

fn resolve_executable(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for directory in env::split_paths(&path) {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn tool_version(name: &str, path: &Path) -> Option<String> {
    let args: &[&str] = if name == "mawk" {
        &["-W", "version"]
    } else {
        &["--version"]
    };

    let output = Command::new(path).args(args).output().ok()?;

    let text = if !output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stdout)
    } else {
        String::from_utf8_lossy(&output.stderr)
    };

    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experiment::{EnvironmentVariant, FactorValue};

    fn value(text: &str) -> FactorValue {
        FactorValue::new(text).unwrap()
    }

    #[test]
    fn normalize_locale_accepts_common_utf8_aliases() {
        assert_eq!(
            normalize_locale("en_US.UTF-8"),
            normalize_locale("en_US.utf8")
        );
    }

    #[test]
    fn timezone_identifier_rejects_absolute_path() {
        assert!(!valid_timezone_id("/usr/share/zoneinfo/UTC"));
    }

    #[test]
    fn timezone_identifier_rejects_parent_traversal() {
        assert!(!valid_timezone_id("../UTC"));
        assert!(!valid_timezone_id("America/../UTC"));
    }

    #[test]
    fn timezone_identifier_accepts_region_name() {
        assert!(valid_timezone_id("America/Lima"));
        assert!(valid_timezone_id("UTC"));
    }

    #[test]
    fn tool_name_rejects_paths() {
        assert!(!valid_tool_name("/usr/bin/gawk"));
        assert!(!valid_tool_name("../gawk"));
        assert!(valid_tool_name("gawk"));
        assert!(valid_tool_name("mawk"));
    }

    #[test]
    fn unknown_factor_is_rejected() {
        let variant =
            EnvironmentVariant::new(1, vec![("unknown_factor".to_string(), value("x"))]).unwrap();

        assert!(resolve_variant(&variant).is_err());
    }

    #[test]
    fn unavailable_tool_is_reported_without_failing_resolution() {
        let missing = value("envmorph-awk-that-does-not-exist");
        let capability = probe_awk(&missing);

        assert_eq!(capability.status(), &CapabilityStatus::Unavailable);
        assert!(!capability.is_available());
    }

    #[test]
    fn utc_timezone_is_resolved_on_unix_baseline() {
        let capability = probe_timezone(&value("UTC"));
        assert!(capability.is_available());
        assert!(capability.resolved().is_some());
    }

    #[test]
    fn resolved_environment_reports_unavailable_capability() {
        let variant = EnvironmentVariant::new(
            1,
            vec![(
                "awk_implementation".to_string(),
                value("envmorph-awk-that-does-not-exist"),
            )],
        )
        .unwrap();

        let resolved = resolve_variant(&variant).unwrap();
        assert!(!resolved.is_available());
        assert!(resolved.tools().is_empty());
        assert_eq!(resolved.path_prefixes().len(), 0);
    }
}
