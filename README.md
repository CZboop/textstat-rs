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
One call per metric over 100 concatenated Wikipedia articles (0.46 MB), median of 3. Both libraries are warmed first, so these are steady-state numbers with lazy resource loading excluded.

| metric | textstat-rs (ms) | textstat (ms) | speedup |
| --- | --- | --- | --- |
| flesch_reading_ease | 39.23 | 137.41 | 3.50x |
| flesch_kincaid_grade | 33.02 | 139.14 | 4.21x |
| automated_readability_index | 17.02 | 46.73 | 2.75x |
| coleman_liau_index | 12.91 | 56.27 | 4.36x |
| dale_chall_readability_score | 30.17 | 171.64 | 5.69x |
| gunning_fog | 28.51 | 185.50 | 6.51x |
| smog_index | 25.68 | 193.78 | 7.55x |
| linsear_write_formula | 4.10 | 3.74 | 0.91x |
| mcalpine_eflaw | 18.70 | 43.21 | 2.31x |
| spache_readability | 28.43 | 183.70 | 6.46x |
| reading_time | 1.01 | 13.10 | 12.98x |

On top of that, the first call in a fresh process pays a one-off load of the syllable and word-list resources: up to 31 ms for `textstat-rs` against 424 ms for `textstat`.

_Generated from `bbf8ad9` on 2026-07-22, python 3.12.4 vs textstat 0.7.13, on Windows-11-10.0.26200-SP0._
<!-- bench:perf:end -->
