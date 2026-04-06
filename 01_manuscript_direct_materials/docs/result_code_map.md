# Result Code Map

This note answers three questions for each result folder:

1. What does this folder contain
2. Which code produces it
3. Which script drives it

## Core code entry points

- main experiment driver: `code/src/main.rs`
- main canonical run script: `code/scripts/run_main_canonical.ps1`
- supplementary run script: `code/scripts/run_supplementary_experiments.ps1`
- analysis figure/table generator: `code/scripts/generate_supplementary_figures.py`
- compact paper asset generator: `code/scripts/generate_paper_figures.py`
- buildable dependency: `code/tfhe-rs/`
- compact modified-library view: `code/library_modified/`

## Result folder map

- `results/canonical/04_main_guarded_all_pairs_10000/`
  - purpose: main comparison used by the paper
  - code: `code/src/main.rs`
  - mode: `continuous`
  - script: `code/scripts/run_main_canonical.ps1`

- `results/canonical/05_repeatability_multiseed_1000/`
  - purpose: repeatability across explicit seeds
  - code: `code/src/main.rs`
  - mode: `continuous`
  - script: `code/scripts/run_supplementary_experiments.ps1`

- `results/canonical/06_guardband_ablation/`
  - purpose: guard factor and input offset ablation
  - code: `code/src/main.rs`
  - mode: `continuous`
  - script: `code/scripts/run_supplementary_experiments.ps1`

- `results/canonical/07_codebook_recovery_validation/`
  - purpose: exact discrete code recovery validation
  - code: `code/src/main.rs`
  - mode: `codebook`
  - script: `code/scripts/run_supplementary_experiments.ps1`

- `results/canonical/08_bitwidth_sensitivity/`
  - purpose: bitwidth sensitivity from 8 to 11 bits
  - code: `code/src/main.rs`
  - mode: `continuous`
  - script: `code/scripts/run_supplementary_experiments.ps1`

- `results/canonical/09_tail_stability_100k/`
  - purpose: long-tail stability and rare-event checking
  - code: `code/src/main.rs`
  - mode: `continuous`
  - script: `code/scripts/run_supplementary_experiments.ps1`

- `results/canonical/10_paper_figures/`
  - purpose: generated analysis figures for the paper
  - code: `code/scripts/generate_supplementary_figures.py`
  - input results: `04`, `05`, `06`, `07`, `08`, `09`, `12`

- `results/canonical/11_paper_tables/`
  - purpose: generated analysis tables for the paper
  - code: `code/scripts/generate_supplementary_figures.py`
  - input results: `04`, `05`, `06`, `07`, `08`, `09`, `12`

- `results/canonical/12_end_to_end_micro_pipeline/`
  - purpose: application-level end-to-end validation
  - code: `code/src/main.rs`
  - mode: `end_to_end`
  - script: `code/scripts/run_supplementary_experiments.ps1`

## Key modified library files

- `code/library_modified/tfhe-rs/tfhe/src/core_crypto/algorithms/lwe_programmable_bootstrapping/mod.rs`
  - contains the accumulator-related implementation used by the buildable code

- `code/library_modified/tfhe-rs/tfhe/src/core_crypto/algorithms/lwe_programmable_bootstrapping/fft64_pbs.rs`
  - contains the dual-coefficient extraction helper used by the SDR-PBS path

If you only want one reading path, use:

1. `code/src/main.rs`
2. `results/canonical/04_main_guarded_all_pairs_10000/`
3. `results/canonical/06_guardband_ablation/`
4. `results/canonical/09_tail_stability_100k/`
5. `results/canonical/12_end_to_end_micro_pipeline/`
6. `results/canonical/10_paper_figures/`
7. `results/canonical/11_paper_tables/`
