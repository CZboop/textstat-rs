import csv
import subprocess
from pathlib import Path

import textstat
import textstat_rs

from bench.corpora import wikipedia_samples

METRICS = [
    "flesch_reading_ease",
    "flesch_kincaid_grade",
    "automated_readability_index",
    "coleman_liau_index",
    "dale_chall_readability_score",
    "gunning_fog",
    "smog_index",
    "linsear_write_formula",
    "mcalpine_eflaw",
    "spache_readability",
    "reading_time",
]


def _git_sha() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"], text=True
        ).strip()
    except Exception:
        return "nogit"


def main():
    sha = _git_sha()
    out = Path(__file__).parent.parent / "results" / f"parity_{sha}.csv"
    out.parent.mkdir(exist_ok=True)
    samples = wikipedia_samples(1000)

    with out.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["metric", "sample_id", "rs", "py", "abs_delta", "rel_delta"])
        for sample_id, text in samples:
            for m in METRICS:
                rs = getattr(textstat_rs, m)(text)
                py = getattr(textstat, m)(text)
                abs_d = abs(rs - py)
                rel_d = abs(abs_d / py if py else 0.0)
                w.writerow(
                    [
                        m,
                        sample_id,
                        f"{rs:.6f}",
                        f"{py:.6f}",
                        f"{abs_d:.6f}",
                        f"{rel_d:.6f}",
                    ]
                )
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
