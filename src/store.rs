//! Store en disco (`.envmorph/`), mapeo de rutas y hashing de artefactos.
//!
//! Estructura:
//!   <home>/CURRENT                    id de la ejecución iniciada más recientemente
//!   <home>/runs/run-NNN/manifest.json cabecera de ejecución (entorno, mapas, estado)
//!   <home>/runs/run-NNN/stages/NNN.json un registro por etapa
//!   <home>/objects/<sha256>           snapshots de archivos de salida pequeños

use crate::model::{Artifact, Manifest, Stage};
use crate::{hash, json};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub const SNAPSHOT_LIMIT: u64 = 8 * 1024 * 1024;

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
}

pub fn home() -> PathBuf {
    match std::env::var_os("ENVMORPH_HOME") {
        Some(h) if !h.is_empty() => absolutize(Path::new(&h)),
        _ => cwd().join(".envmorph"),
    }
}

pub fn run_dir(id: &str) -> PathBuf {
    home().join("runs").join(id)
}

pub fn objects_dir() -> PathBuf {
    home().join("objects")
}

pub fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn absolutize(p: &Path) -> PathBuf {
    if p.is_absolute() {
        normalize(p)
    } else {
        normalize(&cwd().join(p))
    }
}

/// Mapea rutas absolutas a nombres lógicos estables para que ejecuciones de
/// directorios o máquinas diferentes sigan siendo comparables.
pub struct PathMapper {
    maps: Vec<(String, String)>,
    cwd: String,
}

impl PathMapper {
    pub fn new(maps: &[(String, String)], cwd: &Path) -> PathMapper {
        let mut m: Vec<(String, String)> = maps.to_vec();
        m.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        PathMapper {
            maps: m,
            cwd: cwd.to_string_lossy().to_string(),
        }
    }

    pub fn logical(&self, abs: &Path) -> String {
        let s = abs.to_string_lossy().to_string();
        for (prefix, name) in &self.maps {
            if &s == prefix {
                return name.clone();
            }
            if let Some(rest) = s.strip_prefix(prefix.as_str()) {
                if rest.starts_with('/') {
                    return format!("{}{}", name, rest);
                }
            }
        }
        if s == self.cwd {
            return ".".to_string();
        }
        if let Some(rest) = s.strip_prefix(&format!("{}/", self.cwd)) {
            return rest.to_string();
        }
        s
    }

    /// Inversa de `logical`. Los nombres relativos se resuelven respecto del directorio actual.
    pub fn real(&self, logical: &str) -> PathBuf {
        for (prefix, name) in &self.maps {
            if logical == name {
                return PathBuf::from(prefix);
            }
            if let Some(rest) = logical.strip_prefix(name.as_str()) {
                if rest.starts_with('/') {
                    return PathBuf::from(format!("{}{}", prefix, rest));
                }
            }
        }
        let p = Path::new(logical);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            Path::new(&self.cwd).join(p)
        }
    }

    pub fn rewrite(&self, arg: &str) -> String {
        let mut cur = arg.to_string();
        for (prefix, name) in &self.maps {
            cur = replace_prefix(&cur, prefix, name);
        }
        cur
    }
}

fn replace_prefix(s: &str, prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        return s.to_string();
    }
    let mut out = String::new();
    let mut i = 0;
    while let Some(pos) = s[i..].find(prefix) {
        let start = i + pos;
        let end = start + prefix.len();
        let boundary = match s[end..].chars().next() {
            None => true,
            Some(c) => !(c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'),
        };
        out.push_str(&s[i..start]);
        if boundary {
            out.push_str(name);
        } else {
            out.push_str(prefix);
        }
        i = end;
    }
    out.push_str(&s[i..]);
    out
}

