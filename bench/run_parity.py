import csv
import subprocess
from collections import defaultdict
from pathlib import Path

import textstat
import textstat_rs

from bench.corpora import wikipedia_samples

METRICS = [
    "flesch_reading_ease",
    "flesch_kincaid_grade",
    "automated_readability_index",
    "coleman_liau_index",
    "dale_chall_readability_score",
    "gunning_fog",
    "smog_index",
    "linsear_write_formula",
    "mcalpine_eflaw",
    "spache_readability",
    "reading_time",
]


def _git_sha() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"], text=True
        ).strip()
    except Exception:
        return "nogit"


def _summary_row(name: str, deltas: list[float]) -> str:
    avg = sum(deltas) / len(deltas)
    return f"| {name} | {len(deltas)} | {avg:.6f} | {min(deltas):.6f} | {max(deltas):.6f} |"


def _bucket_table(deltas: list[float]) -> str:
    thresholds = [("== 0", lambda d: d == 0.0)]
    thresholds += [(f"< {t}", (lambda t: lambda d: d < t)(t)) for t in (1, 2, 5, 10)]
    total = len(deltas)
    lines = ["| bucket | count | percentage |", "| --- | --- | --- |"]
    for label, pred in thresholds:
        n = sum(1 for d in deltas if pred(d))
        pct = (n / total * 100) if total else 0.0
        lines.append(f"| {label} | {n} | {pct:.2f}% |")
    return "\n".join(lines)


def _write_summary(path: Path, sha: str, per_metric: dict[str, list[float]]) -> None:
    all_deltas = [d for ds in per_metric.values() for d in ds]
    lines = [
        f"# Parity summary ({sha})",
        "",
        "## Per metric",
        "",
        "| metric | n | avg | min | max |",
        "| --- | --- | --- | --- | --- |",
    ]
    for m in METRICS:
        if per_metric.get(m):
            lines.append(_summary_row(m, per_metric[m]))
    lines += [
        "",
        "## Overall",
        "",
        "| n | avg | min | max |",
        "| --- | --- | --- | --- |",
        f"| {len(all_deltas)} | {sum(all_deltas) / len(all_deltas):.6f} "
        f"| {min(all_deltas):.6f} | {max(all_deltas):.6f} |",
        "",
        "## Delta buckets (overall)",
        "",
        _bucket_table(all_deltas),
        "",
    ]
    path.write_text("\n".join(lines), encoding="utf-8")


def main():
    sha = _git_sha()
    results_dir = Path(__file__).parent.parent / "results"
    results_dir.mkdir(exist_ok=True)
    out = results_dir / f"parity_{sha}.csv"
    summary_out = results_dir / f"parity_{sha}.md"
    samples = wikipedia_samples(1000)

    per_metric: dict[str, list[float]] = defaultdict(list)

    with out.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["metric", "sample_id", "rs", "py", "abs_delta", "rel_delta", "text"])
        for sample_id, text in samples:
            for m in METRICS:
                rs = getattr(textstat_rs, m)(text)
                py = getattr(textstat, m)(text)
                abs_d = abs(rs - py)
                rel_d = abs(abs_d / py if py else 0.0)
                per_metric[m].append(abs_d)
                w.writerow(
                    [
                        m,
                        sample_id,
                        f"{rs:.6f}",
                        f"{py:.6f}",
                        f"{abs_d:.6f}",
                        f"{rel_d:.6f}",
                        text,
                    ]
                )

    _write_summary(summary_out, sha, per_metric)
    print(f"wrote {out}")
    print(f"wrote {summary_out}")


if __name__ == "__main__":
    import nltk
    print(nltk.data.find("corpora/cmudict/cmudict"))
    main()
