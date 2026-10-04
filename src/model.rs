//! Modelo de datos: artefactos, registros de etapas, manifest de ejecución y su representación JSON.

use crate::json::Json;

#[derive(Clone, Debug, PartialEq)]
pub struct Artifact {
    pub path: String,
    /// Valores de protocolo: "file", "dir" o "missing".
    pub kind: String,
    pub sha256: Option<String>,
    pub size: u64,
    /// Valores de protocolo: "data" o "code" para scripts ejecutados por la etapa.
    pub role: String,
}

impl Artifact {
    pub fn digest(&self) -> String {
        match &self.sha256 {
            Some(h) => h.clone(),
            None => "<missing>".to_string(),
        }
    }

    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("path", Json::str(self.path.clone())),
            ("kind", Json::str(self.kind.clone())),
            (
                "sha256",
                self.sha256.clone().map(Json::Str).unwrap_or(Json::Null),
            ),
            ("size", Json::Num(self.size as i64)),
            ("role", Json::str(self.role.clone())),
        ])
    }

    pub fn from_json(j: &Json) -> Result<Artifact, String> {
        Ok(Artifact {
            path: field_str(j, "path")?,
            kind: field_str(j, "kind")?,
            sha256: j
                .get("sha256")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            size: j.get("size").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            role: j
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("data")
                .to_string(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tool {
    pub path: String,
    pub sha256: String,
    pub version: String,
}

impl Tool {
    pub fn label(&self) -> String {
        if self.version.is_empty() {
            format!("{} (sha256 {})", self.path, short(&self.sha256))
        } else {
            format!("{} [{}]", self.path, self.version)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stage {
    pub index: u32,
    pub name: String,
    pub command: Vec<String>,
    pub cwd: String,
    pub tool: Option<Tool>,
    pub inputs: Vec<Artifact>,
    pub outputs: Vec<Artifact>,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub started_ms: u64,
    /// Rutas lógicas que recibieron stdout o stderr de la orden para replay.
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Env {
    pub os: String,
    pub kernel: String,
    pub arch: String,
    pub vars: Vec<(String, String)>,
    pub git_commit: String,
    pub git_dirty: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Manifest {
    pub run_id: String,
    pub status: String,
    pub started_ms: u64,
    pub ended_ms: u64,
    pub version: String,
    pub maps: Vec<(String, String)>,
    pub env: Env,
    pub stages: Vec<Stage>,
}

pub fn short(h: &str) -> String {
    h.chars().take(12).collect()
}

fn field_str(j: &Json, k: &str) -> Result<String, String> {
    j.get(k)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("falta el campo de texto '{}'", k))
}

fn pairs_to_json(p: &[(String, String)]) -> Json {
    Json::Obj(
        p.iter()
            .map(|(k, v)| (k.clone(), Json::str(v.clone())))
            .collect(),
    )
}

fn pairs_from_json(j: Option<&Json>) -> Vec<(String, String)> {
    match j {
        Some(Json::Obj(o)) => o
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect(),
        _ => Vec::new(),
    }
}

impl Tool {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("path", Json::str(self.path.clone())),
            ("sha256", Json::str(self.sha256.clone())),
            ("version", Json::str(self.version.clone())),
        ])
    }

    pub fn from_json(j: &Json) -> Result<Tool, String> {
        Ok(Tool {
            path: field_str(j, "path")?,
            sha256: field_str(j, "sha256")?,
            version: field_str(j, "version").unwrap_or_default(),
        })
    }
}

impl Stage {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("index", Json::Num(self.index as i64)),
            ("name", Json::str(self.name.clone())),
            (
                "command",
                Json::Arr(self.command.iter().map(|s| Json::str(s.clone())).collect()),
            ),
            ("cwd", Json::str(self.cwd.clone())),
            (
                "tool",
                self.tool
                    .as_ref()
                    .map(|t| t.to_json())
                    .unwrap_or(Json::Null),
            ),
            (
                "inputs",
                Json::Arr(self.inputs.iter().map(|a| a.to_json()).collect()),
            ),
            (
                "outputs",
                Json::Arr(self.outputs.iter().map(|a| a.to_json()).collect()),
            ),
            ("exit_code", Json::Num(self.exit_code as i64)),
            ("duration_ms", Json::Num(self.duration_ms as i64)),
            ("started_ms", Json::Num(self.started_ms as i64)),
            (
                "stdout",
                self.stdout.clone().map(Json::Str).unwrap_or(Json::Null),
            ),
            (
                "stderr",
                self.stderr.clone().map(Json::Str).unwrap_or(Json::Null),
            ),
        ])
    }

    pub fn from_json(j: &Json) -> Result<Stage, String> {
        let arts = |k: &str| -> Result<Vec<Artifact>, String> {
            match j.get(k).and_then(|v| v.as_arr()) {
                Some(a) => a.iter().map(Artifact::from_json).collect(),
                None => Ok(Vec::new()),
            }
        };
        let command = j
            .get("command")
            .and_then(|v| v.as_arr())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let tool = match j.get("tool") {
            Some(t @ Json::Obj(_)) => Some(Tool::from_json(t)?),
            _ => None,
        };
        Ok(Stage {
            index: j.get("index").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
            name: field_str(j, "name")?,
            command,
            cwd: field_str(j, "cwd").unwrap_or_default(),
            tool,
            inputs: arts("inputs")?,
            outputs: arts("outputs")?,
            exit_code: j.get("exit_code").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            duration_ms: j.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            started_ms: j.get("started_ms").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            stdout: j
                .get("stdout")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            stderr: j
                .get("stderr")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
}

impl Env {
    pub fn to_json(&self) -> Json {
        Json::obj(vec![
            ("os", Json::str(self.os.clone())),
            ("kernel", Json::str(self.kernel.clone())),
            ("arch", Json::str(self.arch.clone())),
            ("vars", pairs_to_json(&self.vars)),
            ("git_commit", Json::str(self.git_commit.clone())),
            ("git_dirty", Json::Bool(self.git_dirty)),
        ])
    }

    pub fn from_json(j: &Json) -> Env {
        Env {
            os: field_str(j, "os").unwrap_or_default(),
            kernel: field_str(j, "kernel").unwrap_or_default(),
            arch: field_str(j, "arch").unwrap_or_default(),
            vars: pairs_from_json(j.get("vars")),
            git_commit: field_str(j, "git_commit").unwrap_or_default(),
            git_dirty: j
                .get("git_dirty")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }
}

impl Manifest {
    /// Solo la cabecera. Las etapas permanecen en archivos separados mientras la ejecución está abierta.
    pub fn header_json(&self) -> Json {
        Json::obj(vec![
            ("run_id", Json::str(self.run_id.clone())),
            ("status", Json::str(self.status.clone())),
            ("envmorph_version", Json::str(self.version.clone())),
            ("started_ms", Json::Num(self.started_ms as i64)),
            ("ended_ms", Json::Num(self.ended_ms as i64)),
            ("maps", pairs_to_json(&self.maps)),
            ("environment", self.env.to_json()),
            ("stage_count", Json::Num(self.stages.len() as i64)),
        ])
    }

    pub fn full_json(&self) -> Json {
        let mut j = self.header_json();
        if let Json::Obj(o) = &mut j {
            o.push((
                "stages".to_string(),
                Json::Arr(self.stages.iter().map(|s| s.to_json()).collect()),
            ));
        }
        j
    }

    pub fn header_from_json(j: &Json) -> Result<Manifest, String> {
        Ok(Manifest {
            run_id: field_str(j, "run_id")?,
            status: field_str(j, "status")?,
            started_ms: j.get("started_ms").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            ended_ms: j.get("ended_ms").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
            version: field_str(j, "envmorph_version").unwrap_or_default(),
            maps: pairs_from_json(j.get("maps")),
            env: j.get("environment").map(Env::from_json).unwrap_or_default(),
            stages: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn art(path: &str, h: &str) -> Artifact {
        Artifact {
            path: path.into(),
            kind: "file".into(),
            sha256: Some(h.into()),
            size: 1,
            role: "data".into(),
        }
    }

    #[test]
    fn stage_roundtrip() {
        let s = Stage {
            index: 3,
            name: "merge".into(),
            command: vec!["awk".into(), "-f".into(), "x.awk".into()],
            cwd: "/w".into(),
            tool: Some(Tool {
                path: "/usr/bin/mawk".into(),
                sha256: "ab".into(),
                version: "mawk 1.3.4".into(),
            }),
            inputs: vec![art("a", "1")],
            outputs: vec![art("b", "2")],
            exit_code: 0,
            duration_ms: 12,
            started_ms: 99,
            stdout: Some("b".into()),
            stderr: None,
        };
        let j = crate::json::parse(&s.to_json().pretty()).unwrap();
        assert_eq!(Stage::from_json(&j).unwrap(), s);
    }
}
