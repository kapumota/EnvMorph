### F9 threats to validity

#### Observation completeness

F9 does not claim complete dependency observation.

The current backend combines a ptrace observer with `getenv` interposition.

Known blind spots include:

- direct reads of `environ` that bypass `getenv`;
- vDSO time reads;
- dependencies satisfied through pre-opened file descriptors;
- dependencies mediated through memory mappings or caches;
- syscalls and library mechanisms outside the frozen observer set.

eBPF was evaluated for feasibility but was not selected as the backend on the current host.

#### Observer effect

Observer effect is evaluated for the frozen external workload executions.

A preserved observer effect for those executions does not imply transparency for arbitrary programs or operating systems.

#### Paths

Path normalization is lexical.

EnvMorph does not claim filesystem-canonical identity equivalent to `realpath`.

Symlinks, mount aliases and namespace differences may therefore represent the same physical object with different logical resources.

#### Event identity and process relations

`event_id` does not include `ppid`, while exact capture deduplication does.

Process relations are observational and do not constitute a causal graph.

PID reuse and reparenting remain possible sources of ambiguity outside the bounded run model.

#### Interventions

Only two of 12,717 observed dependencies were eligible under the preregistered F8 factor space.

This very small eligibility fraction is a consequence of the conservative protocol.

It must not be interpreted as evidence that the remaining dependencies are behaviorally irrelevant.

#### External results

Both eligible external dependencies were `observed_only`.

This supports only the bounded statement that the frozen `tmpdir` interventions did not change the frozen observables in the two eligible cases.

It does not estimate population prevalence and does not support universal portability or hermeticity.

#### Causality

F9 uses controlled interventions and operational sufficiency.

It does not claim physical causality.

#### Ordering

Events from different observation sources do not have a claimed global total order.

#### Generalization

The external corpus remains the four F8 workloads.

No claim is made that the corpus represents Unix software in general.
