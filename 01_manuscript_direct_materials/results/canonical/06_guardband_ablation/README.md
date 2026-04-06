# 06 Guard-Band Ablation

This folder studies how the guard-band design affects quality.

## Goal

Evaluate how factor and offset choices influence error behavior in guarded
standard PBS and SDR-PBS configurations.

## Layout

The directory structure is:

- function pair
- guard configuration

Example:

- `sigmoid_sigmoid_deriv/factor2_offset256/`

Each configuration directory contains:

- `run.log`
- `run_notes.txt`
- `runtime_breakdown.csv`
- `summary.csv`

## Why This Folder Matters

This folder explains why the guarded configuration in the main result set was
chosen instead of being treated as an arbitrary parameter setting.

