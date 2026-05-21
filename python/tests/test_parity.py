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


# Parity tests: Compare original and ported outputs for same inputs

# NOTE: slight drift expected until/without porting syllable, sentence and word tokenisation. Hence approx
# metrics most impacted so far (syllable based) - flesch_reading_ease, flesch_kincaid_grade, linsear_write_formula
# should tune/update abs value


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_flesch_reading_ease_parity(text):
    assert textstat_rs.flesch_reading_ease(text) == pytest.approx(
        textstat.flesch_reading_ease(text), abs=15.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_flesch_kincaid_grade_parity(text):
    assert textstat_rs.flesch_kincaid_grade(text) == pytest.approx(
        textstat.flesch_kincaid_grade(text), abs=4.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_automated_readability_index_parity(text):
    assert textstat_rs.automated_readability_index(text) == pytest.approx(
        textstat.automated_readability_index(text), abs=4.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_coleman_liau_index_parity(text):
    assert textstat_rs.coleman_liau_index(text) == pytest.approx(
        textstat.coleman_liau_index(text), abs=4.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_linsear_write_formula_parity(text):
    assert textstat_rs.linsear_write_formula(text) == pytest.approx(
        textstat.linsear_write_formula(text), abs=4.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_reading_time_parity(text):
    assert textstat_rs.reading_time(text) == pytest.approx(
        textstat.reading_time(text), abs=4.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_mcalpine_eflaw_parity(text):
    assert textstat_rs.mcalpine_eflaw(text) == pytest.approx(
        textstat.mcalpine_eflaw(text), abs=10.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_spache_readability_parity(text):
    assert textstat_rs.spache_readability(text) == pytest.approx(
        textstat.spache_readability(text), abs=2.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_dale_chall_parity(text):
    assert textstat_rs.dale_chall_readability_score(text) == pytest.approx(
        textstat.dale_chall_readability_score(text), abs=0.5
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_gunning_fog_parity(text):
    assert textstat_rs.gunning_fog(text) == pytest.approx(
        textstat.gunning_fog(text), abs=6.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_smog_index_parity(text):
    assert textstat_rs.smog_index(text) == pytest.approx(
        textstat.smog_index(text), abs=2.0
    )


@pytest.mark.parametrize("text", SAMPLE_TEXTS)
def test_text_standard_parity(text):
    ts_grade_lower, ts_grade_upper = [
        re.search("\\d+", i).group(0)
        for i in textstat.text_standard(text).split(" and ")
    ]
    rs_grade_lower, rs_grade_upper = [
        re.search("\\d+", i).group(0)
        for i in textstat.text_standard(text).split(" and ")
    ]
    assert ts_grade_upper == pytest.approx(rs_grade_upper, abs=1.0)
    assert ts_grade_lower == pytest.approx(rs_grade_lower, abs=1.0)
