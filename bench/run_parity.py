"""Compare textstat-rs against textstat, once per supported locale.

    uv run python -m bench.run_parity                    # every supported locale
    uv run python -m bench.run_parity --locales en_GB    # just one
    uv run python -m bench.run_parity --samples 100      # quick smoke run

Locales come from `textstat_rs.supported_langs()` rather than a list kept here,
so adding one to the extension brings it into the benchmark with no edit to
this file.
"""

import argparse
import csv
from collections import defaultdict
from pathlib import Path

import textstat
import textstat_rs

from bench.corpora import corpus_for, lang_root
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

# Metrics returning a string rather than a number
# Kept out of METRICS where figure is absolute delta
STRING_METRICS = [
    "text_standard",
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


# What one locale's run produces: absolute deltas per numeric metric, and
# match/no-match per string metric
# Two accumulators because the two cannot be
# summarised the same way.
LocaleResults = tuple[dict[str, list[float]], dict[str, list[bool]]]


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


def _string_section(per_string: dict[str, list[bool]]) -> list[dict]:
    """Exact-match counts for the string-valued metrics."""
    out = []
    for m in STRING_METRICS:
        matches = per_string.get(m)
        if not matches:
            continue
        exact = sum(matches)
        out.append(
            {
                "metric": m,
                "n": len(matches),
                "exact": exact,
                "exact_pct": exact / len(matches) * 100,
            }
        )
    return out


def _string_table(per_string: dict[str, list[bool]]) -> list[str]:
    lines = [
        "## String metrics",
        "",
        "| metric | n | exact | percentage |",
        "| --- | --- | --- | --- |",
    ]
    for r in _string_section(per_string):
        lines.append(
            f"| {r['metric']} | {r['n']} | {r['exact']} | {r['exact_pct']:.2f}% |"
        )
    lines.append("")
    return lines


def _bucket_table(deltas: list[float]) -> str:
    lines = ["| bucket | count | percentage |", "| --- | --- | --- |"]
    for b in _buckets(deltas):
        lines.append(f"| {b['label']} | {b['count']} | {b['pct']:.2f}% |")
    return "\n".join(lines)


def _locale_payload(
    per_metric: dict[str, list[float]],
    per_string: dict[str, list[bool]],
    lang: str,
) -> dict:
    """One locale's summary, in a form update_readme can render."""
    metric_deltas = [d for m in METRICS for d in per_metric.get(m, [])]
    buckets = _buckets(metric_deltas)
    exact = next(b["pct"] for b in buckets if b["label"] == "== 0")

    def section(names):
        return [{"metric": m, **_stats(per_metric[m])} for m in names if per_metric.get(m)]

    return {
        "lang": lang,
        "corpus_root": lang_root(lang),
        "metrics": section(METRICS),
        "primitives": section(PRIMITIVES),
        "strings": _string_section(per_string),
        "overall": {**_stats(metric_deltas), "exact_match_pct": exact},
        "buckets": buckets,
    }


def _payload(
    per_locale: dict[str, LocaleResults],
    n_samples: int,
    max_chars: int,
    default_lang: str,
) -> dict:
    """One artifact per commit, with locales nested inside it.

    Nested rather than a file per locale so every locale in a comparison is
    guaranteed to have come from the same build, and so adding a locale does
    not multiply artifacts.
    """
    locales = {
        lang: _locale_payload(pm, ps, lang) for lang, (pm, ps) in per_locale.items()
    }

    # Combined figure for the README headline. Metrics only, matching the
    # per-locale `overall`, so the two are directly comparable.
    combined_deltas = [
        d for pm, _ps in per_locale.values() for m in METRICS for d in pm.get(m, [])
    ]
    combined_buckets = _buckets(combined_deltas)

    return {
        "corpus": {"source": "wikipedia", "samples": n_samples, "max_chars": max_chars},
        "default_lang": default_lang,
        "locales": locales,
        "combined": {
            **_stats(combined_deltas),
            "exact_match_pct": next(
                b["pct"] for b in combined_buckets if b["label"] == "== 0"
            ),
        },
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


def _write_summary(
    path: Path, sha: str, per_locale: dict[str, LocaleResults]
) -> None:
    # Overall stats cover metrics only, to keep the historical baseline comparable.
    lines = [f"# Parity summary ({sha})", ""]
    for lang, (per_metric, per_string) in per_locale.items():
        metric_deltas = [d for m in METRICS for d in per_metric.get(m, [])]
        lines += [f"# {lang}", ""]
        lines += _section_table("Per metric", METRICS, per_metric)
        lines += _section_table("Primitive counts", PRIMITIVES, per_metric)
        lines += _string_table(per_string)
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

CSV_HEADER = [
    "lang",
    "kind",
    "metric",
    "sample_id",
    "rs",
    "py",
    "abs_delta",
    "rel_delta",
    "text",
]


def _score_locale(lang: str, samples, writer) -> LocaleResults:
    """Every metric over every sample, in one locale."""
    textstat.set_lang(lang)
    textstat_rs.set_lang(lang)

    per_metric: dict[str, list[float]] = defaultdict(list)
    per_string: dict[str, list[bool]] = defaultdict(list)

    for sample_id, text in samples:
        for kind, names in (("metric", METRICS), ("primitive", PRIMITIVES)):
            for m in names:
                rs = getattr(textstat_rs, m)(text)
                py = getattr(textstat, m)(text)
                abs_d = abs(rs - py)
                rel_d = abs(abs_d / py if py else 0.0)
                per_metric[m].append(abs_d)
                writer.writerow(
                    [
                        lang,
                        kind,
                        m,
                        sample_id,
                        f"{rs:.6f}",
                        f"{py:.6f}",
                        f"{abs_d:.6f}",
                        f"{rel_d:.6f}",
                        # Text only if mismatch (for debugging)
                        text if abs_d else "",
                    ]
                )

        for m in STRING_METRICS:
            rs = getattr(textstat_rs, m)(text)
            py = getattr(textstat, m)(text)
            matched = rs == py
            per_string[m].append(matched)
            writer.writerow(
                [
                    lang,
                    "string",
                    m,
                    sample_id,
                    rs,
                    py,
                    # No delta, so this column has a 0/1 mismatch flag
                    "0" if matched else "1",
                    "",
                    "" if matched else text,
                ]
            )

    return per_metric, per_string


def _parse_args(argv=None):
    supported = textstat_rs.supported_langs()
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument(
        "--locales",
        nargs="+",
        default=supported,
        metavar="TAG",
        help=f"locales to compare (default: all supported, {', '.join(supported)})",
    )
    p.add_argument("--samples", type=int, default=CORPUS_SAMPLES)
    p.add_argument("--max-chars", type=int, default=CORPUS_MAX_CHARS)
    args = p.parse_args(argv)

    unknown = [tag for tag in args.locales if tag not in supported]
    if unknown:
        p.error(
            f"unsupported locale(s): {', '.join(unknown)}. "
            f"Supported: {', '.join(supported)}"
        )
    return args


def main(argv=None):
    args = _parse_args(argv)
    sha = git_sha()
    results_dir = Path(__file__).parent.parent / "results"
    results_dir.mkdir(exist_ok=True)
    out = results_dir / f"parity_{sha}.csv"
    summary_out = results_dir / f"parity_{sha}.md"

    # Captured before the run, because scoring rewrites it.
    default_lang = textstat_rs.get_lang()
    per_locale: dict[str, LocaleResults] = {}

    try:
        with out.open("w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(CSV_HEADER)
            for lang in args.locales:
                samples = corpus_for(lang, args.samples, args.max_chars)
                print(f"scoring {lang} over {len(samples)} samples...")
                per_locale[lang] = _score_locale(lang, samples, w)
    finally:
        # `set_lang` is process-wide in both libraries; leaving the last locale
        # in place would quietly reinterpret anything that runs after this.
        textstat.set_lang(default_lang)
        textstat_rs.set_lang(default_lang)

    _write_summary(summary_out, sha, per_locale)
    report_out = write_report(
        "parity", _payload(per_locale, args.samples, args.max_chars, default_lang)
    )
    print(f"wrote {out}")
    print(f"wrote {summary_out}")
    print(f"wrote {report_out}")


if __name__ == "__main__":
    import nltk
    print(nltk.data.find("corpora/cmudict/cmudict"))
    main()
