import csv
from collections import defaultdict
from pathlib import Path

import textstat
import textstat_rs

from bench.corpora import wikipedia_samples
from bench.report import git_sha, write_report

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

# primitive counts, exposed on both textstat and textstat_rs under the same names
PRIMITIVES = [
    "syllable_count",
    "sentence_count",
    "lexicon_count",
    "char_count",
    "letter_count",
    "polysyllabcount",
    "miniword_count",
    "difficult_words",
]


def _summary_row(name: str, deltas: list[float]) -> str:
    avg = sum(deltas) / len(deltas)
    return f"| {name} | {len(deltas)} | {avg:.6f} | {min(deltas):.6f} | {max(deltas):.6f} |"


def _stats(deltas: list[float]) -> dict:
    return {
        "n": len(deltas),
        "avg": sum(deltas) / len(deltas),
        "min": min(deltas),
        "max": max(deltas),
    }


def _buckets(deltas: list[float]) -> list[dict]:
    # `== 0` is an exact-match count, so the float equality check is deliberate.
    thresholds = [("== 0", lambda d: d == 0.0)]
    thresholds += [(f"< {t}", (lambda t: lambda d: d < t)(t)) for t in (1, 2, 5, 10)]
    total = len(deltas)
    out = []
    for label, pred in thresholds:
        n = sum(1 for d in deltas if pred(d))
        out.append({"label": label, "count": n, "pct": (n / total * 100) if total else 0.0})
    return out


def _bucket_table(deltas: list[float]) -> str:
    lines = ["| bucket | count | percentage |", "| --- | --- | --- |"]
    for b in _buckets(deltas):
        lines.append(f"| {b['label']} | {b['count']} | {b['pct']:.2f}% |")
    return "\n".join(lines)


def _payload(per_metric: dict[str, list[float]], n_samples: int, max_chars: int) -> dict:
    """The same summary the markdown shows, in a form update_readme can render."""
    metric_deltas = [d for m in METRICS for d in per_metric.get(m, [])]
    buckets = _buckets(metric_deltas)
    exact = next(b["pct"] for b in buckets if b["label"] == "== 0")

    def section(names):
        return [{"metric": m, **_stats(per_metric[m])} for m in names if per_metric.get(m)]

    return {
        "corpus": {"source": "wikipedia", "samples": n_samples, "max_chars": max_chars},
        "metrics": section(METRICS),
        "primitives": section(PRIMITIVES),
        "overall": {**_stats(metric_deltas), "exact_match_pct": exact},
        "buckets": buckets,
    }


def _section_table(title: str, names: list[str], per_metric: dict[str, list[float]]) -> list[str]:
    lines = [
        f"## {title}",
        "",
        "| metric | n | avg | min | max |",
        "| --- | --- | --- | --- | --- |",
    ]
    for m in names:
        if per_metric.get(m):
            lines.append(_summary_row(m, per_metric[m]))
    lines.append("")
    return lines


def _write_summary(path: Path, sha: str, per_metric: dict[str, list[float]]) -> None:
    # Overall stats cover metrics only, to keep the historical baseline comparable.
    metric_deltas = [d for m in METRICS for d in per_metric.get(m, [])]
    lines = [f"# Parity summary ({sha})", ""]
    lines += _section_table("Per metric", METRICS, per_metric)
    lines += _section_table("Primitive counts", PRIMITIVES, per_metric)
    lines += [
        "## Overall (metrics)",
        "",
        "| n | avg | min | max |",
        "| --- | --- | --- | --- |",
        f"| {len(metric_deltas)} | {sum(metric_deltas) / len(metric_deltas):.6f} "
        f"| {min(metric_deltas):.6f} | {max(metric_deltas):.6f} |",
        "",
        "## Delta buckets (metrics)",
        "",
        _bucket_table(metric_deltas),
        "",
    ]
    path.write_text("\n".join(lines), encoding="utf-8")


CORPUS_SAMPLES = 1000
CORPUS_MAX_CHARS = 5000


def main():
    sha = git_sha()
    results_dir = Path(__file__).parent.parent / "results"
    results_dir.mkdir(exist_ok=True)
    out = results_dir / f"parity_{sha}.csv"
    summary_out = results_dir / f"parity_{sha}.md"
    samples = wikipedia_samples(CORPUS_SAMPLES, CORPUS_MAX_CHARS)

    per_metric: dict[str, list[float]] = defaultdict(list)

    with out.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            ["kind", "metric", "sample_id", "rs", "py", "abs_delta", "rel_delta", "text"]
        )
        for sample_id, text in samples:
            for kind, names in (("metric", METRICS), ("primitive", PRIMITIVES)):
                for m in names:
                    rs = getattr(textstat_rs, m)(text)
                    py = getattr(textstat, m)(text)
                    abs_d = abs(rs - py)
                    rel_d = abs(abs_d / py if py else 0.0)
                    per_metric[m].append(abs_d)
                    w.writerow(
                        [
                            kind,
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
    report_out = write_report(
        "parity", _payload(per_metric, len(samples), CORPUS_MAX_CHARS)
    )
    print(f"wrote {out}")
    print(f"wrote {summary_out}")
    print(f"wrote {report_out}")


if __name__ == "__main__":
    import nltk
    print(nltk.data.find("corpora/cmudict/cmudict"))
    main()
