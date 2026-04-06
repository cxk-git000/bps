# Current Source Snapshot

This note describes the current project snapshot that served as the source for
this curated release.

Source snapshot:

- `BPS_LOCAL_FINAL_20260403/`

In the source snapshot, the active project material was organized around:

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

In this curated release, that source snapshot has been reorganized into:

- `01_manuscript_direct_materials/`
  - manuscript-facing code, results, and manuscript assets
- `02_current_supporting_materials/`
  - current diagnostics, revision results, working notes, and supporting scripts

Intentional exclusions from the curated release:

- Git metadata
- build outputs under `code/target/`
- regenerated scratch outputs
- archive-only documentation and archive-only directories
- temporary files under `tmp/`
