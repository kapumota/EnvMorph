use crate::capability::{resolve_variant, ResolvedEnvironment};
use crate::experiment::{EnvironmentVariant, ExperimentSpec};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Unavailable,
    ExecutionSucceeded,
    ExecutionFailed,
}

impl ExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::ExecutionSucceeded => "execution_succeeded",
            Self::ExecutionFailed => "execution_failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExploreOptions {
    pub output_dir: PathBuf,
    pub workdir: PathBuf,
    pub command: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VariantExecution {
    variant_id: String,
    status: ExecutionStatus,
    exit_code: Option<i32>,
    duration_ms: u128,
    directory: PathBuf,
}

impl VariantExecution {
    pub fn variant_id(&self) -> &str {
        &self.variant_id
    }

    pub fn status(&self) -> &ExecutionStatus {
        &self.status
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn duration_ms(&self) -> u128 {
        self.duration_ms
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

#[derive(Debug, Clone)]
pub struct ExperimentBundle {
    output_dir: PathBuf,
    executions: Vec<VariantExecution>,
}

impl ExperimentBundle {
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    pub fn executions(&self) -> &[VariantExecution] {
        &self.executions
    }

    pub fn unavailable_count(&self) -> usize {
        self.executions
            .iter()
            .filter(|execution| execution.status == ExecutionStatus::Unavailable)
            .count()
    }

    pub fn succeeded_count(&self) -> usize {
        self.executions
            .iter()
            .filter(|execution| execution.status == ExecutionStatus::ExecutionSucceeded)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.executions
            .iter()
            .filter(|execution| execution.status == ExecutionStatus::ExecutionFailed)
            .count()
    }

    pub fn has_failures(&self) -> bool {
        self.failed_count() > 0
    }
}

pub fn explore(
    spec: &ExperimentSpec,
    variants: &[EnvironmentVariant],
    options: &ExploreOptions,
) -> Result<ExperimentBundle, String> {
    validate_options(options)?;

    if options.output_dir.exists() {
        return Err(format!(
            "el directorio de salida ya existe: {}",
            options.output_dir.display()
        ));
    }

    fs::create_dir_all(&options.output_dir).map_err(|error| {
        format!(
            "no se pudo crear '{}': {}",
            options.output_dir.display(),
            error
        )
    })?;

    let variants_root = options.output_dir.join("variants");
    fs::create_dir_all(&variants_root)
        .map_err(|error| format!("no se pudo crear variants/: {error}"))?;

    let mut executions = Vec::with_capacity(variants.len());

    for variant in variants {
        executions.push(execute_variant(
            variant,
            options,
            &variants_root.join(variant.id()),
        )?);
    }

    let bundle = ExperimentBundle {
        output_dir: options.output_dir.clone(),
        executions,
    };

    write_experiment_manifest(spec, variants, options, &bundle)?;
    write_hashes(&bundle)?;

    Ok(bundle)
}

pub fn render_explore(spec: &ExperimentSpec, bundle: &ExperimentBundle) -> String {
    format!(
        "Experimento: {}\nVariantes planificadas: {}\nEjecutadas correctamente: {}\nNo disponibles: {}\nFallidas: {}\nBundle: {}\n",
        spec.name(),
        bundle.executions().len(),
        bundle.succeeded_count(),
        bundle.unavailable_count(),
        bundle.failed_count(),
        bundle.output_dir().display(),
    )
}

fn execute_variant(
    variant: &EnvironmentVariant,
    options: &ExploreOptions,
    variant_dir: &Path,
) -> Result<VariantExecution, String> {
    fs::create_dir_all(variant_dir)
        .map_err(|error| format!("no se pudo crear '{}': {error}", variant_dir.display()))?;

    let resolved = resolve_variant(variant)?;
    let workspace = variant_dir.join("workspace");
    copy_workspace(&options.workdir, &workspace, &options.output_dir)?;

    let stdout_path = variant_dir.join("stdout.txt");
    let stderr_path = variant_dir.join("stderr.txt");

    if !resolved.is_available() {
        fs::write(&stdout_path, b"")
            .map_err(|error| format!("no se pudo crear stdout vacío: {error}"))?;
        fs::write(&stderr_path, b"")
            .map_err(|error| format!("no se pudo crear stderr vacío: {error}"))?;

        let execution = VariantExecution {
            variant_id: variant.id().to_string(),
            status: ExecutionStatus::Unavailable,
            exit_code: None,
            duration_ms: 0,
            directory: variant_dir.to_path_buf(),
        };

        write_variant_manifest(variant, &resolved, options, &execution)?;
        return Ok(execution);
    }

    let shim_dir = variant_dir.join("shim");
    let path_value = prepare_path(&resolved, &shim_dir)?;

    let stdout_file = fs::File::create(&stdout_path)
        .map_err(|error| format!("no se pudo crear '{}': {error}", stdout_path.display()))?;
    let stderr_file = fs::File::create(&stderr_path)
        .map_err(|error| format!("no se pudo crear '{}': {error}", stderr_path.display()))?;

    let mut command = Command::new(&options.command[0]);
    command
        .args(&options.command[1..])
        .current_dir(&workspace)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    for (name, value) in resolved.variables() {
        command.env(name, value);
    }

    if let Some(path_value) = path_value {
        command.env("PATH", path_value);
    }

    let start = Instant::now();
    let status = command.status().map_err(|error| {
        format!(
            "no se pudo ejecutar '{}' para {}: {}",
            options.command[0],
            variant.id(),
            error
        )
    })?;
    let duration_ms = start.elapsed().as_millis();

    let execution = VariantExecution {
        variant_id: variant.id().to_string(),
        status: if status.success() {
            ExecutionStatus::ExecutionSucceeded
        } else {
            ExecutionStatus::ExecutionFailed
        },
        exit_code: status.code(),
        duration_ms,
        directory: variant_dir.to_path_buf(),
    };

    write_variant_manifest(variant, &resolved, options, &execution)?;
    Ok(execution)
}

fn validate_options(options: &ExploreOptions) -> Result<(), String> {
    if options.command.is_empty() {
        return Err("explore requiere una orden después de --".into());
    }

    if !options.workdir.is_dir() {
        return Err(format!(
            "el directorio de trabajo no existe: {}",
            options.workdir.display()
        ));
    }

    Ok(())
}

fn copy_workspace(source: &Path, destination: &Path, output_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("no se pudo crear '{}': {error}", destination.display()))?;

    let output_abs = absolute_path(output_dir)?;

    copy_directory(source, destination, &output_abs)
}

fn copy_directory(source: &Path, destination: &Path, output_abs: &Path) -> Result<(), String> {
    for entry in fs::read_dir(source)
        .map_err(|error| format!("no se pudo leer '{}': {error}", source.display()))?
    {
        let entry = entry.map_err(|error| format!("no se pudo leer una entrada: {error}"))?;
        let source_path = entry.path();
        let name = entry.file_name();

        if matches!(
            name.to_str(),
            Some(".git") | Some("target") | Some(".envmorph")
        ) {
            continue;
        }

        if absolute_path(&source_path)? == output_abs {
            continue;
        }

        let destination_path = destination.join(&name);
        let metadata = fs::symlink_metadata(&source_path).map_err(|error| {
            format!(
                "no se pudo inspeccionar '{}': {error}",
                source_path.display()
            )
        })?;

        if metadata.file_type().is_symlink() {
            copy_symlink(&source_path, &destination_path)?;
        } else if metadata.is_dir() {
            fs::create_dir_all(&destination_path).map_err(|error| {
                format!("no se pudo crear '{}': {error}", destination_path.display())
            })?;
            copy_directory(&source_path, &destination_path, output_abs)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                format!(
                    "no se pudo copiar '{}' a '{}': {}",
                    source_path.display(),
                    destination_path.display(),
                    error
                )
            })?;
            fs::set_permissions(&destination_path, metadata.permissions()).map_err(|error| {
                format!(
                    "no se pudieron preservar permisos de '{}': {}",
                    destination_path.display(),
                    error
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(unix)]
fn copy_symlink(source: &Path, destination: &Path) -> Result<(), String> {
    use std::os::unix::fs::symlink;

    let target = fs::read_link(source)
        .map_err(|error| format!("no se pudo leer symlink '{}': {error}", source.display()))?;
    symlink(&target, destination).map_err(|error| {
        format!(
            "no se pudo copiar symlink '{}' a '{}': {}",
            source.display(),
            destination.display(),
            error
        )
    })
}

#[cfg(not(unix))]
fn copy_symlink(_source: &Path, _destination: &Path) -> Result<(), String> {
    Err("F2 requiere soporte Unix para preservar symlinks".into())
}

fn prepare_path(resolved: &ResolvedEnvironment, shim_dir: &Path) -> Result<Option<String>, String> {
    if resolved.tools().is_empty() {
        return Ok(None);
    }

    fs::create_dir_all(shim_dir)
        .map_err(|error| format!("no se pudo crear '{}': {error}", shim_dir.display()))?;

    for tool in resolved.tools() {
        if matches!(tool.requested(), "mawk" | "gawk") {
            create_awk_shim(&shim_dir.join("awk"), tool.path())?;
        }
    }

    let inherited = env::var("PATH").unwrap_or_default();
    let value = if inherited.is_empty() {
        shim_dir.display().to_string()
    } else {
        format!("{}:{}", shim_dir.display(), inherited)
    };

    Ok(Some(value))
}

#[cfg(unix)]
fn create_awk_shim(path: &Path, target: &Path) -> Result<(), String> {
    use std::os::unix::fs::symlink;

    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("no se pudo reemplazar '{}': {error}", path.display()))?;
    }

    symlink(target, path).map_err(|error| {
        format!(
            "no se pudo crear shim AWK '{}' hacia '{}': {}",
            path.display(),
            target.display(),
            error
        )
    })
}

#[cfg(not(unix))]
fn create_awk_shim(_path: &Path, _target: &Path) -> Result<(), String> {
    Err("F2 requiere soporte Unix para crear el shim AWK".into())
}

fn write_variant_manifest(
    variant: &EnvironmentVariant,
    resolved: &ResolvedEnvironment,
    options: &ExploreOptions,
    execution: &VariantExecution,
) -> Result<(), String> {
    let path = execution.directory.join("manifest.json");
    let mut file = fs::File::create(&path)
        .map_err(|error| format!("no se pudo crear '{}': {error}", path.display()))?;

    let assignments = variant
        .assignments()
        .iter()
        .map(|(name, value)| {
            format!(
                "{{\"factor\":\"{}\",\"value\":\"{}\"}}",
                json_escape(name),
                json_escape(value.as_str())
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let variables = resolved
        .variables()
        .iter()
        .map(|(name, value)| {
            format!(
                "{{\"name\":\"{}\",\"value\":\"{}\"}}",
                json_escape(name),
                json_escape(value)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let tools = resolved
        .tools()
        .iter()
        .map(|tool| {
            format!(
                "{{\"requested\":\"{}\",\"path\":\"{}\",\"version\":{}}}",
                json_escape(tool.requested()),
                json_escape(&tool.path().display().to_string()),
                option_json(tool.version())
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let command = options
        .command
        .iter()
        .map(|arg| format!("\"{}\"", json_escape(arg)))
        .collect::<Vec<_>>()
        .join(",");

    writeln!(
        file,
        "{{\"variant_id\":\"{}\",\"status\":\"{}\",\"exit_code\":{},\"duration_ms\":{},\"assignments\":[{}],\"variables\":[{}],\"tools\":[{}],\"command\":[{}]}}",
        json_escape(execution.variant_id()),
        execution.status().as_str(),
        execution
            .exit_code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "null".to_string()),
        execution.duration_ms(),
        assignments,
        variables,
        tools,
        command
    )
    .map_err(|error| format!("no se pudo escribir '{}': {error}", path.display()))
}

fn write_experiment_manifest(
    spec: &ExperimentSpec,
    variants: &[EnvironmentVariant],
    options: &ExploreOptions,
    bundle: &ExperimentBundle,
) -> Result<(), String> {
    let path = bundle.output_dir.join("experiment.json");
    let mut file = fs::File::create(&path)
        .map_err(|error| format!("no se pudo crear '{}': {error}", path.display()))?;

    let command = options
        .command
        .iter()
        .map(|arg| format!("\"{}\"", json_escape(arg)))
        .collect::<Vec<_>>()
        .join(",");

    let statuses = bundle
        .executions
        .iter()
        .map(|execution| {
            format!(
                "{{\"variant_id\":\"{}\",\"status\":\"{}\"}}",
                json_escape(execution.variant_id()),
                execution.status().as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    writeln!(
        file,
        "{{\"experiment\":\"{}\",\"schema_version\":{},\"planned\":{},\"succeeded\":{},\"unavailable\":{},\"failed\":{},\"workdir\":\"{}\",\"command\":[{}],\"variants\":[{}]}}",
        json_escape(spec.name()),
        spec.schema_version(),
        variants.len(),
        bundle.succeeded_count(),
        bundle.unavailable_count(),
        bundle.failed_count(),
        json_escape(&options.workdir.display().to_string()),
        command,
        statuses
    )
    .map_err(|error| format!("no se pudo escribir '{}': {error}", path.display()))
}

fn write_hashes(bundle: &ExperimentBundle) -> Result<(), String> {
    let mut files = vec![bundle.output_dir.join("experiment.json")];

    for execution in &bundle.executions {
        files.push(execution.directory.join("manifest.json"));
        files.push(execution.directory.join("stdout.txt"));
        files.push(execution.directory.join("stderr.txt"));
    }

    let mut out = fs::File::create(bundle.output_dir.join("hashes.sha256"))
        .map_err(|error| format!("no se pudo crear hashes.sha256: {error}"))?;

    for path in files {
        let output = Command::new("sha256sum")
            .arg(&path)
            .output()
            .map_err(|error| format!("no se pudo ejecutar sha256sum: {error}"))?;

        if !output.status.success() {
            return Err(format!("sha256sum falló para '{}'", path.display()));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let hash = text
            .split_whitespace()
            .next()
            .ok_or_else(|| format!("sha256sum no produjo hash para '{}'", path.display()))?;

        let relative = path.strip_prefix(&bundle.output_dir).unwrap_or(&path);

        writeln!(out, "{}  {}", hash, relative.display())
            .map_err(|error| format!("no se pudo escribir hashes.sha256: {error}"))?;
    }

    Ok(())
}

fn option_json(value: Option<&str>) -> String {
    value
        .map(|text| format!("\"{}\"", json_escape(text)))
        .unwrap_or_else(|| "null".to_string())
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

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|error| format!("no se pudo resolver ruta absoluta: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experiment::FactorValue;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    fn temp_dir(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "envmorph-executor-test-{}-{}-{}",
            std::process::id(),
            id,
            name
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn value(text: &str) -> FactorValue {
        FactorValue::new(text).unwrap()
    }

    #[test]
    fn json_escape_handles_control_characters() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
    }

    #[test]
    fn execution_status_has_stable_protocol_names() {
        assert_eq!(ExecutionStatus::Unavailable.as_str(), "unavailable");
        assert_eq!(
            ExecutionStatus::ExecutionSucceeded.as_str(),
            "execution_succeeded"
        );
        assert_eq!(
            ExecutionStatus::ExecutionFailed.as_str(),
            "execution_failed"
        );
    }

    #[test]
    fn validate_options_rejects_empty_command() {
        let options = ExploreOptions {
            output_dir: temp_dir("output").join("bundle"),
            workdir: temp_dir("workdir"),
            command: Vec::new(),
        };
        assert!(validate_options(&options).is_err());
    }

    #[test]
    fn copy_workspace_skips_repository_artifacts() {
        let source = temp_dir("copy-source");
        fs::write(source.join("keep.txt"), "ok").unwrap();
        fs::create_dir_all(source.join(".git")).unwrap();
        fs::write(source.join(".git").join("config"), "x").unwrap();
        fs::create_dir_all(source.join("target")).unwrap();
        fs::write(source.join("target").join("binary"), "x").unwrap();

        let destination = temp_dir("copy-destination").join("workspace");
        copy_workspace(&source, &destination, &source.join(".envmorph")).unwrap();

        assert!(destination.join("keep.txt").is_file());
        assert!(!destination.join(".git").exists());
        assert!(!destination.join("target").exists());
    }

    #[test]
    fn unavailable_variant_is_not_executed() {
        let workdir = temp_dir("unavailable-workdir");
        fs::write(workdir.join("marker.sh"), "#!/bin/sh\ntouch executed\n").unwrap();

        let variant = EnvironmentVariant::new(
            1,
            vec![(
                "awk_implementation".to_string(),
                value("envmorph-awk-that-does-not-exist"),
            )],
        )
        .unwrap();

        let output = temp_dir("unavailable-output").join("variant");
        let options = ExploreOptions {
            output_dir: output.parent().unwrap().to_path_buf(),
            workdir: workdir.clone(),
            command: vec!["sh".into(), "marker.sh".into()],
        };

        let execution = execute_variant(&variant, &options, &output).unwrap();
        assert_eq!(execution.status(), &ExecutionStatus::Unavailable);
        assert!(!output.join("workspace").join("executed").exists());
    }

    #[test]
    fn successful_variant_captures_stdout_and_stderr() {
        let workdir = temp_dir("success-workdir");
        fs::write(
            workdir.join("workflow.sh"),
            "#!/bin/sh\nprintf 'salida\\n'\nprintf 'error\\n' >&2\n",
        )
        .unwrap();

        let variant = EnvironmentVariant::new(1, vec![("locale".to_string(), value("C"))]).unwrap();

        let output_root = temp_dir("success-output");
        let variant_dir = output_root.join("variant");
        let options = ExploreOptions {
            output_dir: output_root,
            workdir,
            command: vec!["sh".into(), "workflow.sh".into()],
        };

        let execution = execute_variant(&variant, &options, &variant_dir).unwrap();
        assert_eq!(execution.status(), &ExecutionStatus::ExecutionSucceeded);
        assert_eq!(
            fs::read_to_string(variant_dir.join("stdout.txt")).unwrap(),
            "salida\n"
        );
        assert_eq!(
            fs::read_to_string(variant_dir.join("stderr.txt")).unwrap(),
            "error\n"
        );
    }

    #[test]
    fn failed_variant_preserves_exit_code() {
        let workdir = temp_dir("failed-workdir");
        fs::write(workdir.join("workflow.sh"), "#!/bin/sh\nexit 7\n").unwrap();

        let variant = EnvironmentVariant::new(1, vec![("locale".to_string(), value("C"))]).unwrap();

        let output_root = temp_dir("failed-output");
        let variant_dir = output_root.join("variant");
        let options = ExploreOptions {
            output_dir: output_root,
            workdir,
            command: vec!["sh".into(), "workflow.sh".into()],
        };

        let execution = execute_variant(&variant, &options, &variant_dir).unwrap();
        assert_eq!(execution.status(), &ExecutionStatus::ExecutionFailed);
        assert_eq!(execution.exit_code(), Some(7));
    }

    #[test]
    fn render_explore_reports_all_categories() {
        let spec = ExperimentSpec::from_toml_str(
            r#"
[experiment]
name = "summary"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "POSIX"]
"#,
        )
        .unwrap();

        let bundle = ExperimentBundle {
            output_dir: PathBuf::from("bundle"),
            executions: vec![
                VariantExecution {
                    variant_id: "variant-0001".into(),
                    status: ExecutionStatus::ExecutionSucceeded,
                    exit_code: Some(0),
                    duration_ms: 1,
                    directory: PathBuf::from("a"),
                },
                VariantExecution {
                    variant_id: "variant-0002".into(),
                    status: ExecutionStatus::Unavailable,
                    exit_code: None,
                    duration_ms: 0,
                    directory: PathBuf::from("b"),
                },
            ],
        };

        let text = render_explore(&spec, &bundle);
        assert!(text.contains("Variantes planificadas: 2"));
        assert!(text.contains("Ejecutadas correctamente: 1"));
        assert!(text.contains("No disponibles: 1"));
        assert!(text.contains("Fallidas: 0"));
    }
}
