//! Captura del entorno y de la toolchain.

use crate::model::{Env, Tool};
use crate::{hash, store};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Variables que suelen cambiar el comportamiento de herramientas, como locale, zona horaria y shell.
const DEFAULT_VARS: &[&str] = &[
    "LANG",
    "LC_ALL",
    "LC_COLLATE",
    "LC_CTYPE",
    "LC_NUMERIC",
    "LC_TIME",
    "TZ",
    "SHELL",
    "PATH",
    "POSIXLY_CORRECT",
    "SOURCE_DATE_EPOCH",
];

/// Variables almacenadas como referencia que no se reportan como causa de divergencia.
pub const INFORMATIONAL_VARS: &[&str] = &["PATH", "SHELL"];

const VERSION_PROBE_TOOLS: &[&str] = &[
    "awk", "gawk", "mawk", "nawk", "sed", "sort", "grep", "egrep", "fgrep", "bash", "sh", "cut",
    "head", "tail", "tr", "wc", "cat", "uniq", "paste", "join", "date", "find", "xargs",
];

fn read_trim(p: &str) -> String {
    fs::read_to_string(p)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn capture(extra_vars: &[String]) -> Env {
    let mut names: Vec<String> = DEFAULT_VARS.iter().map(|s| s.to_string()).collect();
    for e in extra_vars {
        if !names.contains(e) {
            names.push(e.clone());
        }
    }
    let vars = names
        .into_iter()
        .filter_map(|n| std::env::var(&n).ok().map(|v| (n, v)))
        .collect();
    let git_commit = git(&["rev-parse", "HEAD"]).unwrap_or_default();
    let git_dirty = !git_commit.is_empty()
        && git(&["status", "--porcelain"])
            .map(|s| !s.is_empty())
            .unwrap_or(false);
    Env {
        os: read_trim("/proc/sys/kernel/ostype"),
        kernel: read_trim("/proc/sys/kernel/osrelease"),
        arch: std::env::consts::ARCH.to_string(),
        vars,
        git_commit,
        git_dirty,
    }
}

pub fn resolve_exe(name: &str) -> Option<PathBuf> {
    if name.contains('/') {
        let p = store::absolutize(Path::new(name));
        return if p.is_file() { Some(p) } else { None };
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

fn probe_version(exe: &Path) -> String {
    let probes: [&[&str]; 4] = [&["--version"], &["-W", "version"], &["-version"], &["-V"]];
    for args in probes.iter() {
        if let Ok(out) = Command::new(exe)
            .args(*args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout).to_string();
                if let Some(line) = text.lines().find(|l| !l.trim().is_empty()) {
                    return line.trim().chars().take(120).collect();
                }
            }
        }
    }
    String::new()
}

/// Identidad del ejecutable, usando el hash del contenido tras resolver enlaces
/// simbólicos, y versión best-effort para una lista acotada de herramientas conocidas.
pub fn tool_info(cmd0: &str, run_dir: &Path) -> Option<Tool> {
    let found = resolve_exe(cmd0)?;
    let canon = fs::canonicalize(&found).unwrap_or(found.clone());
    let (sha, _) = hash::hash_file(&canon).ok()?;
    let invoked = found
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let real = canon
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let probe = VERSION_PROBE_TOOLS.contains(&invoked.as_str())
        || VERSION_PROBE_TOOLS.contains(&real.as_str());
    let key = format!("{}\t{}", canon.display(), sha);
    let cache_path = run_dir.join("tools.cache");
    let mut version = String::new();
    if probe {
        let cached = fs::read_to_string(&cache_path).unwrap_or_default();
        if let Some(line) = cached
            .lines()
            .find(|l| l.starts_with(&format!("{}\t", key)))
        {
            version = line[key.len() + 1..].to_string();
        } else {
            version = probe_version(&canon);
            if let Ok(mut f) = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&cache_path)
            {
                use std::io::Write;
                let _ = writeln!(f, "{}\t{}", key, version);
            }
        }
    }
    Some(Tool {
        path: canon.to_string_lossy().to_string(),
        sha256: sha,
        version,
    })
}
