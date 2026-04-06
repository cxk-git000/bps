# Reviewer Guide

This guide is the fastest way to inspect the repository without relying on a
separate manuscript PDF.

## Scope

The repository studies shared-input dual-function evaluation in TFHE for
activation-derivative pairs.

The baseline is `Standard PBS`, which uses two independent programmable
bootstrapping calls on the same encrypted input. The proposed path is
`SDR-PBS`, which reuses one blind-rotation path and then recovers two outputs.
`Many-LUT` is included as a shared-blind-rotation comparison point.

## Main Takeaway

At the checked-in validated operating point, SDR-PBS keeps accuracy close to
Standard PBS while roughly halving online latency.

The clearest checked-in summary files are:

- `01_manuscript_direct_materials/manuscript_assets/tables/tbl02_main_results.csv`
- `01_manuscript_direct_materials/manuscript_assets/tables/tbl03_timing_summary.csv`
- `01_manuscript_direct_materials/results/canonical/11_paper_tables/table5_timing_breakdown.csv`

## Recommended Reading Path

1. `01_manuscript_direct_materials/README.md`
2. `01_manuscript_direct_materials/manuscript_assets/tables/tbl02_main_results.csv`
3. `01_manuscript_direct_materials/manuscript_assets/tables/tbl03_timing_summary.csv`
4. `01_manuscript_direct_materials/docs/result_code_map.md`
5. `02_current_supporting_materials/results/01_identity_diagnostics/README.md`
6. `02_current_supporting_materials/docs/project_docs/reproduction.md`

## Where The Claims Come From

- main comparison across seven function pairs:
  - `01_manuscript_direct_materials/results/canonical/04_main_guarded_all_pairs_10000/summary.csv`
- repeatability across seeds:
  - `01_manuscript_direct_materials/results/canonical/05_repeatability_multiseed_1000/`
- guard-band ablation:
  - `01_manuscript_direct_materials/results/canonical/06_guardband_ablation/`
- codebook recovery:
  - `01_manuscript_direct_materials/results/canonical/07_codebook_recovery_validation/`
- bitwidth sensitivity:
  - `01_manuscript_direct_materials/results/canonical/08_bitwidth_sensitivity/`
- long-tail stability:
  - `01_manuscript_direct_materials/results/canonical/09_tail_stability_100k/`
- application-level end-to-end validation:
  - `01_manuscript_direct_materials/results/canonical/12_end_to_end_micro_pipeline/`

## Code Entry Points

- main experiment driver:
  - `01_manuscript_direct_materials/code/src/main.rs`
- low-level diagnostic driver:
  - `01_manuscript_direct_materials/code/src/diagnose.rs`
- key modified vendored-library files:
  - `01_manuscript_direct_materials/code/library_modified/`

## Reproduction

The shortest reproduction route is documented in:

- `02_current_supporting_materials/docs/project_docs/reproduction.md`

If you only want to inspect checked-in evidence rather than rerun the pipeline,
you can stay entirely within the checked-in result and table folders above.
