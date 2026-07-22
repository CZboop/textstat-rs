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
        f"python {env['python']} vs {PY_LABEL} {env['textstat']}, on {env['platform']}._"
    )


def parity_block(doc: dict) -> str:
    corpus, overall = doc["corpus"], doc["overall"]
    rows = [
        [r["metric"], f"{r['avg']:.6f}", f"{r['max']:.6f}"] for r in doc["metrics"]
    ]
    return "\n".join(
        [
            f"Scores are compared against `{PY_LABEL}` over {corpus['samples']} "
            f"{corpus['source'].capitalize()} articles "
            f"(up to {corpus['max_chars']} chars each): "
            f"**{overall['exact_match_pct']:.2f}% of {overall['n']} comparisons match exactly**, "
            f"and the largest disagreement anywhere is {overall['max']:.6f}.",
            "",
            md_table(["metric", "avg delta", "max delta"], rows),
            "",
            _provenance(doc),
        ]
    )


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
