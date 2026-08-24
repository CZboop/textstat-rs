"""Benchmark corpora, selected by the language of the locale under test.

A locale picks two things: which formulas run, and which text they should run
against.`en_US` and `en_GB` differ only in a
hyphenation dictionary, so they share a corpus; a future `fr` would need French
text, because scoring French coefficients over English Wikipedia measures
nothing.

So corpora are keyed on the *language root* rather than the full tag -- the
same truncation pyphen does -- which means adding a locale under an existing
language costs no download, and adding a new language is a one-line entry in
`DATASETS` plus a clear error until it is made.
"""

from pathlib import Path
import pickle

CACHE = Path(__file__).parent / "_cache"
CACHE.mkdir(exist_ok=True)

# language root -> huggingface `wikimedia/wikipedia` config
DATASETS = {
    "en": "20231101.en",
}


def lang_root(lang: str) -> str:
    """`"en_GB"` -> `"en"`. Mirrors textstat's `get_lang_root`."""
    return lang.replace("-", "_").lower().split("_")[0]


def _cache_paths(root: str, n: int, max_chars: int) -> list[Path]:
    """Candidate cache files, newest naming first.

    The second entry is the pre-multi-locale name. Keeping it readable means
    the corpora already on disk stay usable instead of forcing a re-download
    the first time this runs.
    """
    paths = [CACHE / f"wikipedia_{root}_{n}_{max_chars}.pkl"]
    if root == "en":
        paths.append(CACHE / f"wikipedia_{n}_{max_chars}.pkl")
    return paths


def _larger_cached(root: str, n: int, max_chars: int) -> list[tuple[str, str]] | None:
    """Smallest cached corpus for this language holding at least `n` samples.

    Smallest rather than any, so slicing 20 out of a cached 50 does not load a
    1000-article pickle to throw most of it away.
    """
    best: tuple[int, Path] | None = None
    for path in CACHE.glob(f"wikipedia_*_{max_chars}.pkl"):
        parts = path.stem.split("_")
        # `wikipedia_<root>_<n>_<max_chars>`, or the legacy English
        # `wikipedia_<n>_<max_chars>`. Matched field by field rather than by
        # glob: `wikipedia_*_5000.pkl` also matches another language's cache,
        # and slicing French articles into an English run would be silent.
        if len(parts) == 4 and parts[1] == root and parts[2].isdigit():
            size = int(parts[2])
        elif len(parts) == 3 and root == "en" and parts[1].isdigit():
            size = int(parts[1])
        else:
            continue
        # Smallest sufficient, so slicing 20 out of a cached 50 does not load a
        # 1000-article pickle to throw most of it away.
        if size >= n and (best is None or size < best[0]):
            best = (size, path)

    if best is None:
        return None
    return pickle.loads(best[1].read_bytes())


def corpus_for(
    lang: str = "en_US", n: int = 1000, max_chars: int = 5000
) -> list[tuple[str, str]]:
    """Return n (id, text) pairs in the language of `lang`, cached locally.

    Raises
    ------
    KeyError
        If no corpus is registered for the language
        Prevents benchmarking one language's text against another's dicts
    """
    root = lang_root(lang)
    if root not in DATASETS:
        raise KeyError(
            f"no benchmark corpus registered for language {root!r} (from {lang!r}). "
            f"Registered: {', '.join(sorted(DATASETS))}. "
            f"Add a 'wikimedia/wikipedia' config for {root!r} to bench.corpora.DATASETS."
        )

    paths = _cache_paths(root, n, max_chars)
    for path in paths:
        if path.exists():
            return pickle.loads(path.read_bytes())

    # No exact-size cache. A larger one for the same language slices down
    # Without this, `--samples 20` triggers a
    # download purely because nobody had asked for that size before.
    bigger = _larger_cached(root, n, max_chars)
    if bigger is not None:
        return bigger[:n]

    # Imported lazily - cached corpus doesn't require `datasets` 
    # to be installed
    from datasets import load_dataset

    ds = load_dataset(
        "wikimedia/wikipedia", DATASETS[root], split="train", streaming=True
    )
    out = []
    for i, row in enumerate(ds):
        if i >= n:
            break
        out.append((f"wiki_{root}_{i:05d}", row["text"][:max_chars]))
    paths[0].write_bytes(pickle.dumps(out))
    return out


def wikipedia_samples(n: int = 1000, max_chars: int = 5000) -> list[tuple[str, str]]:
    """English corpus, for callers that predate the locale dimension."""
    return corpus_for("en_US", n, max_chars)
