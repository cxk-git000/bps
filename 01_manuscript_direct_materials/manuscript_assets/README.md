# Paper Assets

This directory contains the compact figure and table set prepared for the main manuscript.

## Figures

- `fig01_system_overview`
  - shared input binning, shared codebooks, and scheme-specific plaintext embedding
- `fig02_input_encoding`
  - how Standard PBS, SDR-PBS, and Many-LUT store the same neighboring code blocks
- `fig03_scheme_comparison`
  - structural comparison of the three scheme families
- `fig04_guardband_layout`
  - concrete guarded versus unguarded storage layouts
- `fig05_main_results`
  - main comparison across function pairs
- `fig06_repair_effect`
  - pre-fix versus guarded comparison
- `fig07_timing_breakdown`
  - total and stage-level timing
- `fig08_guardband_ablation`
  - guard-band factor and offset study

## Tables

- `tbl01_scheme_setup`
  - per-scheme parameter and layout summary
- `tbl02_main_results`
  - per-pair comparison of error, RMSE, and latency
- `tbl03_timing_summary`
  - average stage-level timing summary
- `tbl04_codebook_summary`
  - codebook recovery summary
- `tbl05_end_to_end_micro_pipeline`
  - representative-pair application-level end-to-end summary

## Manuscript Snippets

- `manuscript_snippets/TDSC_END_TO_END_SNIPPETS.tex`
  - ready-to-paste subsection and table for the end-to-end micro-pipeline validation

Figures are exported as `PDF` and `PNG`. Tables are exported as `CSV` and `LaTeX`.

