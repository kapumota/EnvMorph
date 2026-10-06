### F9 reproducibility audit

#### Frozen result

F9.8C closed with:

```text
observed_candidates = 12717
eligible_candidates = 2
behaviorally_relevant = 0
observed_only = 2
decidable_candidates = 2
decidability_among_observed = 0.00015726979633561374
relevance_among_decidable = 0.0
```

The two external confirmatory plans were classified `observed_only`.

This means that the frozen `tmpdir` interventions produced no observable behavioral difference under the frozen F8 traces and F3/F4 oracles.

It does not establish that those workloads are universally hermetic.

#### Reproducibility checks

F9.9 verifies:

- all tracked F8 and F9 provenance manifests through F9.8C;
- Git ancestry from the external census through the result commit;
- `cargo fmt --check`, `cargo test` and release build;
- deterministic reexecution of the frozen F9.7 aggregator without rerunning workloads;
- F9.4 validation of the two frozen plans;
- deterministic regeneration of F9.6 bridge evidence from the stored F4 result;
- current raw evidence manifests when locally available.

No workload treatment is executed in F9.9.

#### Evidence levels

Tracked evidence is part of the repository and is covered by phase manifests.

Raw evidence under `.envmorph` is local experimental evidence and is not assumed to be available in a public clone.

The public artifact must therefore distinguish:

```text
tracked reproducibility evidence
local raw acquisition evidence
```

#### Historical raw manifests

F9.2 and F9.3 generated `RAW.sha256` before removing transient local binaries.

If those historical raw directories still exist, their manifests can reference files that were intentionally deleted afterward.

F9.9 records that condition rather than rewriting historical manifests.

This issue does not alter tracked F9.2 or F9.3 phase manifests.

#### Event identity

`DependencyEvent v1` freezes `event_id` without `ppid`, while exact capture deduplication includes `ppid`.

F9.9 does not silently change that schema.

It is retained as a documented limitation for a future schema revision.

#### Next gate

F9.10 is the scientific closure and release freeze.
