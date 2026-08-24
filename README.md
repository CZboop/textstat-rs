# Textstat-rs
Rust port of the textstat textual analysis library, with Python bindings.
Drop-in replacements for 11 key metrics in British and US English.

**99.98% exact output match, with at least 2.4x-7x speedup, with 2 most sped-up metrics at ~14x and ~79x** based on benchmarking scripts included in repo, running on Wikipedia dataset as a varied, modern text corpus.

## Installation

```bash
pip install textstat-rs
```

Pre-release versions need `pip install --pre textstat-rs`.

Requires Python 3.10+. Wheels are built for Linux (x86_64, x86, aarch64, armv7; glibc and musl), macOS (x86_64, arm64) and Windows (x64, arm64). They are `abi3` wheels, so one wheel per platform covers every supported Python version. On other platforms pip falls back to the sdist, which needs a Rust toolchain to build.

## Usage

Import name is `textstat_rs`; every metric takes the text as its first argument.

```python
import textstat_rs

text = "The cat sat on the mat. It was a sunny day."

textstat_rs.flesch_reading_ease(text)   # 108.96159090909092
textstat_rs.flesch_kincaid_grade(text)  # -0.5722727272727273
textstat_rs.text_standard(text)         # '0th and 1st grade'
```

The exposed metrics match `textstat`'s signatures, so an existing import can be swapped in place:

```python
import textstat_rs as textstat
```

Optional arguments are the same as the Python library, other than the exceptions noted below:

```python
textstat_rs.gunning_fog(text, syllable_threshold=3)
textstat_rs.linsear_write_formula(text, strict_lower=False, strict_upper=True)
textstat_rs.reading_time(text, ms_per_char=14.69)
```

The counting helpers the formulas are built on are exposed too: `syllable_count`, `sentence_count`, `lexicon_count`, `char_count`, `letter_count`, `polysyllabcount`, `miniword_count` and `difficult_words`.

### Language

British and American English are both supported. The default is `en_US`, matching `textstat`.

Set it once, the way `textstat` does:

```python
textstat_rs.set_lang("en_GB")
textstat_rs.get_lang()                  # 'en_GB'
textstat_rs.syllable_count("colourful") # 3   (2 under en_US)
```

`supported_langs()` returns canonical tags, to allow enumerating locales rather than hardcoding them:

```python
textstat_rs.supported_langs()          # ['en_US', 'en_GB']
```

You can also set the locale per call. It is keyword-only, and overrides over the default:

```python
textstat_rs.flesch_reading_ease(text, lang="en_GB")
```

`lang=` is offered only where the locale can change the answer. It reaches exactly one decision — which hyphenation dictionary spells out syllables for words missing from CMUdict — so `char_count`, `lexicon_count`, `sentence_count`, `letter_count`, `miniword_count`, `reading_time`, `mcalpine_eflaw`, `coleman_liau_index` and `automated_readability_index` do not take it, because it would do nothing.

Tags are resolved the way `pyphen` does — lowercased, `-` normalised to `_`, then trailing subtags dropped until something matches — so `en-GB`, `EN_GB` and `en_US_posix` all work.

Four differences from `textstat` worth knowing:

- **Only English is supported.** `textstat` would accept `"fr"` and reach for French coefficients and a French dictionary. textstat-rs vendors only the two English hyphenation dictionaries, so anything that doesn't resolve to `en_US` or `en_GB` raises `ValueError`.
- **Bad locales fail immediately.** `textstat`'s `set_lang` is a bare assignment, so a typo is accepted and only surfaces later as a `KeyError` from inside `pyphen` — and only once some word actually misses CMUdict, which for a short text may be never.
- **Bare `"en"` means British.** `pyphen` registers a short name for the first matching file in sorted order, and `hyph_en_GB.dic` sorts before `hyph_en_US.dic`. Kept deliberately, so `en` and `en_AU` get the British dictionary. The default when nothing is set is still `en_US`.
- **`lang=` is honoured.** `textstat` still accepts `syllable_count(text, lang=...)` but warns and discards it. Here it works.

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
Scores are compared against `textstat` over 1000 Wikipedia articles (up to 5000 chars each), across `en_US` and `en_GB`: **99.98% of 22000 comparisons match exactly**, and the largest disagreement anywhere is 0.049566.