fn walk_dir(base: &Path, rel: &Path, depth: usize, acc: &mut BTreeMap<String, (String, u64)>) {
    if depth > 32 {
        return;
    }
    let dir = base.join(rel);
    let Ok(rd) = fs::read_dir(&dir) else { return };
    for entry in rd.flatten() {
        let rel_child = rel.join(entry.file_name());
        let full = base.join(&rel_child);
        match fs::metadata(&full) {
            Ok(md) if md.is_dir() => walk_dir(base, &rel_child, depth + 1, acc),
            Ok(md) if md.is_file() => {
                if let Ok((h, sz)) = hash::hash_file(&full) {
                    acc.insert(rel_child.to_string_lossy().to_string(), (h, sz));
                }
            }
            _ => {}
        }
    }
}

/// Calcula el hash de un archivo o directorio usando rutas relativas ordenadas y hashes de archivos.
pub fn hash_artifact(real: &Path, logical: String, role: &str) -> Artifact {
    match fs::metadata(real) {
        Ok(md) if md.is_file() => match hash::hash_file(real) {
            Ok((h, size)) => Artifact {
                path: logical,
                kind: "file".into(),
                sha256: Some(h),
                size,
                role: role.into(),
            },
            Err(_) => missing(logical, role),
        },
        Ok(md) if md.is_dir() => {
            let mut acc = BTreeMap::new();
            walk_dir(real, Path::new(""), 0, &mut acc);
            let mut buf = Vec::new();
            let mut total = 0u64;
            for (rel, (h, sz)) in &acc {
                buf.extend_from_slice(rel.as_bytes());
                buf.push(0);
                buf.extend_from_slice(h.as_bytes());
                buf.push(b'\n');
                total += sz;
            }
            Artifact {
                path: logical,
                kind: "dir".into(),
                sha256: Some(hash::hash_bytes(&buf)),
                size: total,
                role: role.into(),
            }
        }
        _ => missing(logical, role),
    }
}

fn missing(logical: String, role: &str) -> Artifact {
    Artifact {
        path: logical,
        kind: "missing".into(),
        sha256: None,
        size: 0,
        role: role.into(),
    }
}

pub fn snapshot(real: &Path, a: &Artifact) {
    if a.kind != "file" || a.size > SNAPSHOT_LIMIT {
        return;
    }
    let Some(h) = &a.sha256 else { return };
    let dir = objects_dir();
    let dest = dir.join(h);
    if dest.exists() || fs::create_dir_all(&dir).is_err() {
        return;
    }
    let tmp = dir.join(format!(".tmp-{}-{}", std::process::id(), now_ms()));
    if fs::copy(real, &tmp).is_ok() {
        let _ = fs::rename(&tmp, &dest);
    } else {
        let _ = fs::remove_file(&tmp);
    }
}

pub fn read_object(sha: &str) -> Option<Vec<u8>> {
    fs::read(objects_dir().join(sha)).ok()
}

pub fn list_runs() -> Vec<String> {
    let mut v: Vec<String> = match fs::read_dir(home().join("runs")) {
        Ok(rd) => rd
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n.starts_with("run-"))
            .collect(),
        Err(_) => Vec::new(),
    };
    v.sort();
    v
}

