# temporarily override lru_cache to test non-cached py performance (get a sense of speedup potential once rust caching too)
import functools
import multiprocessing as mp
import time
import timeit
from bench.corpora import wikipedia_samples

_real_lru = functools.lru_cache


def _no_op_lru(*args, **kwargs):
    def decorator(fn):
        @functools.wraps(fn)
        def wrapper(*a, **kw):
            return fn(*a, **kw)

        wrapper.cache_clear = lambda: None
        wrapper.cache_info = lambda: None
        wrapper.__wrapped__ = fn
        return wrapper

    # Support both forms: @lru_cache and @lru_cache(maxsize=128)
    if args and callable(args[0]) and not kwargs:
        return decorator(args[0])
    return decorator


# functools.lru_cache = _no_op_lru

import textstat
import textstat_rs

# functools.lru_cache = (
#     _real_lru  # revert post-import, textstat should use faked passthrough version
# )

# Per-metric time cap on the python side. Some uncached py metrics are
# very slow on large inputs, so without this the benchmark can run for hours.
PY_TIMEOUT_SECONDS = 300


def _py_time_once(metric_name, text, q):
    fn = getattr(textstat, metric_name)
    t0 = time.perf_counter()
    fn(text)
    q.put(time.perf_counter() - t0)


def time_py_with_timeout(metric_name, text, timeout):
    """Run one python call in a subprocess, return elapsed seconds or None if it overran."""
    ctx = mp.get_context("spawn")
    q = ctx.Queue()
    p = ctx.Process(target=_py_time_once, args=(metric_name, text, q), daemon=True)
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


def main():
    samples = wikipedia_samples(100)
    big_text = "\n\n".join(t for _, t in samples)
    n_chars = len(big_text)

    print(
        f"Corpus: {len(samples)} articles, {n_chars/1e6:.2f} MB  (py timeout/metric: {PY_TIMEOUT_SECONDS}s)"
    )
    print(f"{'metric':<32} {'rust (ms)':>12} {'python (ms)':>14} {'speedup':>12}")
    print("-" * 74)
    for m in METRICS:
        rs_fn = getattr(textstat_rs, m)
        rs_t = timeit.timeit(lambda: rs_fn(big_text), number=5) / 5
        py_t = time_py_with_timeout(m, big_text, PY_TIMEOUT_SECONDS)
        if py_t is None:
            py_str = f">{PY_TIMEOUT_SECONDS * 1000:.0f}"
            speedup_str = f">{PY_TIMEOUT_SECONDS / rs_t:.0f}x" if rs_t else "inf"
        else:
            py_str = f"{py_t * 1000:.2f}"
            speedup_str = f"{py_t / rs_t:.2f}x" if rs_t else "inf"
        print(f"{m:<32} {rs_t * 1000:>12.2f} {py_str:>14} {speedup_str:>12}")


if __name__ == "__main__":
    main()
