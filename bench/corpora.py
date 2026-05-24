from pathlib import Path
import pickle
from datasets import load_dataset

CACHE = Path(__file__).parent / "_cache"
CACHE.mkdir(exist_ok=True)

def wikipedia_samples(n: int = 1000, max_chars: int = 5000) -> list[tuple[str, str]]:
    """Return n (id, text) pairs from English Wikipedia, cached locally."""
    key = CACHE / f"wikipedia_{n}_{max_chars}.pkl"
    if key.exists():
        return pickle.loads(key.read_bytes())

    ds = load_dataset("wikimedia/wikipedia", "20231101.en", split="train", streaming=True)
    out = []
    for i, row in enumerate(ds):
        if i >= n:
            break
        out.append((f"wiki_{i:05d}", row["text"][:max_chars]))
    key.write_bytes(pickle.dumps(out))
    return out