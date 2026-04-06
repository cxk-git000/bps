# 01 Manuscript Direct Materials

This folder contains the buildable code, checked-in canonical results, and
manuscript-ready assets that support the main claims of the repository.

It is intended to be understandable without a separate manuscript PDF.

## Layout

- `code/`
  - main experiment code and figure/table generation scripts
  - the main entry point is `code/src/main.rs`
  - current implementation notes are in `code/docs/`

- `results/`
  - canonical checked-in result folders used for the main repository claims

- `manuscript_assets/`
  - compact reviewer-facing figures and tables
  - includes the current supplemental tables `tbl06_*` and `tbl07_*`

- `docs/`
  - clean explanation files for this section

## Most important code

- `code/src/main.rs`
  - main experiment driver
  - includes `standard_pbs`, `sdr_pbs`, and `many_lut`
  - supports `continuous`, `codebook`, and `end_to_end`

- `code/scripts/run_main_canonical.ps1`
  - drives `results/canonical/04_main_guarded_all_pairs_10000/`

- `code/scripts/run_supplementary_experiments.ps1`
  - drives repeatability, guard-band ablation, codebook validation,
    bitwidth sensitivity, tail stability, and end-to-end runs

- `code/scripts/generate_supplementary_figures.py`
  - generates `results/canonical/10_paper_figures/`
  - generates `results/canonical/11_paper_tables/`

- `code/scripts/generate_paper_figures.py`
  - generates `manuscript_assets/figures/`
  - generates `manuscript_assets/tables/`

- `../02_current_supporting_materials/code/scripts/generate_revision_tables.py`
  - generates `manuscript_assets/tables/tbl06_*`
  - generates `manuscript_assets/tables/tbl07_*`

- `code/tfhe-rs/`
  - buildable vendored dependency

- `code/library_modified/`
  - compact view of the key modified library files

## Most important results

- `results/canonical/04_main_guarded_all_pairs_10000/`
  - main comparison result

- `results/canonical/05_repeatability_multiseed_1000/`
  - multi-seed repeatability

- `results/canonical/06_guardband_ablation/`
  - guard-band factor and offset ablation

- `results/canonical/07_codebook_recovery_validation/`
  - discrete codebook recovery validation

- `results/canonical/08_bitwidth_sensitivity/`
  - bitwidth sensitivity

- `results/canonical/09_tail_stability_100k/`
  - long-tail stability

- `results/canonical/10_paper_figures/`
  - generated analysis figures

- `results/canonical/11_paper_tables/`
  - generated analysis tables

- `results/canonical/12_end_to_end_micro_pipeline/`
  - application-level end-to-end validation

## Quick links

- asset-to-source map: `docs/manuscript_asset_map.md`
- result-to-code map: `docs/result_code_map.md`

This folder does not include archive-only material, regenerated scratch outputs, or build artifacts.
