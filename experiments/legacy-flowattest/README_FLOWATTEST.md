# FlowAttest

Stage-level provenance recording and first-divergence localization for Unix pipelines.

You wrap each stage of a shell pipeline with `flowattest stage`. FlowAttest records the
command, the executable (content hash and version), the script files it executes, and the
SHA-256 of every declared input and output. Given two runs, `flowattest diff` finds the first
stage whose output diverges and classifies why. `flowattest isolate` replays the confounded
stages on identical inputs to separate a stage's own sensitivity from upstream propagation.

Status. v0.1, explicit instrumentation only. Zero dependencies (std only), Linux.
Transparent tracing (ptrace) is the planned next step and is not implemented.

## Quick start

    cargo build --release
    make demo                # records benchmarks/literature-pipeline twice, diffs, graphs
    make check               # 23 unit tests + 37 integration checks
    make experiments         # mutation experiment, writes results/rq2_localization.csv

    export FLOWATTEST_HOME=$PWD/.flowattest
    run=$(flowattest begin --map "$PWD/out=@out" --map "$PWD=@src")
    export FLOWATTEST_RUN=$run
    flowattest stage --name clean --input raw.csv --stdout out/clean.tsv -- awk -f clean.awk raw.csv
    flowattest stage --name dedup --input out/clean.tsv --stdout out/dedup.tsv -- sort -u out/clean.tsv
    flowattest end

    flowattest inspect run-001
    flowattest diff run-001 run-002          # exit 0 identical, 1 diverged
    flowattest isolate run-001 run-002       # replay stages whose executable differs
    flowattest graph run-001 --diff run-002 | dot -Tsvg > provenance.svg
    flowattest verify run-001                # re-hash artifacts on disk

`--map PREFIX=NAME` gives directories stable logical names, so runs recorded in different
checkouts or machines compare cleanly. Scripts run by `awk -f FILE` or `bash FILE` are
recorded automatically as code inputs.

## What diff reports

Each stage is classified by comparing run A and run B.

| Status | Meaning |
|---|---|
| identical | nothing recorded differs |
| absorbed | inputs, command or executable differ but outputs are identical |
| root | outputs differ and every input is identical or changed from outside the pipeline |
| propagated | outputs differ only because an upstream stage changed its inputs |
| confounded | outputs differ, the stage has its own change and also received upstream-changed inputs |

The first root stage is the localization result. A cause list follows: `code`, `command`,
`toolchain`, `data`, `environment` (candidate only), or `unexplained`. For `unexplained`
stages FlowAttest scans the recorded scripts for `systime()`, `rand()`, `$RANDOM`, `mktemp`,
`date` and similar, and prints file and line.

## Results so far (preliminary, self-authored corpus)

Workload. `benchmarks/literature-pipeline`, 18 recorded stages (bash, awk, sort, cp).

Mutation experiment, `make experiments REPEATS=10`, reference data in
`experiments/reference_rq2_localization.csv`.

| Family | Mutations | Runs | Localized correctly |
|---|---:|---:|---:|
| DATA (row added, quoted comma, altered rows) | 5 | 50 | 50 |
| CODE (edited scripts) | 4 | 40 | 40 |
| ENV (gawk instead of mawk) | 1 | 10 | 10 |
| NOND (wall clock) | 1 | 10 | 10 |
| Controls (no observable effect) | 2 | 20 | 0 false divergences |

Expected stage per mutation is declared by the author. Whether a mutation is observable is
decided by an independent byte comparison of the two output trees, so a mutation with no
effect is scored as a control, not as a miss. Read these numbers as a regression suite for
the tool, not as evidence of generality. See `docs/DESIGN.md`, section Threats.

gawk against mawk on the same inputs. `diff` localizes the divergence to `merge_sources`
(`for (d in seen)` iteration order) but leaves 14 of 18 stages as candidates because 13 are
confounded. `isolate` replays them on identical inputs. 5 of 16 replayable stages are
intrinsically sensitive to the awk implementation (`merge_sources`, `provenance_graph`,
`zettel_generator`, `contradiction_map`, `bias_audit`), 10 are not, and 1 cannot be replayed
because its input is a directory.

Found while building this. `zettel_generator.awk` stamped every note with `systime()`, so
two otherwise identical runs diverged whenever they crossed a second boundary. FlowAttest
reported it as `unexplained` and the static hint pointed at the line. The workload now honors
`SOURCE_DATE_EPOCH`; default behavior is unchanged.

## Layout

    src/            sha256, json, model, store, env, stage, diff, replay, graph, report, main
    tests/          integration.sh (37 checks)
    benchmarks/     literature-pipeline with pipeline_flowattest.sh (instrumented driver)
    experiments/    run_mutations.sh, reference results
    docs/DESIGN.md  model, algorithm, limits, research plan

## Limits

Explicit instrumentation: undeclared inputs and outputs are invisible. Directory artifacts
are hashed but not snapshotted, so stages with directory inputs cannot be replayed. Files over
8 MiB are hashed but not snapshotted. No network, mmap or concurrent-writer provenance. Linux
only. Replay compares bytes, so any embedded absolute path or timestamp makes a stage
non-reproducible (reported as such).
