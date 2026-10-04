//! Localización de la primera divergencia entre dos ejecuciones.
//!
//! Las etapas se emparejan por posición y sus nombres deben coincidir. Para cada
//! par se comparan la orden, la identidad del ejecutable, entradas de código,
//! entradas de datos y salidas. Una etapa con salidas diferentes es una
//! divergencia *root* salvo que toda diferencia de entrada se explique por una
//! salida divergente anterior, en cuyo caso es *propagated*. La primera
//! divergencia root constituye el resultado de localización.

use crate::environment::INFORMATIONAL_VARS;
use crate::model::{Manifest, Stage};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub enum Cause {
    CodeChanged(Vec<String>),
    CommandChanged,
    ToolChanged(String, String),
    ExternalInput(Vec<String>),
    Propagated(Vec<String>),
    EnvVars(Vec<String>),
    Unexplained,
}

impl Cause {
    pub fn kind(&self) -> &'static str {
        match self {
            Cause::CodeChanged(_) => "code",
            Cause::CommandChanged => "command",
            Cause::ToolChanged(_, _) => "toolchain",
            Cause::ExternalInput(_) => "data",
            Cause::Propagated(_) => "propagated",
            Cause::EnvVars(_) => "environment",
            Cause::Unexplained => "unexplained",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Identical,
    /// Las entradas, orden o herramienta difieren, pero las salidas y el código de salida son idénticos.
    Absorbed,
    /// Las salidas difieren solo porque una etapa anterior cambió las entradas.
    Propagated,
    /// Las salidas difieren y esta etapa tiene un cambio propio de herramienta,
    /// orden, código o entrada externa, pero también recibió entradas modificadas
    /// aguas arriba. El cambio local no puede separarse de la propagación sin
    /// repetir la etapa con entradas idénticas.
    Confounded,
    /// Las salidas o el código de salida difieren aunque todas las entradas sean
    /// idénticas o hayan cambiado fuera del pipeline. La causa es local a esta etapa.
    Root,
}

#[derive(Debug, Clone)]
pub struct StageDiff {
    pub position: usize,
    pub name: String,
    pub status: Status,
    pub outputs_diff: Vec<String>,
    pub inputs_diff: Vec<String>,
    pub code_diff: Vec<String>,
    pub command_diff: bool,
    pub tool_diff: Option<(String, String)>,
    pub exit_diff: Option<(i32, i32)>,
    pub causes: Vec<Cause>,
}

#[derive(Debug, Clone, Default)]
pub struct RunDiff {
    pub structural: Option<String>,
    pub env_diff: Vec<(String, String, String)>,
    pub notes: Vec<String>,
    pub stages: Vec<StageDiff>,
}

impl RunDiff {
    pub fn first_root(&self) -> Option<&StageDiff> {
        self.stages.iter().find(|s| s.status == Status::Root)
    }

    pub fn roots(&self) -> Vec<&StageDiff> {
        self.stages
            .iter()
            .filter(|s| s.status == Status::Root)
            .collect()
    }

    pub fn count(&self, st: Status) -> usize {
        self.stages.iter().filter(|s| s.status == st).count()
    }

