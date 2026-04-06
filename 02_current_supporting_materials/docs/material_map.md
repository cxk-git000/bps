# Other Material Map

This material does not directly appear in the current manuscript body, but it still has a clear purpose.

## 1. Identity diagnostics

- result folder: `results/01_identity_diagnostics/`
- purpose:
  - low-level PBS structure checks
  - confirms accumulator layout and blind-rotation behavior
- code:
  - buildable source: `../01_manuscript_direct_materials/code/src/diagnose.rs`
  - supporting-section mirror: `code/src/diagnose.rs`
- script:
  - `code/scripts/run_identity_matrix.ps1`

## 2. Codebook displacement revision runs

### 2.1 All pairs

- result folder: `results/revision_20260405/14_codebook_displacement_allpairs/`
- purpose:
  - all-pair displacement check from the revision stage
  - useful as extra validation, not as a direct manuscript figure/table source
- code:
  - `../01_manuscript_direct_materials/code/src/main.rs`
- script:
  - driven by the main experiment logic
  - no separate batch script is included in this clean bundle

### 2.2 Representative three pairs

- result folder: `results/revision_20260405/14_codebook_displacement_repr3/`
- purpose:
  - compact displacement summary for three representative pairs
- code:
  - `../01_manuscript_direct_materials/code/src/main.rs`
- script:
  - `code/scripts/generate_revision_tables.py`
  - writes `../01_manuscript_direct_materials/manuscript_assets/tables/tbl06_*`
  - writes `../01_manuscript_direct_materials/manuscript_assets/tables/tbl07_*`

## 3. Additional current supporting material

- `code/scripts/run_paper_queue.ps1`
  - grouped queue-style rerun script for additional batch execution
  - uses `../01_manuscript_direct_materials/code/` as the buildable code root

- `docs/implementation_notes/`
  - current implementation notes for accumulator repair and PBS behavior

- `notes/manuscript_notes/`
  - current manuscript rewrite notes

- `notes/workspace_notes/`
  - current workspace-level investigation notes

- `results/revision_20260405/README.md`
  - overview of the revision-only checked-in result folders

## Note

These folders are kept for review and follow-up analysis, but they are not direct source assets for the manuscript body.
