"""Compare per-metric runtime between textstat (python) and textstat_rs.

Both sides are measured the same way: in a fresh process, a warmup call on a
throwaway text pays any lazy resource load (reported separately as init), then
the corpus call is timed. Without that split the python numbers absorb ~380ms
of dictionary loading and the speedups come out roughly 3x too flattering.

Python needs a fresh process per sample because textstat lru_caches its metric
functions, so a second call on the same text returns instantly. The rust
extension does no such caching, so its samples run in one process.
"""

import multiprocessing as mp
import statistics
import time
import textstat_rs

from bench.corpora import corpus_for
from bench.report import write_report

# Per-metric time cap on the python side. Some py metrics are very slow on
# large inputs, so without this the benchmark can run for hours.
PY_TIMEOUT_SECONDS = 300

# Timed calls per side per metric; the median is reported.
SAMPLES = 5

CORPUS_ARTICLES = 100

# Distinct from the corpus, so textstat's lru_cache can't serve the real call
# from the warmup. Four sentences with some polysyllables, so metrics with an
# early return on short input (smog needs 3+ sentences) still load everything.
WARMUP_TEXT = (
    "The quick brown fox jumps over the lazy dog. "
    "Extraordinary circumstances necessitated immediate reconsideration. "
    "Readability metrics evaluate textual complexity systematically. "
    "Short words help as well."
)

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


def _py_worker(metric_name, text, q):
    """One python sample: warm up, then time a single call on the corpus."""
    import textstat

    fn = getattr(textstat, metric_name)
    t0 = time.perf_counter()
    fn(WARMUP_TEXT)
    init_s = time.perf_counter() - t0
    t1 = time.perf_counter()
    fn(text)
    q.put({"init_s": init_s, "samples": [time.perf_counter() - t1]})


def _rs_worker(metric_name, text, n_samples, q):
    """All rust samples: warm up once, then time n_samples calls."""
    import textstat_rs

    fn = getattr(textstat_rs, metric_name)
    t0 = time.perf_counter()
    fn(WARMUP_TEXT)
    init_s = time.perf_counter() - t0
    samples = []
    for _ in range(n_samples):
        t1 = time.perf_counter()
        fn(text)
        samples.append(time.perf_counter() - t1)
    q.put({"init_s": init_s, "samples": samples})


def _run_worker(target, args, timeout):
    """Run a worker in a spawned process; None if it overran or died."""
    ctx = mp.get_context("spawn")
    q = ctx.Queue()
    p = ctx.Process(target=target, args=(*args, q), daemon=True)
    p.start()
    p.join(timeout)
    if p.is_alive():
        p.terminate()
        p.join()
        return None
    try:
        return q.get_nowait()
    except Exception:
        return None


def _side(init_s, samples) -> dict:
    return {
        "init_ms": init_s * 1000,
        "warm_ms": statistics.median(samples) * 1000,
        "cold_ms": (init_s + statistics.median(samples)) * 1000,
        "samples_ms": [s * 1000 for s in samples],
    }


def measure(metric_name, big_text) -> dict:
    """Time one metric on both sides, median of SAMPLES timed calls each."""
    rs_raw = _run_worker(_rs_worker, (metric_name, big_text, SAMPLES), PY_TIMEOUT_SECONDS)
    if rs_raw is None:  # a panic or hang in the extension, not a slow metric
        raise RuntimeError(f"rust worker failed or overran on {metric_name}")
    rs = _side(rs_raw["init_s"], rs_raw["samples"])

    py_samples, py_init_s, timed_out = [], None, False
    for _ in range(SAMPLES):
        raw = _run_worker(_py_worker, (metric_name, big_text), PY_TIMEOUT_SECONDS)
        if raw is None:  # no point sampling further once it overruns the cap
            timed_out = True
            break
        py_init_s = raw["init_s"] if py_init_s is None else py_init_s
        py_samples += raw["samples"]

    if timed_out:
        # All we know is that python took longer than the cap, so the speedups
        # below are lower bounds computed off it.
        py = {"init_ms": None, "warm_ms": None, "cold_ms": None, "samples_ms": []}
        py_warm_ms = py_cold_ms = PY_TIMEOUT_SECONDS * 1000
    else:
        py = _side(py_init_s, py_samples)
        py_warm_ms, py_cold_ms = py["warm_ms"], py["cold_ms"]

    return {
        "metric": metric_name,
        "rust": rs,
        "python": py,
        "timed_out": timed_out,
        "speedup": (py_warm_ms / rs["warm_ms"]) if rs["warm_ms"] else None,
        "speedup_cold": (py_cold_ms / rs["cold_ms"]) if rs["cold_ms"] else None,
    }


