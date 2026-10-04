//! Exportación del grafo de procedencia en Graphviz DOT. Los artefactos están
//! versionados: cada escritura sobre una ruta crea un nodo nuevo y conserva
//! distinguibles los archivos sobrescritos.

use crate::compare::{RunDiff, Status};
use crate::model::{short, Manifest};
use std::collections::{BTreeMap, BTreeSet};

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn dot(m: &Manifest, diff: Option<&RunDiff>) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "digraph \"{}\" {{\n  rankdir=LR;\n  node [fontname=\"Helvetica\", fontsize=10];\n",
        esc(&m.run_id)
    ));
    let mut current: BTreeMap<String, String> = BTreeMap::new();
    let mut counter = 0usize;
    let mut node_lines = Vec::new();
    let mut edge_lines = Vec::new();
    let mut diverged_paths: BTreeSet<String> = BTreeSet::new();
    let mut stage_status: BTreeMap<usize, Status> = BTreeMap::new();
    if let Some(d) = diff {
        for s in &d.stages {
            stage_status.insert(s.position, s.status);
            if matches!(
                s.status,
                Status::Root | Status::Propagated | Status::Confounded
            ) {
                for p in &s.outputs_diff {
                    diverged_paths.insert(p.clone());
                }
            }
        }
    }
    let first_root = diff.and_then(|d| d.first_root()).map(|s| s.position);

    for (i, st) in m.stages.iter().enumerate() {
        let sid = format!("s{}", i + 1);
        let (fill, extra) = match stage_status.get(&(i + 1)) {
            Some(Status::Root) if Some(i + 1) == first_root => {
                ("#f4a6a6", ", penwidth=3, color=\"#b00020\"")
            }
            Some(Status::Root) => ("#f4a6a6", ", color=\"#b00020\""),
            Some(Status::Confounded) => ("#f2b6a0", ", color=\"#b00020\""),
            Some(Status::Propagated) => ("#fbd9a5", ""),
            Some(Status::Absorbed) => ("#f2f2c2", ""),
            _ if st.exit_code != 0 => ("#d9d9d9", ", style=\"filled,dashed\""),
            _ => ("#e8e8e8", ""),
        };
        let mut label = format!("{:03} {}", i + 1, st.name);
        if st.exit_code != 0 {
            label.push_str(&format!("\\nexit {}", st.exit_code));
        }
        node_lines.push(format!(
            "  {} [shape=box, style=filled, fillcolor=\"{}\"{}, label=\"{}\"];",
            sid,
            fill,
            extra,
            esc(&label).replace("\\\\n", "\\n")
        ));
        for a in &st.inputs {
            let nid = match current.get(&a.path) {
                Some(n) => n.clone(),
                None => {
                    counter += 1;
                    let n = format!("a{}", counter);
                    node_lines.push(format!("  {} [shape=ellipse, style=filled, fillcolor=\"#d6ead6\", label=\"{}\\n{}\"];", n, esc(&a.path), short(&a.digest())));
                    current.insert(a.path.clone(), n.clone());
                    n
                }
            };
            let style = if a.role == "code" {
                " [style=dashed]"
            } else {
                ""
            };
            edge_lines.push(format!("  {} -> {}{};", nid, sid, style));
        }
        for a in &st.outputs {
            counter += 1;
            let n = format!("a{}", counter);
            let bad = diverged_paths.contains(&a.path);
            let attrs = if bad {
                ", color=\"#b00020\", penwidth=2"
            } else {
                ""
            };
            node_lines.push(format!(
                "  {} [shape=ellipse{}, label=\"{}\\n{}\"];",
                n,
                attrs,
                esc(&a.path),
                short(&a.digest())
            ));
            current.insert(a.path.clone(), n.clone());
            edge_lines.push(format!("  {} -> {};", sid, n));
        }
    }
    for l in node_lines.iter().chain(edge_lines.iter()) {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Artifact, Env, Stage};

    #[test]
    fn overwritten_artifact_gets_new_version() {
        let art = |p: &str, h: &str| Artifact {
            path: p.into(),
            kind: "file".into(),
            sha256: Some(h.into()),
            size: 1,
            role: "data".into(),
        };
        let st = |i: u32, ins: Vec<Artifact>, outs: Vec<Artifact>| Stage {
            index: i,
            name: format!("s{}", i),
            command: vec![],
            cwd: ".".into(),
            tool: None,
            inputs: ins,
            outputs: outs,
            exit_code: 0,
            duration_ms: 0,
            started_ms: 0,
            stdout: None,
            stderr: None,
        };
        let m = Manifest {
            run_id: "r".into(),
            status: "closed".into(),
            started_ms: 0,
            ended_ms: 0,
            version: "t".into(),
            maps: vec![],
            env: Env::default(),
            stages: vec![
                st(1, vec![art("in", "i")], vec![art("x", "1")]),
                st(2, vec![art("x", "1")], vec![art("x", "2")]),
                st(3, vec![art("x", "2")], vec![art("y", "3")]),
            ],
        };
        let d = dot(&m, None);
        // in, x@1, x@2, y, además de 3 etapas
        assert_eq!(d.matches("shape=ellipse").count(), 4);
        assert!(d.contains("a1 -> s1"));
        assert!(d.contains("a2 -> s2"));
        assert!(d.contains("a3 -> s3"));
    }
}
