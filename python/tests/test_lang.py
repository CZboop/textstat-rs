"""Tests for the locale surface itself, rather than for score parity.

`set_lang` writes process-wide state, so every test here restores the default.
The `restore_default_lang` fixture does it unconditionally, including when the
test body raises.
"""

import pytest
import textstat_rs

# A word pair the two dictionaries genuinely disagree about. Everything below
# relies on this: without a locale-sensitive input, nothing here can fail.
DIVERGENT = "colourful behaviour"


@pytest.fixture(autouse=True)
def restore_default_lang():
    yield
    textstat_rs.set_lang("en_US")


def test_default_is_american():
    assert textstat_rs.get_lang() == "en_US"


def test_set_lang_round_trips():
    textstat_rs.set_lang("en_GB")
    assert textstat_rs.get_lang() == "en_GB"


@pytest.mark.parametrize(
    "tag, expected",
    [
        ("en_US", "en_US"),
        ("en_GB", "en_GB"),
        # Separator and case are normalised the way pyphen does it.
        ("en-GB", "en_GB"),
        ("EN_GB", "en_GB"),
        ("en_gb", "en_GB"),
        # No en_AU dictionary is vendored, so it truncates to the `en` root...
        ("en_AU", "en_GB"),
        # ...and bare `en` is British, not American. pyphen registers a short
        # name for the first matching file in sorted order, and hyph_en_GB.dic
        # sorts before hyph_en_US.dic. Matching the quirk, not fixing it.
        ("en", "en_GB"),
        # Extra subtags are lopped off one at a time.
        ("en_US_posix", "en_US"),
        # Including an empty one: pyphen's loop is `while '_' in language`, so
        # a trailing separator costs one extra turn and then resolves.
        ("en_", "en_GB"),
    ],
)
def test_tag_resolution(tag, expected):
    textstat_rs.set_lang(tag)
    assert textstat_rs.get_lang() == expected


@pytest.mark.parametrize("tag", ["fr", "de_DE", "", "nonsense"])
def test_unsupported_language_raises(tag):
    """textstat-rs vendors only the English dictionaries.

    A divergence from textstat, which accepts any string here and only fails
    later with a KeyError out of `Pyphen.__init__` -- and only once some word
    actually misses CMUdict. See PARITY.md.
    """
    with pytest.raises(ValueError, match="unsupported language"):
        textstat_rs.set_lang(tag)


def test_failed_set_lang_leaves_default_untouched():
    textstat_rs.set_lang("en_GB")
    with pytest.raises(ValueError):
        textstat_rs.set_lang("fr")
    assert textstat_rs.get_lang() == "en_GB"


# --- per-call lang= ------------------------------------------------------


LOCALE_SENSITIVE = [
    textstat_rs.flesch_reading_ease,
    textstat_rs.flesch_kincaid_grade,
    textstat_rs.linsear_write_formula,
    textstat_rs.spache_readability,
    textstat_rs.dale_chall_readability_score,
    textstat_rs.gunning_fog,
    textstat_rs.smog_index,
    textstat_rs.text_standard,
    textstat_rs.syllable_count,
    textstat_rs.polysyllabcount,
    textstat_rs.difficult_words,
]

LOCALE_INERT = [
    textstat_rs.automated_readability_index,
    textstat_rs.coleman_liau_index,
    textstat_rs.reading_time,
    textstat_rs.mcalpine_eflaw,
    textstat_rs.sentence_count,
    textstat_rs.lexicon_count,
    textstat_rs.char_count,
    textstat_rs.letter_count,
    textstat_rs.miniword_count,
]


@pytest.mark.parametrize("fn", LOCALE_SENSITIVE, ids=lambda f: f.__name__)
def test_kwarg_accepted_where_locale_matters(fn):
    assert fn(DIVERGENT, lang="en_US") is not None
    assert fn(DIVERGENT, lang="en_GB") is not None


@pytest.mark.parametrize("fn", LOCALE_INERT, ids=lambda f: f.__name__)
def test_kwarg_rejected_where_locale_cannot_matter(fn):
    """No `lang=` on functions the locale can't reach.

    Offering it would imply a knob that does nothing. The locale reaches
    exactly one decision -- which hyphenation dictionary spells out-of-
    vocabulary syllables -- and none of these count syllables.
    """
    with pytest.raises(TypeError):
        fn(DIVERGENT, lang="en_GB")


@pytest.mark.parametrize("fn", LOCALE_SENSITIVE, ids=lambda f: f.__name__)
def test_kwarg_overrides_the_default(fn):
    """`lang="en_GB"` answers as if en_GB were the default, whatever it is.

    Stated as an equality against the default-driven path rather than as
    `kwarg != default`: on a short input a metric can legitimately score the
    same in both locales -- `difficult_words` counts both of these words as
    difficult either way -- and that agreement is not a failure to override.
    `test_locale_changes_the_answer` is what proves the switch is live.
    """
    textstat_rs.set_lang("en_GB")
    via_default = fn(DIVERGENT)
    textstat_rs.set_lang("en_US")
    assert fn(DIVERGENT, lang="en_GB") == via_default


def test_locale_changes_the_answer():
    """The switch is real, not just accepted and ignored."""
    assert textstat_rs.syllable_count(DIVERGENT, lang="en_US") != textstat_rs.syllable_count(
        DIVERGENT, lang="en_GB"
    )
    textstat_rs.set_lang("en_US")
    american = textstat_rs.syllable_count(DIVERGENT)
    textstat_rs.set_lang("en_GB")
    assert textstat_rs.syllable_count(DIVERGENT) != american


@pytest.mark.parametrize("fn", LOCALE_SENSITIVE, ids=lambda f: f.__name__)
def test_omitting_the_kwarg_follows_set_lang(fn):
    textstat_rs.set_lang("en_GB")
    assert fn(DIVERGENT) == fn(DIVERGENT, lang="en_GB")


@pytest.mark.parametrize("fn", LOCALE_SENSITIVE, ids=lambda f: f.__name__)
def test_bad_kwarg_raises(fn):
    with pytest.raises(ValueError, match="unsupported language"):
        fn(DIVERGENT, lang="fr")


def test_lang_is_keyword_only():
    """`lang` is a modal setting, not a formula parameter.

    Keyword-only so a real parameter can be added ahead of it later without
    silently rebinding anyone's positional argument. textstat's vestigial
    `syllable_count(text, lang)` was positional, but it is deprecated there and
    discarded after a warning, so nothing working depends on that spelling.
    """
    with pytest.raises(TypeError):
        textstat_rs.syllable_count(DIVERGENT, "en_GB")
