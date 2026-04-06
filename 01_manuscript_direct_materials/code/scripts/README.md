# Scripts

This directory contains the manuscript-facing orchestration layer around the
Rust binaries.

## Files

- `run_main_canonical.ps1`
  - Runs the guarded all-pairs main experiment.
  - Produces the checked-in output structure for `04_main_guarded_all_pairs_10000`
    when directed to a results folder.

- `run_supplementary_experiments.ps1`
  - Main batch script for the supplementary experiment families.
  - Covers repeatability, guard-band ablation, codebook validation, bit-width
    sensitivity, tail-stability, and end-to-end micro-pipeline runs.

- `generate_supplementary_figures.py`
  - Reads canonical results and derived summaries.
  - Writes regenerated figure/table assets under `results/regenerated/` by default.
  - Can also be pointed at `results/canonical/10_paper_figures` and
    `results/canonical/11_paper_tables` when updating checked-in assets.

- `generate_paper_figures.py`
  - Builds the compact manuscript figure and table set.
  - Writes outputs under `paper_assets/figures` and `paper_assets/tables`.

## Usage Note

The PowerShell scripts now default to repository-relative paths so they can be
run from a clean clone without editing local drive letters.

Supporting-only helpers are kept in:

- `../../02_current_supporting_materials/code/scripts/`
  - `run_identity_matrix.ps1`
  - `run_paper_queue.ps1`
  - `generate_revision_tables.py`
