# Revision Results

This directory contains current revision-only result folders that are kept for
review but are not part of the main canonical claim path.

## Included Folders

- `14_codebook_displacement_allpairs/`
  - all-pair displacement check from the revision stage

- `14_codebook_displacement_repr3/`
  - compact three-pair displacement summary used to build `tbl06_*`

## Related Script

- `../../code/scripts/generate_revision_tables.py`
  - reads `14_codebook_displacement_repr3/codebook_summary.csv`
  - combines it with canonical bitwidth summaries
  - writes summary tables into
    `../../../01_manuscript_direct_materials/manuscript_assets/tables/`
