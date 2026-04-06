# Experiment Map

This file maps the repository's code and scripts to the checked-in result folders
used in the GitHub package.

## Core Drivers

- `01_manuscript_direct_materials/code/src/diagnose.rs`
  - low-level identity diagnostics
- `01_manuscript_direct_materials/code/src/main.rs`
  - main function-pair evaluation driver

## Script Layer

- `02_current_supporting_materials/code/scripts/run_identity_matrix.ps1`
  - base identity-diagnostic matrix
- `01_manuscript_direct_materials/code/scripts/run_main_canonical.ps1`
  - guarded all-pairs canonical run
- `02_current_supporting_materials/code/scripts/run_paper_queue.ps1`
  - grouped queue-style batch flow for additional current reruns
- `01_manuscript_direct_materials/code/scripts/run_supplementary_experiments.ps1`
  - repeatability, ablation, codebook, bit-width, tail, and end-to-end runs
- `01_manuscript_direct_materials/code/scripts/generate_supplementary_figures.py`
  - canonical figures and tables
- `02_current_supporting_materials/code/scripts/generate_revision_tables.py`
  - reviewer-facing revision summary tables

## Result Mapping

- `02_current_supporting_materials/results/01_identity_diagnostics`
  - `diagnose.rs` plus `run_identity_matrix.ps1`
- `01_manuscript_direct_materials/results/canonical/04_main_guarded_all_pairs_10000`
  - `main.rs` main guarded run
- `01_manuscript_direct_materials/results/canonical/05_repeatability_multiseed_1000`
  - `main.rs` plus seeded supplementary batch
- `01_manuscript_direct_materials/results/canonical/06_guardband_ablation`
  - `main.rs` plus ablation batch settings
- `01_manuscript_direct_materials/results/canonical/07_codebook_recovery_validation`
  - `main.rs` in codebook mode
- `01_manuscript_direct_materials/results/canonical/08_bitwidth_sensitivity`
  - `main.rs` with bit-width overrides
- `01_manuscript_direct_materials/results/canonical/09_tail_stability_100k`
  - `main.rs` long-run evaluation
- `01_manuscript_direct_materials/results/canonical/12_end_to_end_micro_pipeline`
  - `main.rs` in end-to-end mode with downstream score/update tasks
- `01_manuscript_direct_materials/results/canonical/10_paper_figures`
  - generated from canonical summaries by `generate_supplementary_figures.py`
- `01_manuscript_direct_materials/results/canonical/11_paper_tables`
  - generated from canonical summaries by `generate_supplementary_figures.py`
- `02_current_supporting_materials/results/revision_20260405`
  - revision-only supporting runs plus `generate_revision_tables.py`
