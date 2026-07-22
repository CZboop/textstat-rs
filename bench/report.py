"""Shared reporting helpers for the bench scripts.

Each bench run drops a machine-readable JSON artifact in results/, keyed by
commit it was run against. `bench.update_readme` later renders the newest
results into the README.
"""

import json
import platform
import subprocess
import sys
from datetime import datetime, timezone
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path

ROOT = Path(__file__).parent.parent
RESULTS = ROOT / "results"
README = ROOT / "README.md"


def git_sha() -> str:
    """Short HEAD sha, suffixed -dirty if the tree has uncommitted changes.

    Without the suffix a run against modified sources publishes numbers under a
    commit that does not contain the code that produced them.
    """
    try:
        sha = subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"], text=True
        ).strip()
        dirty = subprocess.check_output(["git", "status", "--porcelain"], text=True)
        return f"{sha}-dirty" if dirty.strip() else sha
    except Exception:
        return "nogit"


def _pkg_version(name: str) -> str:
    try:
        return version(name)
    except PackageNotFoundError:
        return "unknown"


def environment() -> dict:
    """Machine/library context, recorded so old numbers stay interpretable."""
    return {
        "platform": platform.platform(),
        "processor": platform.processor(),
        "python": platform.python_version(),
        "textstat": _pkg_version("textstat"),
        "textstat_rs": _pkg_version("textstat-rs"),
    }


def write_report(kind: str, payload: dict) -> Path:
    """Write results/<kind>_<sha>.json, stamped with sha, time and environment."""
    RESULTS.mkdir(exist_ok=True)
    sha = payload.get("sha") or git_sha()
    doc = {
        "kind": kind,
        "sha": sha,
        "generated": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "environment": environment(),
        **payload,
    }
    out = RESULTS / f"{kind}_{sha}.json"
    out.write_text(json.dumps(doc, indent=2), encoding="utf-8")
    return out


def latest_report(kind: str) -> dict | None:
    """Newest results/<kind>_*.json by mtime, or None if there are none."""
    candidates = sorted(
        RESULTS.glob(f"{kind}_*.json"), key=lambda p: p.stat().st_mtime, reverse=True
    )
    if not candidates:
        return None
    return json.loads(candidates[0].read_text(encoding="utf-8"))


def md_table(headers: list[str], rows: list[list[str]]) -> str:
    lines = ["| " + " | ".join(headers) + " |"]
    lines.append("| " + " | ".join("---" for _ in headers) + " |")
    for row in rows:
        lines.append("| " + " | ".join(str(c) for c in row) + " |")
    return "\n".join(lines)


def splice(section: str, body: str) -> None:
    """Replace the README block delimited by this section's marker comments."""
    start = f"<!-- bench:{section}:start -->"
    end = f"<!-- bench:{section}:end -->"
    text = README.read_text(encoding="utf-8")
    if start not in text or end not in text:
        sys.exit(
            f"README.md is missing the {start} / {end} markers; "
            "add them where the generated block should go."
        )
    head, rest = text.split(start, 1)
    _, tail = rest.split(end, 1)
    README.write_text(
        f"{head}{start}\n{body.strip()}\n{end}{tail}", encoding="utf-8"
    )
