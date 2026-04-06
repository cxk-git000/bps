# 07 Codebook Validation

This folder contains exact discrete-code recovery validation.

## Goal

Check whether the schemes recover the intended quantized codewords exactly when
evaluated in codebook mode.

## Contents

The canonical run is stored under:

- `allpairs/`

Key files:

- `codebook_summary.csv`
- `run.log`
- `runtime_breakdown.csv`
- `run_notes.txt`

## Why This Folder Matters

The main continuous metrics are useful, but this folder answers a stricter
question: whether the discrete encoded outputs themselves are recovered exactly.
