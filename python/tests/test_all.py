import pytest
import textstat_rs


def test_sum_as_string():
    assert textstat_rs.sum_as_string(1, 1) == "2"
