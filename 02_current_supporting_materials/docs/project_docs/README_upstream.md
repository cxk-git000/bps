# Current Project Overview

This note summarizes the current project snapshot used to assemble the curated
release.

The active source snapshot centers on:

- `code/`
  - Rust experiment drivers, support scripts, and the vendored dependency tree
- `results/canonical/`
  - current checked-in result summaries, figures, and tables
- `paper_assets/`
  - compact manuscript figures, tables, and snippet files
- `docs/`
  - reproduction flow, experiment map, and library-change notes

In the curated release, the same current material is split into:

- `01_manuscript_direct_materials/`
  - manuscript-facing code, canonical results, manuscript assets, and manuscript files
- `02_current_supporting_materials/`
  - diagnostics, revision runs, implementation notes, and supporting scripts

The vendored `tfhe-rs` subtree remains part of the package because the
experiment crate depends on it directly.
