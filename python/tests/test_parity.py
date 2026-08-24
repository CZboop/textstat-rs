import pytest
import textstat
import textstat_rs
import re

SAMPLE_TEXTS = [
    "The cat sat on the mat. It was a sunny day.",
    "Photosynthesis is the process by which plants convert sunlight into chemical energy stored in glucose molecules.",
    "Hello world. This is a simple test sentence for readability scoring.",
    "Mitochondria is the powerhouse of the cell.",
]

LANGS = ["en_US", "en_GB"]


@pytest.fixture(autouse=True, params=LANGS)
def lang(request):
    """Run every parity test in this module once per supported locale.

    Autouse and parametrised, so the tests below need no changes: each one
    doubles, and both libraries are switched underneath it. Both `set_lang`
    calls write process-wide state -- textstat's onto a module-level
    `TextStatistics` singleton, ours onto a `RwLock` in the extension -- so
    restoring the default afterwards is not optional. Leaking `en_GB` out of
    here would silently reinterpret every later test in the session.
    """
    textstat.set_lang(request.param)
    textstat_rs.set_lang(request.param)
    yield request.param
    textstat.set_lang("en_US")
    textstat_rs.set_lang("en_US")


def test_locales_are_distinguishable(lang):
    """Guard against the parametrisation above being vacuous.

    Every other test here would still pass if `set_lang` were a no-op and both
    libraries always scored American English. This is the one that notices.
    """
    us = textstat_rs.syllable_count("colourful behaviour", lang="en_US")
    gb = textstat_rs.syllable_count("colourful behaviour", lang="en_GB")
    assert us != gb


# Parity tests: Compare original and ported outputs for same inputs


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_flesch_reading_ease_parity(text):
    assert textstat_rs.flesch_reading_ease(text) == pytest.approx(
        textstat.flesch_reading_ease(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_flesch_kincaid_grade_parity(text):
    assert textstat_rs.flesch_kincaid_grade(text) == pytest.approx(
        textstat.flesch_kincaid_grade(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_automated_readability_index_parity(text):
    assert textstat_rs.automated_readability_index(text) == pytest.approx(
        textstat.automated_readability_index(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_coleman_liau_index_parity(text):
    assert textstat_rs.coleman_liau_index(text) == pytest.approx(
        textstat.coleman_liau_index(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_linsear_write_formula_parity(text):
    assert textstat_rs.linsear_write_formula(text) == pytest.approx(
        textstat.linsear_write_formula(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_reading_time_parity(text):
    assert textstat_rs.reading_time(text) == pytest.approx(
        textstat.reading_time(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_mcalpine_eflaw_parity(text):
    assert textstat_rs.mcalpine_eflaw(text) == pytest.approx(
        textstat.mcalpine_eflaw(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_spache_readability_parity(text):
    assert textstat_rs.spache_readability(text) == pytest.approx(
        textstat.spache_readability(text), abs=0.065
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_dale_chall_parity(text):
    assert textstat_rs.dale_chall_readability_score(text) == pytest.approx(
        textstat.dale_chall_readability_score(text), abs=0.091
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_gunning_fog_parity(text):
    assert textstat_rs.gunning_fog(text) == pytest.approx(
        textstat.gunning_fog(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_smog_index_parity(text):
    assert textstat_rs.smog_index(text) == pytest.approx(
        textstat.smog_index(text), abs=0.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_text_standard_parity(text):
    ts_grade_lower, ts_grade_upper = [
        int(re.search("\\d+", i).group(0))
        for i in textstat.text_standard(text).split(" and ")
    ]
    rs_grade_lower, rs_grade_upper = [
        int(re.search("\\d+", i).group(0))
        for i in textstat_rs.text_standard(text).split(" and ")
    ]
    assert ts_grade_upper == pytest.approx(rs_grade_upper, abs=0.0)
    assert ts_grade_lower == pytest.approx(rs_grade_lower, abs=0.0)

    # asserting against both int grade and full string
    # int feeds max delta, str ensures suffixes are also the same
    assert textstat_rs.text_standard(text) == textstat.text_standard(text)
