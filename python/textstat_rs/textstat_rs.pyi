"""Type stubs for the compiled `textstat_rs` extension module.

Signatures mirror the `#[pyo3(signature = ...)]` declarations in `src/lib.rs`;
keep the two in step when adding or changing a metric.

`lang` is keyword-only and appears only on functions whose result actually
depends on the locale. Passing nothing uses the module default, which
`set_lang` sets and `get_lang` reports.
"""

__all__ = [
    "automated_readability_index",
    "char_count",
    "coleman_liau_index",
    "dale_chall_readability_score",
    "difficult_words",
    "flesch_kincaid_grade",
    "flesch_reading_ease",
    "get_lang",
    "gunning_fog",
    "letter_count",
    "lexicon_count",
    "linsear_write_formula",
    "mcalpine_eflaw",
    "miniword_count",
    "polysyllabcount",
    "reading_time",
    "sentence_count",
    "set_lang",
    "smog_index",
    "spache_readability",
    "supported_langs",
    "syllable_count",
    "text_standard",
]

# Locale

def set_lang(lang: str) -> None: ...
def get_lang() -> str: ...
def supported_langs() -> list[str]: ...

# Metrics

def flesch_reading_ease(text: str, *, lang: str | None = None) -> float: ...
def flesch_kincaid_grade(text: str, *, lang: str | None = None) -> float: ...
def automated_readability_index(text: str) -> float: ...
def coleman_liau_index(text: str) -> float: ...
def dale_chall_readability_score(text: str, *, lang: str | None = None) -> float: ...
def smog_index(text: str, *, lang: str | None = None) -> float: ...
def spache_readability(text: str, *, lang: str | None = None) -> float: ...
def mcalpine_eflaw(text: str) -> float: ...
def linsear_write_formula(
    text: str,
    strict_lower: bool = False,
    strict_upper: bool = True,
    *,
    lang: str | None = None,
) -> float: ...
def gunning_fog(
    text: str, syllable_threshold: int = 3, *, lang: str | None = None
) -> float: ...
def reading_time(text: str, ms_per_char: float = 14.69) -> float: ...
def text_standard(text: str, *, lang: str | None = None) -> str: ...

# Counts

def syllable_count(text: str, *, lang: str | None = None) -> int: ...
def sentence_count(text: str) -> int: ...
def letter_count(text: str) -> int: ...
def polysyllabcount(text: str, *, lang: str | None = None) -> int: ...
def char_count(text: str, ignore_spaces: bool = True) -> int: ...
def miniword_count(text: str, max_size: int = 3) -> int: ...
def difficult_words(
    text: str,
    syllable_threshold: int = 2,
    unique: bool = True,
    *,
    lang: str | None = None,
) -> int: ...
def lexicon_count(
    text: str,
    removepunct: bool = True,
    split_contractions: bool = False,
    split_hyphens: bool = False,
) -> int: ...
