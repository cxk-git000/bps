# Project Structure Overview

This note summarizes the repository structure.

The active project material centers on:

- `code/`
  - Rust experiment drivers, support scripts, and the vendored dependency tree
- `results/canonical/`
  - current checked-in result summaries, figures, and tables
- `paper_assets/`
  - compact manuscript figures, tables, and snippet files
- `docs/`
  - reproduction flow, experiment map, and library-change notes

The repository is split into:

- `01_manuscript_direct_materials/`
  - main code, canonical results, and manuscript assets
- `02_current_supporting_materials/`
  - diagnostics, revision runs, implementation notes, and supporting scripts

The vendored `tfhe-rs` subtree remains part of the package because the
experiment crate depends on it directly.
