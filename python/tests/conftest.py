"""Config to run parity tests with deltas printed alongside test ids

Run by adding -s flag to pytest command e.g.
```uv run pytest -s```
or
```uv run pytest -s python/tests/test_parity.py```
"""
import pytest

_orig_approx = pytest.approx


class _LoggedApprox:
    def __init__(self, expected, test_id, **kw):
        self._inner = _orig_approx(expected, **kw)
        self._expected = expected
        self._test_id = test_id

    def __eq__(self, actual):
        ok = actual == self._inner
        if isinstance(actual, (int, float)) and isinstance(
            self._expected, (int, float)
        ):
            delta = actual - self._expected
            print(
                f"    [{self._test_id}] rs={actual:<10.4f} py={self._expected:<10.4f} "
                f"Δ={delta:+.4f}  {'OK' if ok else 'FAIL'}"
            )
        return ok

    def __repr__(self):
        return repr(self._inner)


@pytest.fixture(autouse=True)
def _patch_approx(monkeypatch, request):
    test_id = request.node.name
    monkeypatch.setattr(
        pytest,
        "approx",
        lambda expected, **kw: _LoggedApprox(expected, test_id, **kw),
    )
