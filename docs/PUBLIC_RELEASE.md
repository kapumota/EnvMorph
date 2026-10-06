### Public release

#### Release state

Package version: `0.1.0`
Origin configured: `True`
Main branch exists: `True`
Fast-forward from main possible: `True`
Public ready after F9.10: `True`

#### Required order

After F9.10 PASS:

1. inspect the F9.10 commit and public path audit;
2. integrate `fase9/hermeticidad-conductual` into `main`;
3. push `main` to the configured origin;
4. change repository visibility to Public in GitHub;
5. create an annotated release tag only after the public tree is verified;
6. create the GitHub release from that tag if desired.

F9.10 itself does not push, tag, create a release or change visibility.

#### Public blockers

- none

#### Raw evidence

Raw evidence under `.envmorph` is intentionally not part of the Git repository.
F9.10 records available raw manifest hashes as provenance anchors.
A future artifact package may distribute raw evidence separately if required.

#### Frozen host-specific paths

Absolute `/home/c-lara` paths retained in provenance manifests, audits and legacy fixtures are historical evidence.
Files covered by earlier scientific provenance manifests are preserved byte-for-byte even when they contain original-host paths.
This includes historical acquisition drivers and audit probes.
They describe the original experimental host and are not the portable public entrypoint.
Rewriting them during F9.10 would invalidate earlier scientific manifests.
Any new public execution workflow must use repository-relative paths without rewriting frozen scientific artifacts.
