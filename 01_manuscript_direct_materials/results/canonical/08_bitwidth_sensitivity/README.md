# 08 Bit-Width Sensitivity

This folder evaluates sensitivity to quantization precision.

## Goal

Compare behavior across multiple output bit widths for selected function pairs.

## Layout

The directory structure is:

- function pair
- bit-width configuration

Example:

- `gelu_gelu_deriv/bits_10/`

Each bit-width directory contains:

- `run.log`
- `run_notes.txt`
- `runtime_breakdown.csv`
- `summary.csv`

## Why This Folder Matters

This folder shows how the tradeoff between precision and robustness evolves as
the codebook resolution changes.
