# Manuscript Asset Map

This note answers one question:

Which figures and tables used by the current manuscript come from which result folders, and which script generates them.

## 1. Compact manuscript assets in `manuscript_assets/`

These files are the compact paper-ready assets.

### Method and structure figures

- `manuscript_assets/figures/fig01_system_overview.*`
  - role: system overview
  - generator: `code/scripts/generate_paper_figures.py`
  - source type: schematic, not a single run folder

- `manuscript_assets/figures/fig02_input_encoding.*`
  - role: input encoding comparison
  - generator: `code/scripts/generate_paper_figures.py`
  - source type: schematic

- `manuscript_assets/figures/fig03_scheme_comparison.*`
  - role: Standard PBS vs SDR-PBS vs Many-LUT layout comparison
  - generator: `code/scripts/generate_paper_figures.py`
  - source type: schematic

- `manuscript_assets/figures/fig04_guardband_layout.*`
  - role: guard-band layout
  - generator: `code/scripts/generate_paper_figures.py`
  - source type: schematic

### Compact quantitative figures

- `manuscript_assets/figures/fig05_main_results.*`
  - role: compact main-result figure
  - generator: `code/scripts/generate_paper_figures.py`
  - result source: `results/canonical/04_main_guarded_all_pairs_10000/summary.csv`

- `manuscript_assets/figures/fig06_repair_effect.*`
  - role: repair-effect comparison
  - generator: `code/scripts/generate_paper_figures.py`
  - result source: `results/canonical/11_paper_tables/table4_prepost_compare.csv`

- `manuscript_assets/figures/fig07_timing_breakdown.*`
  - role: timing breakdown
  - generator: `code/scripts/generate_paper_figures.py`
  - result source: `results/canonical/11_paper_tables/table5_timing_breakdown.csv`

- `manuscript_assets/figures/fig08_guardband_ablation.*`
  - role: guard-band ablation
  - generator: `code/scripts/generate_paper_figures.py`
  - result source: `results/canonical/11_paper_tables/table3_guard_ablation.csv`

### Compact paper tables

- `manuscript_assets/tables/tbl01_scheme_setup.*`
  - role: scheme setup summary
  - generator: `code/scripts/generate_paper_figures.py`
  - code source: scheme configuration in `code/src/main.rs`

- `manuscript_assets/tables/tbl02_main_results.*`
  - role: compact main table
  - generator: `code/scripts/generate_paper_figures.py`
  - result source: `results/canonical/04_main_guarded_all_pairs_10000/summary.csv`

- `manuscript_assets/tables/tbl03_timing_summary.*`
  - role: compact timing summary
  - generator: `code/scripts/generate_paper_figures.py`
  - result source: `results/canonical/11_paper_tables/table5_timing_breakdown.csv`

- `manuscript_assets/tables/tbl04_codebook_summary.*`
  - role: compact codebook summary
  - generator: `code/scripts/generate_paper_figures.py`
  - result source: `results/canonical/07_codebook_recovery_validation/allpairs/codebook_summary.csv`

- `manuscript_assets/tables/tbl05_end_to_end_micro_pipeline.*`
  - role: compact end-to-end summary
  - generator: `code/scripts/generate_paper_figures.py`
  - result source: `results/canonical/12_end_to_end_micro_pipeline/representative_pairs_1000/end_to_end_summary.csv`

- `manuscript_assets/tables/tbl06_code_displacement_summary.*`
  - role: current displacement summary table
  - generator: `../02_current_supporting_materials/code/scripts/generate_revision_tables.py`
  - result source: `../02_current_supporting_materials/results/revision_20260405/14_codebook_displacement_repr3/codebook_summary.csv`

- `manuscript_assets/tables/tbl07_manylut_matched_10bit.*`
  - role: current matched 10-bit comparison table
  - generator: `../02_current_supporting_materials/code/scripts/generate_revision_tables.py`
  - result source: `results/canonical/08_bitwidth_sensitivity/*/bits_10/summary.csv`

## 2. Generated analysis figures in `results/canonical/10_paper_figures/`

These are data-driven analysis figures generated from canonical result folders.

- `fig1_multiseed_error_rate.*`
  - result source: `results/canonical/05_repeatability_multiseed_1000/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig2_tail_stability.*`
  - result source: `results/canonical/09_tail_stability_100k/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig3_guard_ablation_max.*`
  - result source: `results/canonical/06_guardband_ablation/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig4_guard_ablation_rate.*`
  - result source: `results/canonical/06_guardband_ablation/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig5_prepost_compare.*`
  - result source: pre/post comparison summary
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig6_timing_breakdown.*`
  - result source: main comparison timing summary
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig7_codebook_exact_recovery.*`
  - result source: `results/canonical/07_codebook_recovery_validation/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig8_bitwidth_sensitivity.*`
  - result source: `results/canonical/08_bitwidth_sensitivity/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig9_threshold_sensitivity.*`
  - result source: threshold sensitivity summary
  - generator: `code/scripts/generate_supplementary_figures.py`

- `fig10_end_to_end_micro_pipeline.*`
  - result source: `results/canonical/12_end_to_end_micro_pipeline/`
  - generator: `code/scripts/generate_supplementary_figures.py`

## 3. Generated analysis tables in `results/canonical/11_paper_tables/`

- `table1_multiseed_summary.*`
  - result source: `results/canonical/05_repeatability_multiseed_1000/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `table2_tail_stability.*`
  - result source: `results/canonical/09_tail_stability_100k/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `table3_guard_ablation.*`
  - result source: `results/canonical/06_guardband_ablation/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `table4_prepost_compare.*`
  - result source: pre/post comparison summary
  - generator: `code/scripts/generate_supplementary_figures.py`

- `table5_timing_breakdown.*`
  - result source: timing summary from main comparison
  - generator: `code/scripts/generate_supplementary_figures.py`

- `table6_codebook_correctness.*`
  - result source: `results/canonical/07_codebook_recovery_validation/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `table7_bitwidth_sensitivity.*`
  - result source: `results/canonical/08_bitwidth_sensitivity/`
  - generator: `code/scripts/generate_supplementary_figures.py`

- `table8_threshold_sensitivity.*`
  - result source: threshold sensitivity summary from the main comparison
  - generator: `code/scripts/generate_supplementary_figures.py`

- `table9_end_to_end_micro_pipeline.*`
  - result source: `results/canonical/12_end_to_end_micro_pipeline/`
  - generator: `code/scripts/generate_supplementary_figures.py`