| locale | comparisons | exact match | max delta |
| --- | --- | --- | --- |
| en_US | 11000 | 99.98% | 0.049566 |
| en_GB | 11000 | 99.97% | 0.049566 |

Metrics returning a grade band rather than a number are compared as exact strings: `text_standard` matches exactly 1000/1000 in `en_US`, 1000/1000 in `en_GB`.

<details>
<summary>Per metric (en_US)</summary>

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

</details>

_Generated from `dc1a373` on 2026-08-24, python 3.12.4 vs textstat 0.7.13._
<!-- bench:parity:end -->

### Performance

<!-- bench:perf:start -->
One call per metric over 100 concatenated Wikipedia articles (0.46 MB), median of 5. Both libraries are warmed first, so these are steady-state numbers with lazy resource loading excluded.

| metric | textstat-rs (ms) | textstat (ms) | speedup |
| --- | --- | --- | --- |
| flesch_reading_ease | 39.29 | 149.60 | 3.81x |
| flesch_kincaid_grade | 36.45 | 148.43 | 4.07x |
| automated_readability_index | 18.87 | 51.10 | 2.71x |
| coleman_liau_index | 13.68 | 59.91 | 4.38x |
| dale_chall_readability_score | 28.11 | 186.25 | 6.63x |
| gunning_fog | 28.72 | 187.38 | 6.52x |
| smog_index | 27.78 | 207.86 | 7.48x |
| linsear_write_formula | 0.04 | 4.16 | 107.98x |
| mcalpine_eflaw | 18.61 | 42.15 | 2.26x |
| spache_readability | 27.91 | 193.68 | 6.94x |
| reading_time | 0.79 | 12.80 | 16.27x |

On top of that, the first call in a fresh process pays a one-off load of the syllable and word-list resources: up to 34 ms for `textstat-rs` against 361 ms for `textstat`.

_Generated from `dc1a373` on 2026-08-24, python 3.12.4 vs textstat 0.7.13._
<!-- bench:perf:end -->

## Third-party data

The scoring resources are compiled into the extension by `include_str!`, so
they ship inside every wheel. Their notices ship with them, under
`textstat_rs-<version>.dist-info/licenses/`.

| Data | Source | Licence | Notice |
| --- | --- | --- | --- |
| `cmudict.txt` | [CMU Pronouncing Dictionary](http://www.speech.cs.cmu.edu/cgi-bin/cmudict) | BSD-2-Clause-style | `src/data/cmudict-LICENSE.txt` |
| `hyph_en_US.dic` | [pyphen](https://github.com/Kozea/Pyphen) / LibreOffice, from `hyphen.tex` | BSD-style | `src/data/README_hyph_en_US.txt` |
| `hyph_en_GB.dic` | [pyphen](https://github.com/Kozea/Pyphen) / LibreOffice, from `ukhyphen.tex` | BSD-style | `src/data/README_hyph_en_GB.txt` |
| `easy_words.txt` | [textstat](https://github.com/textstat/textstat) (Dale-Chall list) | MIT | covered by textstat's MIT licence |

Both `.dic` files are vendored byte-for-byte from `pyphen`, and the notice files
keep their upstream names so they can be diffed against it directly. The
`pyphen-LICENSE.txt` note covers `pyphen` itself (GPL 2.0+/LGPL 2.1+/MPL 1.1
tri-licence). None of `pyphen`'s code is used here, only the two dictionaries,
which carry the BSD-style terms above.

Only the data is third-party. The Rust and Python code in this repository is
MIT, per [LICENSE](LICENSE).
