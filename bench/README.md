# Benchmarking

Benchmarking parity and performance between Python and Rust versions

## Run Parity Benchmarks
```bash
uv run python -m bench.run_parity
```
Results will be saved to /results/parity_<commit_hash>.csv and as .json

Set up to benchmark on Wikipedia dataset from HF, as a large, varied and modern text corpus

## Run Performance Benchmarks
```bash
uv run python -m bench.run_perf
```
Results will be shown in the terminal and saved as .json

Both sides are timed the same way: in a fresh process, a warmup call on a throwaway
text pays any lazy resource loading (reported separately as `init`), then the corpus
call is timed. textstat spends ~400ms loading syllable and word list resources on its
first call, so billing that to the metric makes speedups ~3x too flattering.

Python gets a fresh process per sample, as textstat lru_caches its metric functions and
a repeat call on the same text would time as 0ms. The Rust extension does no caching,
so its samples share one process.

Expect ~15% run-to-run variance at the default `SAMPLES = 3`; raise it for numbers
worth publishing.

## Create Report for README
```bash
uv run python -m bench.update_readme
```
Fills the `<!-- bench:parity:... -->` and `<!-- bench:perf:... -->` blocks in the main
README from the newest .json of each kind. Re-running overwrites those blocks in place
and leaves surrounding prose alone, so benchmark runs never touch the README by
themselves — you pick which run gets published.