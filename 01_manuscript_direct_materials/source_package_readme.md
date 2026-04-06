# Repository Structure Note

This note describes the project structure used to assemble this repository.

Source directory:

- `BPS_LOCAL_FINAL_20260403/`

The source material was organized around:

- `code/`
  - current experiment code, scripts, vendored dependency, and build files
- `results/canonical/`
  - current checked-in result folders
- `paper_assets/`
  - current figures, tables, and manuscript snippets
- `manuscript_notes/`
  - current manuscript rewrite notes
- `workspace_notes/`
  - current implementation and investigation notes
- `docs/`
  - current reproduction and experiment-map documentation

In this repository, that material is organized into:

- `01_manuscript_direct_materials/`
  - main code, results, and manuscript assets
- `02_current_supporting_materials/`
  - current diagnostics, revision results, working notes, and supporting scripts

Intentional exclusions:

- Git metadata
- build outputs under `code/target/`
- regenerated scratch outputs
- archive-only documentation and archive-only directories
- temporary files under `tmp/`
