import timeit
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


def main():
    samples = wikipedia_samples(100)
    big_text = "\n\n".join(t for _, t in samples)
    n_chars = len(big_text)

    print(f"Corpus: {len(samples)} articles, {n_chars/1e6:.2f} MB")
    print(f"{'metric':<32} {'rust (ms)':>12} {'python (ms)':>14} {'speedup':>10}")
    print("-" * 72)
    for m in METRICS:
        rs_fn = getattr(textstat_rs, m)
        py_fn = getattr(textstat, m)
        rs_t = timeit.timeit(lambda: rs_fn(big_text), number=5) / 5
        py_t = timeit.timeit(lambda: py_fn(big_text), number=5) / 5
        speedup = py_t / rs_t if rs_t else float("inf")
        print(f"{m:<32} {rs_t*1000:>12.2f} {py_t*1000:>14.2f} {speedup:>10.2f}x")


if __name__ == "__main__":
    main()
