# FlowAttest design notes

## Model

A run is an ordered list of stage records. A stage record holds the command (with
machine-specific prefixes rewritten to logical names), working directory, executable path,
executable SHA-256 and version string, input artifacts, output artifacts, stdout and stderr
redirection targets, exit code and duration. An artifact is a path, a kind (file, dir or
missing), a SHA-256 and a size. Directory artifacts hash the sorted list of relative paths and
file digests. A role (data or code) separates inputs that the stage reads from scripts that it
executes.

The run header stores kernel, architecture, locale and time variables, git commit and the
prefix-to-name map. `PATH` and `SHELL` are stored but never reported as a cause.

Store layout

    .flowattest/CURRENT
    .flowattest/runs/run-NNN/manifest.json
    .flowattest/runs/run-NNN/stages/NNN.json
    .flowattest/objects/<sha256>          snapshots of files up to 8 MiB (inputs and outputs)

## First divergence

Runs A and B must have the same stage names in the same order. For stage i let
`out_A(i)`, `out_B(i)` be output digests and `D` the set of paths whose digest already
differed at an earlier stage.

1. If outputs and exit code are equal, the stage is identical, or absorbed when something
   upstream of the output differs (input, command, executable, script).
2. Otherwise let `local` be a change in script, command or executable, `ext` the changed
   inputs not in `D`, `prop` the changed inputs in `D`.
3. `prop` only and no `local` and no `ext`: propagated.
4. `prop` together with `local` or `ext`: confounded.
5. Otherwise: root.

Paths in `D` are added after each stage that is root, confounded or propagated. The first
root stage is reported. Cause candidates are listed in this order: script, command,
executable, external input, propagated input. When none applies the stage is
`unexplained`, and environment variable differences are offered as candidates only.

## Isolation

Confounded stages cannot be resolved by comparison alone. For every stage whose executable
differs between A and B, `isolate` rebuilds a sandbox from the object store, rewrites logical
names to sandbox paths, applies the run's recorded locale and clock variables, and runs the
stage four ways.

1. A inputs, A executable. Must reproduce A outputs, otherwise not-reproducible.
2. B inputs, B executable. Must reproduce B outputs.
3. A inputs, B executable.
4. B inputs, A executable.

If 3 or 4 changes any recorded output the stage is sensitive to the executable swap. Only the
executable is swapped. Swapping a script is not implemented.

## Threats to validity (v0.1)

1. Ground truth is authored. The mutation corpus and the expected stage are written by the
   tool's author on the author's pipeline. Oracle for observability is independent (byte
   comparison), oracle for the expected stage is not. A fair RQ needs faults the author did
   not write, for example real bugs from public pipelines.
2. Explicit instrumentation makes localization close to a hash comparison. The research
   question that is not yet answered is how much provenance can be recovered without
   wrapping stages.
3. One workload with 18 stages, mostly awk filters. Results will not transfer to build
   systems or ETL without more workloads.
4. Hash equality is the only equivalence. Semantically equal outputs that differ in byte
   order are reported as divergence.
5. Replay is only as faithful as the recorded inputs, environment variables and executable.
   Libraries, kernel behavior and undeclared files are not captured.

## Roadmap

1. Transparent backend. ptrace `execve`, `openat`, `read`, `write`, `rename`, `close`
   events produce the same stage records. Reuse the process monitor from AgentGuard-FastPath.
   Define a stage as a maximal subtree of processes sharing the same declared `begin`/`end`
   markers or, without markers, one top-level `execve` chain. Open question.
2. Ground truth for the transparent backend is the explicit run recorded by this version.
   Metric is edge precision and recall of the inferred graph against it.
3. External fault corpus. Collect public pipelines with documented reproducibility bugs.
4. Script swap in `isolate`, directory snapshots, parallel stages (DAG instead of a list).
5. Only then decide on a venue. See the project discussion about EASE 2027, JOSS and journals.
