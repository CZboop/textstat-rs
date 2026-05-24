"""Config to run parity tests with the worst-case delta per metric summarised at end.

Run with -s so the summary surfaces, e.g.
```uv run pytest -s python/tests/test_parity.py```
"""
import pytest
from collections import defaultdict

_orig_approx = pytest.approx

# parent test name -> info for the worst-abs-delta param seen so far
_worst: dict[str, dict] = defaultdict(
    lambda: {"abs_delta": -1.0, "delta": 0.0, "param": "", "rs": None, "py": None}
)


class _LoggedApprox:
    def __init__(self, expected, parent_name, param_id, **kw):
        self._inner = _orig_approx(expected, **kw)
        self._expected = expected
        self._parent_name = parent_name
        self._param_id = param_id

    def __eq__(self, actual):
        ok = actual == self._inner
        if isinstance(actual, (int, float)) and isinstance(
            self._expected, (int, float)
        ):
            delta = actual - self._expected
            abs_delta = abs(delta)
            cur = _worst[self._parent_name]
            if abs_delta > cur["abs_delta"]:
                _worst[self._parent_name] = {
                    "abs_delta": abs_delta,
                    "delta": delta,
                    "param": self._param_id,
                    "rs": actual,
                    "py": self._expected,
                    "ok": ok,
                }
        return ok

    def __repr__(self):
        return repr(self._inner)


@pytest.fixture(autouse=True)
def _patch_approx(monkeypatch, request):
    parent_name = request.node.originalname or request.node.name
    param_id = (
        request.node.callspec.id if hasattr(request.node, "callspec") else ""
    )
    monkeypatch.setattr(
        pytest,
        "approx",
        lambda expected, **kw: _LoggedApprox(expected, parent_name, param_id, **kw),
    )


def pytest_terminal_summary(terminalreporter, exitstatus, config):
    if not _worst:
        return
    terminalreporter.section("Parity: worst-case delta per metric")
    rows = sorted(_worst.items(), key=lambda kv: -kv[1]["abs_delta"])
    for parent, info in rows:
        metric = parent.removeprefix("test_").removesuffix("_parity")
        marker = "OK  " if info.get("ok") else "FAIL"
        terminalreporter.write_line(
            f"  {marker}  {metric:<32} [{info['param']:<24}]  "
            f"rs={info['rs']:<10.4f} py={info['py']:<10.4f} "
            f"Δ={info['delta']:+.4f}"
        )
