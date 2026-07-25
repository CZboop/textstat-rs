# Texstat-rs
Rust port of the textstat textual analysis library, with Python bindings.
Drop-in replacements for 11 key metrics in US English.

**99.98% exact output match, with at least 2.4x-7x speedup, with 2 most sped-up metrics at ~14x and ~79x** based on benchmarking scripts included in repo, running on Wikipedia dataset as a varied, modern text corpus.

## Python Comparison

### Functions
#### Exposed Metrics

| Python function                   | Exposed? | Notes                                                      |
| --------------------------------- | :------: | ---------------------------------------------------------- |
| `flesch_reading_ease`             |    ✅     |                                                            |
| `flesch_kincaid_grade`            |    ✅     |                                                            |
| `smog_index`                      |    ✅     |                                                            |
| `coleman_liau_index`              |    ✅     |                                                            |
| `automated_readability_index`     |    ✅     |                                                            |
| `dale_chall_readability_score`    |    ✅     |                                                            |
| `linsear_write_formula`           |    ✅     | signature parity (`strict_lower`, `strict_upper`)          |
| `gunning_fog`                     |    ✅     | Rust exposes `syllable_threshold` arg, Python hardcodes it |
| `spache_readability`              |    ✅     | Rust missing `float_output` arg                            |
| `text_standard`                   |    ✅     | Rust missing `float_output` arg, returns `String` only     |
| `reading_time`                    |    ✅     |                                                            |
| `mcalpine_eflaw`                  |    ✅     |                                                            |
| `dale_chall_readability_score_v2` |    ❌     |                                                            |
| `lix`                             |    ❌     |                                                            |
| `rix`                             |    ❌     |                                                            |
| `fernandez_huerta`                |    ❌     | Spanish                                                    |
| `szigriszt_pazos`                 |    ❌     | Spanish                                                    |
| `gutierrez_polini`                |    ❌     | Spanish                                                    |
| `crawford`                        |    ❌     | Spanish                                                    |
| `gulpease_index`                  |    ❌     | Italian                                                    |
| `wiener_sachtextformel`           |    ❌     | German                                                     |
| `osman`                           |    ❌     | Arabic                                                     |

### Parity

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

_Generated from `5d3aab1` on 2026-07-25, python 3.12.4 vs textstat 0.7.13, on Windows-11-10.0.26200-SP0._
<!-- bench:parity:end -->

### Performance

<!-- bench:perf:start -->
One call per metric over 100 concatenated Wikipedia articles (0.46 MB), median of 5. Both libraries are warmed first, so these are steady-state numbers with lazy resource loading excluded.

| metric | textstat-rs (ms) | textstat (ms) | speedup |
| --- | --- | --- | --- |
| flesch_reading_ease | 39.68 | 145.02 | 3.65x |
| flesch_kincaid_grade | 34.37 | 149.34 | 4.35x |
| automated_readability_index | 16.99 | 47.21 | 2.78x |
| coleman_liau_index | 14.81 | 60.34 | 4.07x |
| dale_chall_readability_score | 29.23 | 180.36 | 6.17x |
| gunning_fog | 28.62 | 192.56 | 6.73x |
| smog_index | 28.09 | 204.73 | 7.29x |
| linsear_write_formula | 0.05 | 3.79 | 79.81x |
| mcalpine_eflaw | 18.05 | 43.65 | 2.42x |
| spache_readability | 30.37 | 192.59 | 6.34x |
| reading_time | 0.91 | 13.23 | 14.58x |

On top of that, the first call in a fresh process pays a one-off load of the syllable and word-list resources: up to 32 ms for `textstat-rs` against 455 ms for `textstat`.

_Generated from `5d3aab1` on 2026-07-25, python 3.12.4 vs textstat 0.7.13, on Windows-11-10.0.26200-SP0._
<!-- bench:perf:end -->