    pub fn identical(&self) -> bool {
        self.structural.is_none()
            && self
                .stages
                .iter()
                .all(|s| matches!(s.status, Status::Identical | Status::Absorbed))
    }
}

fn digest_map(arts: &[crate::model::Artifact], role: &str) -> BTreeMap<String, String> {
    arts.iter()
        .filter(|a| a.role == role)
        .map(|a| (a.path.clone(), a.digest()))
        .collect()
}

fn differing(a: &BTreeMap<String, String>, b: &BTreeMap<String, String>) -> Vec<String> {
    let keys: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    keys.into_iter()
        .filter(|k| a.get(*k) != b.get(*k))
        .cloned()
        .collect()
}

pub fn env_differences(a: &Manifest, b: &Manifest) -> (Vec<(String, String, String)>, Vec<String>) {
    let mut diffs = Vec::new();
    let mut notes = Vec::new();
    let unset = || "<no definida>".to_string();
    let get = |m: &Manifest, k: &str| {
        m.env
            .vars
            .iter()
            .find(|(n, _)| n == k)
            .map(|(_, v)| v.clone())
    };
    let mut names: BTreeSet<String> = BTreeSet::new();
    for m in [a, b] {
        for (k, _) in &m.env.vars {
            names.insert(k.clone());
        }
    }
    for k in names {
        let (va, vb) = (get(a, &k), get(b, &k));
        if va != vb {
            if INFORMATIONAL_VARS.contains(&k.as_str()) {
                notes.push(format!("{} difiere (informativo)", k));
            } else {
                diffs.push((k, va.unwrap_or_else(unset), vb.unwrap_or_else(unset)));
            }
        }
    }
    for (label, x, y) in [
        ("kernel", &a.env.kernel, &b.env.kernel),
        ("arch", &a.env.arch, &b.env.arch),
        ("os", &a.env.os, &b.env.os),
    ] {
        if x != y {
            diffs.push((label.to_string(), x.clone(), y.clone()));
        }
    }
    if a.env.git_commit != b.env.git_commit {
        notes.push("el commit de git difiere".to_string());
    }
    (diffs, notes)
}

fn compare_stage(
    position: usize,
    sa: &Stage,
    sb: &Stage,
    diverged: &BTreeSet<String>,
    env_names: &[String],
) -> StageDiff {
    let outputs_diff = differing(
        &digest_map(&sa.outputs, "data"),
        &digest_map(&sb.outputs, "data"),
    );
    let inputs_diff = differing(
        &digest_map(&sa.inputs, "data"),
        &digest_map(&sb.inputs, "data"),
    );
    let code_diff = differing(
        &digest_map(&sa.inputs, "code"),
        &digest_map(&sb.inputs, "code"),
    );
    let command_diff = sa.command != sb.command;
    let tool_diff = match (&sa.tool, &sb.tool) {
        (Some(x), Some(y)) if x.sha256 != y.sha256 => Some((x.label(), y.label())),
        (Some(x), None) => Some((x.label(), "<no encontrado>".to_string())),
        (None, Some(y)) => Some(("<no encontrado>".to_string(), y.label())),
        _ => None,
    };
    let exit_diff = if sa.exit_code != sb.exit_code {
        Some((sa.exit_code, sb.exit_code))
    } else {
        None
    };

    let output_changed = !outputs_diff.is_empty() || exit_diff.is_some();
    let local_change = !code_diff.is_empty() || command_diff || tool_diff.is_some();
    let any_change = local_change || !inputs_diff.is_empty();

    let (external, propagated): (Vec<String>, Vec<String>) = inputs_diff
        .iter()
        .cloned()
        .partition(|p| !diverged.contains(p));

    let mut causes = Vec::new();
    if output_changed {
        if !code_diff.is_empty() {
            causes.push(Cause::CodeChanged(code_diff.clone()));
        }
        if command_diff {
            causes.push(Cause::CommandChanged);
        }
        if let Some((x, y)) = &tool_diff {
            causes.push(Cause::ToolChanged(x.clone(), y.clone()));
        }
        if !external.is_empty() {
            causes.push(Cause::ExternalInput(external.clone()));
        }
        if !propagated.is_empty() {
            causes.push(Cause::Propagated(propagated.clone()));
        }
        if causes.is_empty() {
            if env_names.is_empty() {
                causes.push(Cause::Unexplained);
            } else {
                causes.push(Cause::EnvVars(env_names.to_vec()));
            }
        }
    }

    let status = if !output_changed {
        if any_change {
            Status::Absorbed
        } else {
            Status::Identical
        }
    } else if !local_change && external.is_empty() && !propagated.is_empty() {
        Status::Propagated
    } else if !propagated.is_empty() {
        Status::Confounded
    } else {
        Status::Root
    };

    StageDiff {
        position,
        name: sa.name.clone(),
        status,
        outputs_diff,
        inputs_diff,
        code_diff,
        command_diff,
        tool_diff,
        exit_diff,
        causes,
    }
}

pub fn compare(a: &Manifest, b: &Manifest) -> RunDiff {
    let (env_diff, notes) = env_differences(a, b);
    let env_names: Vec<String> = env_diff.iter().map(|(k, _, _)| k.clone()).collect();
    let mut rd = RunDiff {
        structural: None,
        env_diff,
        notes,
        stages: Vec::new(),
    };
    let mut diverged: BTreeSet<String> = BTreeSet::new();
    let common = a.stages.len().min(b.stages.len());
    for i in 0..common {
        let (sa, sb) = (&a.stages[i], &b.stages[i]);
        if sa.name != sb.name {
            rd.structural = Some(format!(
                "la etapa {} es '{}' en {} pero '{}' en {}",
                i + 1,
                sa.name,
                a.run_id,
                sb.name,
                b.run_id
            ));
            return rd;
        }
        let sd = compare_stage(i + 1, sa, sb, &diverged, &env_names);
        if matches!(
            sd.status,
            Status::Root | Status::Propagated | Status::Confounded
        ) {
            for p in &sd.outputs_diff {
                diverged.insert(p.clone());
            }
        }
        rd.stages.push(sd);
    }
    if a.stages.len() != b.stages.len() {
        rd.structural = Some(format!(
            "{} registró {} etapas pero {} registró {}",
            a.run_id,
            a.stages.len(),
            b.run_id,
            b.stages.len()
        ));
    }
    rd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Artifact, Env, Tool};

    fn art(path: &str, h: &str, role: &str) -> Artifact {
        Artifact {
            path: path.into(),
            kind: "file".into(),
            sha256: Some(h.into()),
            size: 1,
            role: role.into(),
        }
    }