def _lazy_worker(order, word, q):
    """First syllable_count in each locale, in one fresh process, in order."""
    import textstat_rs

    out = []
    for lang in order:
        t0 = time.perf_counter()
        textstat_rs.syllable_count(word, lang=lang)
        out.append({"lang": lang, "first_call_ms": (time.perf_counter() - t0) * 1000})
    q.put(out)


def dictionary_isolation(langs: list[str]) -> dict:
    """Evidence that each locale's dictionary is parsed lazily and separately.

    `pyphen.rs` keeps one `LazyLock` per dictionary and claims that scoring in
    one locale never pays to parse the other's ~107 KB of patterns. That is not
    directly observable from Python, but its consequence is: if the two are
    parsed independently and on demand, the *second* locale used in a process
    pays a fresh, measurable parse on its first call. Were they parsed together
    up front, that second first-call would be free.

    So `sequential[0]` covers cmudict plus one dictionary, and `sequential[1:]`
    is the marginal cost of each additional dictionary -- which is what a
    single-locale process never pays.
    """
    # Not in cmudict, so the lookup has to fall through to pyphen.
    word = "zzyzxvwgqk"
    sequential = _run_worker(_lazy_worker, (langs, word), PY_TIMEOUT_SECONDS)
    standalone = {
        lang: _run_worker(_lazy_worker, ([lang], word), PY_TIMEOUT_SECONDS)
        for lang in langs
    }
    return {
        "word": word,
        "sequential": sequential,
        "standalone": {
            lang: (res[0]["first_call_ms"] if res else None)
            for lang, res in standalone.items()
        },
    }


def format_cells(row: dict) -> dict:
    """Display strings for one row, shared by the terminal table and markdown."""

    def speedup(key):
        if row[key] is None:
            return "inf"
        return f">{row[key]:.0f}x" if row["timed_out"] else f"{row[key]:.2f}x"

    def ms(side, key):
        value = row[side][key]
        if value is None:
            return f">{PY_TIMEOUT_SECONDS * 1000:.0f}" if key == "warm_ms" else "-"
        return f"{value:.2f}"

    return {
        "rust": ms("rust", "warm_ms"),
        "rust_init": ms("rust", "init_ms"),
        "python": ms("python", "warm_ms"),
        "python_init": ms("python", "init_ms"),
        "speedup": speedup("speedup"),
        "speedup_cold": speedup("speedup_cold"),
    }


def main():
    # Timings are taken in the default locale only. The locale picks which
    # hyphenation dictionary backs the syllable fallback, and the two are the
    # same size and shape, so running the whole suite per locale would roughly
    # double the runtime for numbers expected to match. What *is* locale
    # specific -- the cost of loading each dictionary -- is measured separately
    # by `dictionary_isolation` below.
    lang = textstat_rs.get_lang()
    samples = corpus_for(lang, CORPUS_ARTICLES)
    big_text = "\n\n".join(t for _, t in samples)
    n_chars = len(big_text)

    print(
        f"Corpus: {len(samples)} articles, {n_chars/1e6:.2f} MB, lang {lang}  "
        f"(median of {SAMPLES}, py timeout/metric: {PY_TIMEOUT_SECONDS}s)"
    )
    header = (
        f"{'metric':<30} {'rs warm':>10} {'py warm':>10} {'speedup':>9} "
        f"{'rs init':>9} {'py init':>9} {'speedup+init':>13}"
    )
    print(header)
    print("-" * len(header))

    rows = []
    for m in METRICS:
        row = measure(m, big_text)
        rows.append(row)
        c = format_cells(row)
        print(
            f"{m:<30} {c['rust']:>10} {c['python']:>10} {c['speedup']:>9} "
            f"{c['rust_init']:>9} {c['python_init']:>9} {c['speedup_cold']:>13}",
            flush=True,  # so a redirected run streams instead of dumping at the end
        )

    isolation = dictionary_isolation(textstat_rs.supported_langs())
    print("\nFirst syllable_count per locale, one process, in order:")
    for entry in isolation["sequential"] or []:
        print(f"  {entry['lang']:<8} {entry['first_call_ms']:>8.1f} ms")

    out = write_report(
        "perf",
        {
            "corpus": {
                "source": "wikipedia",
                "articles": len(samples),
                "chars": n_chars,
            },
            "lang": lang,
            "py_timeout_seconds": PY_TIMEOUT_SECONDS,
            "samples": SAMPLES,
            "rows": rows,
            "dictionary_isolation": isolation,
        },
    )
    print(f"\nwrote {out}")


if __name__ == "__main__":
    main()
