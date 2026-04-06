# Reproduction Guide

This repository should be readable as a reviewer-facing GitHub artifact rather
than as a raw lab snapshot.

## Environment

Recommended baseline:

- Windows + PowerShell
- Rust toolchain matching `rust-toolchain.toml`
- Python with the packages listed in `requirements.txt`

## Build Roots

The main buildable Rust crate lives in:

- `01_manuscript_direct_materials/code/`

Supporting scripts that sit outside that crate live in:

- `02_current_supporting_materials/code/scripts/`

Main binaries:

- `cargo run --bin diagnose`
- `cargo run --bin Re-test`

## Canonical Rebuild Path

The intended rebuild flow is:

1. run identity diagnostics
2. run the main guarded all-pairs experiment
3. run supplementary experiments
4. regenerate figures and tables

The main experiment scripts live in:

- `01_manuscript_direct_materials/code/scripts/`

Supporting diagnostic and revision scripts live in:

- `02_current_supporting_materials/code/scripts/`

## Rebuild Commands

From the repository root, one reproducible path is:

```powershell
powershell -ExecutionPolicy Bypass -File .\02_current_supporting_materials\code\scripts\run_identity_matrix.ps1 -MasterSeed 10101
Set-Location .\01_manuscript_direct_materials\code
powershell -ExecutionPolicy Bypass -File .\scripts\run_main_canonical.ps1 -MasterSeed 10101
powershell -ExecutionPolicy Bypass -File .\scripts\run_supplementary_experiments.ps1 -Section short
python .\scripts\generate_supplementary_figures.py
python ..\..\02_current_supporting_materials\code\scripts\generate_revision_tables.py
Set-Location ..\..
```

Notes:

- The identity-diagnostic command writes fresh outputs under
  `02_current_supporting_materials/results/reproduced/` by default.
- The next three commands write fresh outputs under
  `01_manuscript_direct_materials/results/regenerated/`.
- The final Python command updates the checked-in reviewer-facing tables
  `tbl06_*` and `tbl07_*` under
  `01_manuscript_direct_materials/manuscript_assets/tables/`.
- `run_identity_matrix.ps1` is not a seconds-level smoke test; with the checked-in matrix settings it should be budgeted as a multi-minute diagnostic batch.
- For application-level end-to-end reproduction of the decryption-end workflow, run:

```powershell
Set-Location .\01_manuscript_direct_materials\code
powershell -ExecutionPolicy Bypass -File .\scripts\run_supplementary_experiments.ps1 -Section endtoend
Set-Location ..\..
```

- For full long-run reproduction of `09_tail_stability_100k`, run:

```powershell
Set-Location .\01_manuscript_direct_materials\code
powershell -ExecutionPolicy Bypass -File .\scripts\run_supplementary_experiments.ps1 -Section tail100k
Set-Location ..\..
```

## Output Conventions

- Fresh main-run outputs go under `01_manuscript_direct_materials/results/regenerated/`.
- Fresh identity reruns go under `02_current_supporting_materials/results/reproduced/`.
- Checked-in main results live under `01_manuscript_direct_materials/results/canonical/`.
- Checked-in supporting results live under `02_current_supporting_materials/results/`.

## Determinism

- Main runs should use explicit master seeds.
- Diagnostic runs should also be launched with explicit seeds when exact replay is desired.
- Every regenerated run should record seed, selected schemes, and parameter configuration.

## Reading Order

Recommended reading order:

1. `02_current_supporting_materials/results/01_identity_diagnostics`
2. `01_manuscript_direct_materials/results/canonical/04_main_guarded_all_pairs_10000`
3. `01_manuscript_direct_materials/results/canonical/05_repeatability_multiseed_1000`
4. `01_manuscript_direct_materials/results/canonical/06_guardband_ablation`
5. `01_manuscript_direct_materials/results/canonical/07_codebook_recovery_validation`
6. `01_manuscript_direct_materials/results/canonical/08_bitwidth_sensitivity`
7. `01_manuscript_direct_materials/results/canonical/09_tail_stability_100k`
8. `01_manuscript_direct_materials/results/canonical/12_end_to_end_micro_pipeline`
9. `01_manuscript_direct_materials/results/canonical/10_paper_figures`
10. `01_manuscript_direct_materials/results/canonical/11_paper_tables`
