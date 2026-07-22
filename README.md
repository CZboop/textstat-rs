# Texstat-rs
Python bound, Rust backed port of the textstat textual analysis library.
Drop-in replacements for all metrics in US English.

## Parity

<!-- bench:parity:start -->
Scores are compared against `textstat` over 1000 Wikipedia articles (up to 5000 chars each): **99.98% of 11000 comparisons match exactly**, and the largest disagreement anywhere is 0.049566.

| metric | avg delta | max delta |
| --- | --- | --- |
| flesch_reading_ease | 0.000000 | 0.000000 |
| flesch_kincaid_grade | 0.000000 | 0.000000 |
| automated_readability_index | 0.000000 | 0.000000 |
| coleman_liau_index | 0.000000 | 0.000000 |
| dale_chall_readability_score | 0.000000 | 0.000000 |
| gunning_fog | 0.000050 | 0.049566 |
| smog_index | 0.000000 | 0.000000 |
| linsear_write_formula | 0.000000 | 0.000000 |
| mcalpine_eflaw | 0.000000 | 0.000000 |
| spache_readability | 0.000011 | 0.010657 |
| reading_time | 0.000000 | 0.000000 |

_Generated from `bbf8ad9` on 2026-07-22, python 3.12.4 vs textstat 0.7.13, on Windows-11-10.0.26200-SP0._
<!-- bench:parity:end -->

## Performance

<!-- bench:perf:start -->
One call per metric over 100 concatenated Wikipedia articles (0.46 MB), median of 5. Both libraries are warmed first, so these are steady-state numbers with lazy resource loading excluded.

| metric | textstat-rs (ms) | textstat (ms) | speedup |
| --- | --- | --- | --- |
| flesch_reading_ease | 37.76 | 136.10 | 3.60x |
| flesch_kincaid_grade | 34.58 | 128.76 | 3.72x |
| automated_readability_index | 16.78 | 43.69 | 2.60x |
| coleman_liau_index | 12.82 | 56.66 | 4.42x |
| dale_chall_readability_score | 27.00 | 168.03 | 6.22x |
| gunning_fog | 28.07 | 178.12 | 6.34x |
| smog_index | 25.33 | 192.56 | 7.60x |
| linsear_write_formula | 0.10 | 4.05 | 39.90x |
| mcalpine_eflaw | 17.97 | 41.44 | 2.31x |
| spache_readability | 31.70 | 196.31 | 6.19x |
| reading_time | 0.90 | 13.66 | 15.18x |

On top of that, the first call in a fresh process pays a one-off load of the syllable and word-list resources: up to 35 ms for `textstat-rs` against 431 ms for `textstat`.

_Generated from `00d7399-dirty` on 2026-07-22, python 3.12.4 vs textstat 0.7.13, on Windows-11-10.0.26200-SP0._
<!-- bench:perf:end -->
