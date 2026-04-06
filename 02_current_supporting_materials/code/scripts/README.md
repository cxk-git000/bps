# Supporting Scripts

This directory keeps the scripts that remain useful for review and follow-up
analysis but are not part of the main checked-in canonical pipeline.

## Files

- `run_identity_matrix.ps1`
  - reruns the low-level identity diagnostics
  - uses `../01_manuscript_direct_materials/code/` as the buildable code root
  - writes fresh outputs under `../results/reproduced/01_identity_diagnostics/` by default

- `run_paper_queue.ps1`
  - grouped rerun script for additional queue-style batches
  - uses `../01_manuscript_direct_materials/code/` as the buildable code root
  - writes fresh outputs under `../results/reproduced/paper_queue_batch/` by default

- `generate_revision_tables.py`
  - builds the summary tables `tbl06_*` and `tbl07_*`
  - reads supporting revision results and canonical bitwidth results
  - writes into `../../01_manuscript_direct_materials/manuscript_assets/tables/`
