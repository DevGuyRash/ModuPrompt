#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTEXT_DIR = ROOT / "context"
REF_REGISTRY = CONTEXT_DIR / "references.md"

REF_DEF_RE = re.compile(r"^\[([^\]]+)\]:\s+")
INLINE_LINK_RE = re.compile(r"!?\[[^\]]+\]\(([^)]+)\)")
REF_LINK_RE = re.compile(r"\[([^\]]+)\]\[([^\]]*)\]")


def iter_non_code_lines(text: str):
    in_fence = False
    fence = None
    in_indented = False
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            marker = stripped[:3]
            if not in_fence:
                in_fence = True
                fence = marker
            elif marker == fence:
                in_fence = False
                fence = None
            continue
        if in_fence:
            continue
        if in_indented:
            if line.startswith("    ") or line.startswith("\t"):
                continue
            in_indented = False
        if line.startswith("    ") or line.startswith("\t"):
            in_indented = True
            continue
        yield line


def load_canonical_labels() -> set[str]:
    if not REF_REGISTRY.exists():
        print(f"error: missing reference registry at {REF_REGISTRY}")
        sys.exit(1)
    labels = set()
    for line in REF_REGISTRY.read_text(encoding="utf-8").splitlines():
        match = REF_DEF_RE.match(line.strip())
        if match:
            labels.add(match.group(1).strip())
    return labels


def extract_ref_definitions(lines: list[str]) -> set[str]:
    defs = set()
    for line in lines:
        match = REF_DEF_RE.match(line.strip())
        if match:
            defs.add(match.group(1).strip())
    return defs


def extract_used_reference_labels(lines: list[str], canonical: set[str], defined: set[str]) -> set[str]:
    used: set[str] = set()
    for line in lines:
        stripped = line.strip()
        if REF_DEF_RE.match(stripped):
            continue
        for match in REF_LINK_RE.finditer(line):
            text, label = match.groups()
            label = label.strip() if label.strip() else text.strip()
            if label:
                used.add(label)
        # Shortcut reference links: [label] with no inline/ref link syntax.
        idx = 0
        while True:
            start = line.find("[", idx)
            if start == -1:
                break
            if start > 0 and line[start - 1] == "!":
                idx = start + 1
                continue
            end = line.find("]", start + 1)
            if end == -1:
                break
            label = line[start + 1 : end].strip()
            next_char = line[end + 1 : end + 2]
            if not label or next_char in ("(", "[", ":"):
                idx = end + 1
                continue
            if label in canonical or label in defined:
                used.add(label)
            idx = end + 1
    return used


def resolve_link_target(doc: Path, target: str) -> Path | None:
    target = target.strip()
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1].strip()
    if not target or target.startswith("#"):
        return None
    if target.startswith("//"):
        return None
    if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target):
        return None

    path_part = target.split("#", 1)[0].split("?", 1)[0].strip()
    if not path_part:
        return None

    if path_part.startswith("/"):
        candidate = ROOT / path_part.lstrip("/")
    else:
        candidate = (doc.parent / path_part)

    return candidate


def check_internal_links(doc: Path, lines: list[str]) -> list[str]:
    errors: list[str] = []
    for line in lines:
        for match in INLINE_LINK_RE.finditer(line):
            target = match.group(1)
            candidate = resolve_link_target(doc, target)
            if candidate is None:
                continue
            if candidate.exists():
                continue
            if candidate.suffix == "":
                if candidate.with_suffix(".md").exists():
                    continue
            if target.endswith("/"):
                if (candidate / "index.md").exists():
                    continue
            errors.append(f"{doc.relative_to(ROOT)}: broken link -> {target}")
    return errors


def main() -> int:
    if not CONTEXT_DIR.exists():
        print(f"error: missing context directory at {CONTEXT_DIR}")
        return 1

    canonical = load_canonical_labels()
    docs = sorted(CONTEXT_DIR.rglob("*.md"))
    for extra in (ROOT / "README.md", ROOT / "CONTRIBUTING.md"):
        if extra.exists():
            docs.append(extra)

    link_errors: list[str] = []
    reference_errors: list[str] = []

    for doc in docs:
        text = doc.read_text(encoding="utf-8")
        lines = list(iter_non_code_lines(text))

        link_errors.extend(check_internal_links(doc, lines))

        defined = extract_ref_definitions(lines)
        used = extract_used_reference_labels(lines, canonical, defined)

        missing_defs = sorted(label for label in used if label in canonical and label not in defined)
        unknown_defs = sorted(label for label in defined if label not in canonical)
        unknown_used = sorted(label for label in used if label not in canonical)

        if missing_defs:
            reference_errors.append(
                f"{doc.relative_to(ROOT)}: missing reference definitions for {', '.join(missing_defs)}"
            )
        if unknown_defs:
            reference_errors.append(
                f"{doc.relative_to(ROOT)}: defines non-canonical references {', '.join(unknown_defs)}"
            )
        if unknown_used:
            reference_errors.append(
                f"{doc.relative_to(ROOT)}: uses non-canonical references {', '.join(unknown_used)}"
            )

    if link_errors:
        print("Broken internal links detected:")
        for err in link_errors:
            print(f"- {err}")

    if reference_errors:
        print("\nReference policy violations detected:")
        for err in reference_errors:
            print(f"- {err}")

    if link_errors or reference_errors:
        return 1

    print("Docs checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
