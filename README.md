# BPS

The repository contains the current experiment code, checked-in results, and
figures and tables, and supporting implementation notes.

## What This Repository Studies

The repository compares three ways to evaluate activation-derivative pairs at one
encrypted input:

- `Standard PBS`
  - two independent PBS calls
- `SDR-PBS`
  - one shared blind rotation with dual-output recovery
- `Many-LUT`
  - one shared blind rotation with slot-based multi-LUT recovery

The repository also includes the guard-band stabilization experiments that make
continuous-function evaluation dependable at the validated operating points.

## Key Checked-In Evidence

- main seven-pair comparison:
  - `01_manuscript_direct_materials/manuscript_assets/tables/tbl02_main_results.csv`
- timing summary:
  - `01_manuscript_direct_materials/manuscript_assets/tables/tbl03_timing_summary.csv`
  - checked-in summary values are `221.60 ms` for Standard PBS, `111.03 ms`
    for SDR-PBS, and `110.86 ms` for Many-LUT
- codebook and end-to-end summaries:
  - `01_manuscript_direct_materials/manuscript_assets/tables/tbl04_codebook_summary.csv`
  - `01_manuscript_direct_materials/manuscript_assets/tables/tbl05_end_to_end_micro_pipeline.csv`
- revision-only supporting tables:
  - `01_manuscript_direct_materials/manuscript_assets/tables/tbl06_code_displacement_summary.csv`
  - `01_manuscript_direct_materials/manuscript_assets/tables/tbl07_manylut_matched_10bit.csv`

## Start Here

1. `REVIEWER_GUIDE.md`
2. `01_manuscript_direct_materials/README.md`
3. `01_manuscript_direct_materials/docs/manuscript_asset_map.md`
4. `01_manuscript_direct_materials/docs/result_code_map.md`
5. `02_current_supporting_materials/README.md`
6. `02_current_supporting_materials/docs/material_map.md`

## Layout

- `01_manuscript_direct_materials/`
  - buildable code, canonical checked-in results, and manuscript-ready assets
- `02_current_supporting_materials/`
  - identity diagnostics, revision runs, and implementation notes
