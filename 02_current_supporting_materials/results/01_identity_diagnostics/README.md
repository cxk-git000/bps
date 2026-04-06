# 01 Identity Diagnostics

This folder contains the low-level identity checks used to validate the structural behavior of the three evaluation paths before nonlinear-function benchmarking.

## Contents

- `base_matrix/`
  - the six canonical identity runs

## Canonical Base Matrix

The canonical base matrix consists of:

- `standard_identity_centered.txt`
- `standard_identity_standard.txt`
- `many_lut_identity_centered.txt`
- `many_lut_identity_standard.txt`
- `sdr_pbs_identity_centered.txt`
- `sdr_pbs_identity_standard.txt`

## Why This Folder Matters

This is the first place to inspect if you want to understand why guarded layouts were introduced before the full main experiments.

This curated release keeps the checked-in base matrix above as the current
identity-diagnostic reference set.

