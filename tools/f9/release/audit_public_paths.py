#!/usr/bin/env python3
import argparse
import json
import subprocess
from pathlib import Path

HISTORICAL_PREFIXES = (
    "audits/",
    "provenance/",
)

LEGACY_PREFIXES = (
    "workloads/literature-pipeline/harness/legacy/",
)

ACTIVE_PREFIXES = (
    ".github/",
    "src/",
    "tools/",
    "scripts/",
    "tests/",
    "experiments/",
    "workloads/",
)

PUBLIC_ALWAYS_REVIEW_PREFIXES = (
    ".github/",
)

PUBLIC_ALWAYS_REVIEW_FILES = {
    "README.md",
    "Cargo.toml",
    "Makefile",
    "Dockerfile",
    "compose.yml",
    "docker-compose.yml",
}

def frozen_manifest_paths(repo):
    result = set()

    for manifest in sorted((repo / "provenance").rglob("*.sha256")):
        try:
            lines = manifest.read_text(encoding="utf-8").splitlines()
        except UnicodeDecodeError:
            continue

        for line in lines:
            if not line.strip():
                continue

            parts = line.split(None, 1)
            if len(parts) != 2:
                continue

            raw = parts[1].lstrip("*").strip()
            candidate = Path(raw)

            if candidate.is_absolute():
                try:
                    candidate = candidate.resolve().relative_to(repo)
                except (ValueError, OSError):
                    frozen_root = "/home/c-lara/EnvMorph/"
                    if raw.startswith(frozen_root):
                        candidate = Path(raw[len(frozen_root):])
                    else:
                        continue

            normalized = candidate.as_posix()
            if normalized and normalized != ".":
                result.add(normalized)

    return result

def git(repo, *args):
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--json-output", required=True)
    parser.add_argument("--text-output", required=True)
    args = parser.parse_args()

    repo = Path(args.repo_root).resolve()
    frozen_paths = frozen_manifest_paths(repo)

    proc = git(repo, "grep", "-n", "-F", "/home/c-lara")

    if proc.returncode not in (0, 1):
        raise SystemExit("git grep falló")

    rows = []

    for raw in proc.stdout.splitlines():
        if not raw.strip():
            continue

        parts = raw.split(":", 2)
        if len(parts) != 3:
            continue

        path, line_raw, text = parts
        line = int(line_raw)

        if path.startswith(HISTORICAL_PREFIXES):
            classification = "historical_evidence"
            blocker = False
        elif path.startswith(LEGACY_PREFIXES):
            classification = "legacy_preserved_artifact"
            blocker = False
        elif (
            path in PUBLIC_ALWAYS_REVIEW_FILES
            or path.startswith(PUBLIC_ALWAYS_REVIEW_PREFIXES)
        ):
            classification = "public_entrypoint_requires_review"
            blocker = True
        elif path in frozen_paths:
            classification = "frozen_scientific_artifact"
            blocker = False
        elif path.startswith(ACTIVE_PREFIXES):
            classification = "active_unfrozen_public_surface"
            blocker = True
        elif path.startswith("docs/"):
            classification = "documentation_requires_review"
            blocker = True
        else:
            classification = "unknown_requires_review"
            blocker = True

        rows.append({
            "path": path,
            "line": line,
            "classification": classification,
            "blocker": blocker,
            "text": text.strip(),
        })

    blockers = [row for row in rows if row["blocker"]]

    result = {
        "schema_version": "f9.public_path_audit.v1",
        "absolute_home": "/home/c-lara",
        "occurrences": len(rows),
        "blockers": len(blockers),
        "rows": rows,
        "frozen_manifest_path_count": len(frozen_paths),
        "review_policy": {
            "historical_evidence": "preserve",
            "legacy_preserved_artifact": "preserve",
            "frozen_scientific_artifact":
                "preserve_and_document",
            "public_entrypoint_requires_review": "block",
            "active_unfrozen_public_surface": "block",
            "documentation_requires_review": "block",
            "unknown_requires_review": "block",
        },
    }

    Path(args.json_output).write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    lines = [
        "ENVMORPH F9.10 PUBLIC PATH AUDIT",
        "",
        f"Absolute path: {result['absolute_home']}",
        f"Occurrences: {result['occurrences']}",
        f"Blockers: {result['blockers']}",
        "",
    ]

    for row in rows:
        lines.append(
            f"{row['classification']}\t"
            f"{row['path']}:{row['line']}\t"
            f"{row['text']}"
        )

    if not rows:
        lines.append("No absolute home paths found.")

    lines += [
        "",
        "BLOCKERS ONLY",
        "",
    ]

    if blockers:
        for row in blockers:
            lines.append(
                f"{row['classification']}\t"
                f"{row['path']}:{row['line']}\t"
                f"{row['text']}"
            )
    else:
        lines.append("none")

    Path(args.text_output).write_text(
        "\n".join(lines) + "\n",
        encoding="utf-8",
    )

    print(f"Absolute home occurrences: {len(rows)}")
    print(f"Public path blockers: {len(blockers)}")

    if blockers:
        for row in blockers:
            print(
                "BLOCKER:",
                f"{row['path']}:{row['line']}",
                row["classification"],
            )
        raise SystemExit(3)

if __name__ == "__main__":
    main()