    fn stage(i: u32, name: &str, ins: &[(&str, &str)], outs: &[(&str, &str)]) -> Stage {
        Stage {
            index: i,
            name: name.into(),
            command: vec!["awk".into(), "-f".into(), format!("{}.awk", name)],
            cwd: ".".into(),
            tool: Some(Tool {
                path: "/usr/bin/mawk".into(),
                sha256: "T1".into(),
                version: "mawk".into(),
            }),
            inputs: ins.iter().map(|(p, h)| art(p, h, "data")).collect(),
            outputs: outs.iter().map(|(p, h)| art(p, h, "data")).collect(),
            exit_code: 0,
            duration_ms: 1,
            started_ms: 0,
            stdout: None,
            stderr: None,
        }
    }

    fn run(id: &str, stages: Vec<Stage>) -> Manifest {
        Manifest {
            run_id: id.into(),
            status: "closed".into(),
            started_ms: 0,
            ended_ms: 0,
            version: "t".into(),
            maps: vec![],
            env: Env::default(),
            stages,
        }
    }

    fn chain(h_in: &str, h1: &str, h2: &str, h3: &str) -> Vec<Stage> {
        vec![
            stage(1, "s1", &[("in", h_in)], &[("o1", h1)]),
            stage(2, "s2", &[("o1", h1)], &[("o2", h2)]),
            stage(3, "s3", &[("o2", h2)], &[("o3", h3)]),
        ]
    }

    #[test]
    fn identical_runs() {
        let a = run("a", chain("i", "1", "2", "3"));
        let b = run("b", chain("i", "1", "2", "3"));
        let d = compare(&a, &b);
        assert!(d.identical());
        assert!(d.first_root().is_none());
    }

    #[test]
    fn external_input_change_is_root_then_propagates() {
        let a = run("a", chain("i", "1", "2", "3"));
        let b = run("b", chain("i2", "1x", "2x", "3x"));
        let d = compare(&a, &b);
        let first = d.first_root().unwrap();
        assert_eq!(first.position, 1);
        assert_eq!(first.causes, vec![Cause::ExternalInput(vec!["in".into()])]);
        assert_eq!(d.stages[1].status, Status::Propagated);
        assert_eq!(d.stages[2].status, Status::Propagated);
        assert_eq!(d.roots().len(), 1);
    }

    #[test]
    fn tool_change_localizes_to_middle_stage() {
        let a = run("a", chain("i", "1", "2", "3"));
        let mut bs = chain("i", "1", "2x", "3x");
        bs[1].tool.as_mut().unwrap().sha256 = "T2".into();
        let b = run("b", bs);
        let d = compare(&a, &b);
        let first = d.first_root().unwrap();
        assert_eq!(first.position, 2);
        assert_eq!(first.causes[0].kind(), "toolchain");
        assert_eq!(d.stages[2].status, Status::Propagated);
    }

    #[test]
    fn same_tool_change_downstream_is_confounded_not_root() {
        let a = run("a", chain("i", "1", "2", "3"));
        let mut bs = chain("i", "1", "2x", "3x");
        bs[1].tool.as_mut().unwrap().sha256 = "T2".into();
        bs[2].tool.as_mut().unwrap().sha256 = "T2".into();
        let d = compare(&a, &run("b", bs));
        assert_eq!(d.stages[1].status, Status::Root);
        assert_eq!(d.stages[2].status, Status::Confounded);
        assert_eq!(d.roots().len(), 1);
        assert_eq!(d.count(Status::Confounded), 1);
    }

    #[test]
    fn nondeterminism_is_unexplained() {
        let a = run("a", chain("i", "1", "2", "3"));
        let b = run("b", chain("i", "1", "2x", "3x"));
        let d = compare(&a, &b);
        assert_eq!(d.first_root().unwrap().causes, vec![Cause::Unexplained]);
    }

    #[test]
    fn env_difference_becomes_candidate_cause() {
        let a = run("a", chain("i", "1", "2", "3"));
        let mut b = run("b", chain("i", "1", "2x", "3x"));
        b.env.vars.push(("LC_ALL".into(), "C".into()));
        let d = compare(&a, &b);
        assert_eq!(
            d.first_root().unwrap().causes,
            vec![Cause::EnvVars(vec!["LC_ALL".into()])]
        );
    }

    #[test]
    fn absorbed_difference_is_not_a_divergence() {
        let a = run("a", chain("i", "1", "2", "3"));
        let b = run("b", chain("i2", "1", "2", "3"));
        let d = compare(&a, &b);
        assert!(d.identical());
        assert_eq!(d.stages[0].status, Status::Absorbed);
    }

    #[test]
    fn code_change_detected() {
        let mut as_ = chain("i", "1", "2", "3");
        as_[1].inputs.push(art("s2.awk", "c1", "code"));
        let mut bs = chain("i", "1", "2x", "3x");
        bs[1].inputs.push(art("s2.awk", "c2", "code"));
        let d = compare(&run("a", as_), &run("b", bs));
        assert_eq!(
            d.first_root().unwrap().causes,
            vec![Cause::CodeChanged(vec!["s2.awk".into()])]
        );
    }

    #[test]
    fn structural_mismatch() {
        let a = run("a", chain("i", "1", "2", "3"));
        let b = run("b", chain("i", "1", "2", "3")[..2].to_vec());
        assert!(compare(&a, &b).structural.is_some());
    }
}
