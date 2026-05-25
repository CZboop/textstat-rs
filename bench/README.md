# Benchmarking

Benchmarking parity and performance between Python and Rust versions

## Run Parity Benchmarks
```bash
uv run python -m bench.run_parity
```
Results will be saved to /results/parity_<commit_hash>.csv

Currently set up to benchmark on Wikipedia dataset from HF, as a large modern text corpus

## Run Performance Benchmarks
```bash
uv run python -m bench.run_perf
```
Results will be shown in the terminal