pub fn create_run(maps: Vec<(String, String)>, env: crate::model::Env) -> Result<Manifest, String> {
    let runs = home().join("runs");
    fs::create_dir_all(&runs)
        .map_err(|e| format!("no se puede crear {}: {}", runs.display(), e))?;
    let mut n = list_runs().len() + 1;
    loop {
        let id = format!("run-{:03}", n);
        match fs::create_dir(runs.join(&id)) {
            Ok(()) => {
                fs::create_dir_all(runs.join(&id).join("stages")).map_err(|e| e.to_string())?;
                let m = Manifest {
                    run_id: id.clone(),
                    status: "open".into(),
                    started_ms: now_ms(),
                    ended_ms: 0,
                    version: env!("CARGO_PKG_VERSION").into(),
                    maps,
                    env,
                    stages: Vec::new(),
                };
                save_header(&m)?;
                fs::write(home().join("CURRENT"), format!("{}\n", id))
                    .map_err(|e| e.to_string())?;
                return Ok(m);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => n += 1,
            Err(e) => {
                return Err(format!(
                    "no se puede crear el directorio de ejecución: {}",
                    e
                ))
            }
        }
    }
}

pub fn save_header(m: &Manifest) -> Result<(), String> {
    let path = run_dir(&m.run_id).join("manifest.json");
    fs::write(&path, m.header_json().pretty())
        .map_err(|e| format!("no se puede escribir {}: {}", path.display(), e))
}

pub fn current_run_id() -> Result<String, String> {
    if let Ok(v) = std::env::var("ENVMORPH_RUN") {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    fs::read_to_string(home().join("CURRENT"))
        .map(|s| s.trim().to_string())
        .map_err(|_| {
            "no hay una ejecución activa, usa `envmorph begin` o define ENVMORPH_RUN".to_string()
        })
}

/// Acepta "run-003", "3" o "latest".
pub fn resolve_run(arg: &str) -> Result<String, String> {
    let runs = list_runs();
    if arg == "latest" {
        return runs
            .last()
            .cloned()
            .ok_or_else(|| "no hay ejecuciones registradas".to_string());
    }
    let id = if let Ok(n) = arg.parse::<usize>() {
        format!("run-{:03}", n)
    } else {
        arg.to_string()
    };
    if runs.contains(&id) {
        Ok(id)
    } else {
        Err(format!(
            "ejecución desconocida '{}' (conocidas: {})",
            arg,
            runs.join(", ")
        ))
    }
}

pub fn load_header(id: &str) -> Result<Manifest, String> {
    let path = run_dir(id).join("manifest.json");
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("no se puede leer {}: {}", path.display(), e))?;
    let j = json::parse(&text).map_err(|e| format!("{}: {}", path.display(), e))?;
    Manifest::header_from_json(&j)
}

pub fn load_run(id: &str) -> Result<Manifest, String> {
    let mut m = load_header(id)?;
    let dir = run_dir(id).join("stages");
    let mut files: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
            .collect(),
        Err(_) => Vec::new(),
    };
    files.sort();
    for f in files {
        let text = fs::read_to_string(&f)
            .map_err(|e| format!("no se puede leer {}: {}", f.display(), e))?;
        let j = json::parse(&text).map_err(|e| format!("{}: {}", f.display(), e))?;
        m.stages
            .push(Stage::from_json(&j).map_err(|e| format!("{}: {}", f.display(), e))?);
    }
    Ok(m)
}

/// Reserva atómicamente el siguiente índice de etapa y escribe el registro.
pub fn write_stage(run_id: &str, mut st: Stage) -> Result<Stage, String> {
    let dir = run_dir(run_id).join("stages");
    let mut n = fs::read_dir(&dir)
        .map(|rd| rd.flatten().count())
        .unwrap_or(0) as u32
        + 1;
    loop {
        let path = dir.join(format!("{:03}.json", n));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                st.index = n;
                f.write_all(st.to_json().pretty().as_bytes())
                    .map_err(|e| e.to_string())?;
                return Ok(st);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => n += 1,
            Err(e) => return Err(format!("no se puede escribir el registro de etapa: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper_prefers_longest_prefix_and_relative() {
        let maps = vec![
            ("/data/out".to_string(), "@out".to_string()),
            ("/data".to_string(), "@data".to_string()),
        ];
        let m = PathMapper::new(&maps, Path::new("/work/proj"));
        assert_eq!(m.logical(Path::new("/data/out/a.tsv")), "@out/a.tsv");
        assert_eq!(m.logical(Path::new("/data/x.csv")), "@data/x.csv");
        assert_eq!(
            m.logical(Path::new("/work/proj/scripts/a.awk")),
            "scripts/a.awk"
        );
        assert_eq!(m.logical(Path::new("/etc/passwd")), "/etc/passwd");
        assert_eq!(m.rewrite("-v x=/data/out/a"), "-v x=@out/a");
        assert_eq!(m.rewrite("/data/outer"), "@data/outer");
    }

    #[test]
    fn normalize_dots() {
        assert_eq!(normalize(Path::new("/a/./b/../c")), PathBuf::from("/a/c"));
    }
}
