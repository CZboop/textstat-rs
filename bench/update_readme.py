"""Render the newest bench artifacts into README.md.

Reads results/parity_*.json and results/perf_*.json (newest of each, whichever
commit they came from) and replaces the marked blocks in the README. Run it
when a bench run produces numbers worth publishing:

    uv run python -m bench.update_readme
"""

import sys

from bench.report import latest_report, md_table, splice

PY_LABEL = "textstat"
RS_LABEL = "textstat-rs"


def _provenance(doc: dict) -> str:
    env = doc["environment"]
    return (
        f"_Generated from `{doc['sha']}` on {doc['generated'][:10]}, "
        f"python {env['python']} vs {PY_LABEL} {env['textstat']}._"
    )


def _as_locales(doc: dict) -> tuple[dict, dict, str]:
    """Return (locales, combined, default_lang) for any parity artifact.

    Artifacts predating the locale dimension keep `metrics`/`overall` at the
    top level. Reading one as a single implicit locale means an older file left
    in results/ still renders instead of raising KeyError.
    """
    if "locales" in doc:
        default = doc.get("default_lang") or next(iter(doc["locales"]))
        return doc["locales"], doc["combined"], default

    lang = doc.get("default_lang", "en_US")
    single = {k: doc[k] for k in ("metrics", "primitives", "overall", "buckets") if k in doc}
    return {lang: single}, doc["overall"], lang


def parity_block(doc: dict) -> str:
    corpus = doc["corpus"]
    locales, combined, default_lang = _as_locales(doc)
    tags = list(locales)
    # fallback if the default locale wasn't run
    if default_lang not in locales:
        default_lang = tags[0]

    scope = (
        f"in `{tags[0]}`"
        if len(tags) == 1
        else "across " + " and ".join(f"`{t}`" for t in tags)
    )

    parts = [
        f"Scores are compared against `{PY_LABEL}` over {corpus['samples']} "
        f"{corpus['source'].capitalize()} articles "
        f"(up to {corpus['max_chars']} chars each), {scope}: "
        f"**{combined['exact_match_pct']:.2f}% of {combined['n']} comparisons match exactly**, "
        f"and the largest disagreement anywhere is {combined['max']:.6f}.",
        "",
    ]

    # Per-locale summary only when there is more than one to compare
    if len(tags) > 1:
        parts += [
            md_table(
                ["locale", "comparisons", "exact match", "max delta"],
                [
                    [
                        t,
                        locales[t]["overall"]["n"],
                        f"{locales[t]['overall']['exact_match_pct']:.2f}%",
                        f"{locales[t]['overall']['max']:.6f}",
                    ]
                    for t in tags
                ],
            ),
            "",
        ]

    # String-valued metrics have no delta, so they get their own line rather
    # than a column in the tables above. Older artifacts have no "strings" key.
    string_rows = [
        (r["metric"], t, r)
        for t in tags
        for r in locales[t].get("strings", [])
    ]
    if string_rows:
        names = sorted({name for name, _t, _r in string_rows})
        per_name = []
        for name in names:
            results = ", ".join(
                f"{r['exact']}/{r['n']} in `{t}`"
                for n, t, r in string_rows
                if n == name
            )
            per_name.append(f"`{name}` matches exactly {results}")
        parts += [
            "Metrics returning a grade band rather than a number are compared as "
            "exact strings: " + "; ".join(per_name) + ".",
            "",
        ]

    # The per-metric breakdown is near-identical between locales, so only the
    # default one is shown in full -- collapsed, to keep this section short as
    # locales are added. The per-locale table above is what carries the
    # comparison.
    rows = [
        [r["metric"], f"{r['avg']:.6f}", f"{r['max']:.6f}"]
        for r in locales[default_lang]["metrics"]
    ]
    parts += [
        f"<details>",
        f"<summary>Per metric ({default_lang})</summary>",
        "",
        md_table(["metric", "avg delta", "max delta"], rows),
        "",
        "</details>",
        "",
        _provenance(doc),
    ]
    return "\n".join(parts)


def perf_block(doc: dict) -> str:
    corpus, samples = doc["corpus"], doc["samples"]
    cap_ms = doc["py_timeout_seconds"] * 1000
    rows = []
    for r in doc["rows"]:
        if r["timed_out"]:
            py, speedup = f">{cap_ms:.0f}", f">{r['speedup']:.0f}x"
        else:
            py, speedup = f"{r['python']['warm_ms']:.2f}", f"{r['speedup']:.2f}x"
        rows.append([r["metric"], f"{r['rust']['warm_ms']:.2f}", py, speedup])

    # Init is a per-process constant, so quoting the range beats a column of near
    # identical numbers; the syllable metrics load dictionaries, the rest don't.
    rs_init = max(r["rust"]["init_ms"] for r in doc["rows"])
    py_init = max(r["python"]["init_ms"] or 0 for r in doc["rows"])

    return "\n".join(
        [
            f"One call per metric over {corpus['articles']} concatenated "
            f"{corpus['source'].capitalize()} articles ({corpus['chars'] / 1e6:.2f} MB), "
            f"median of {samples}. Both libraries are warmed first, so these are "
            f"steady-state numbers with lazy resource loading excluded.",
            "",
            md_table(
                ["metric", f"{RS_LABEL} (ms)", f"{PY_LABEL} (ms)", "speedup"], rows
            ),
            "",
            f"On top of that, the first call in a fresh process pays a one-off load of "
            f"the syllable and word-list resources: up to {rs_init:.0f} ms for "
            f"`{RS_LABEL}` against {py_init:.0f} ms for `{PY_LABEL}`.",
            "",
            _provenance(doc),
        ]
    )


def main():
    wrote = []
    for section, render in (("parity", parity_block), ("perf", perf_block)):
        doc = latest_report(section)
        if doc is None:
            print(f"no results/{section}_*.json found, leaving that section alone")
            continue
        splice(section, render(doc))
        wrote.append(f"{section} (from {doc['sha']})")
    if not wrote:
        sys.exit("nothing to publish; run bench.run_parity / bench.run_perf first")
    print(f"updated README.md: {', '.join(wrote)}")


if __name__ == "__main__":
    main()
