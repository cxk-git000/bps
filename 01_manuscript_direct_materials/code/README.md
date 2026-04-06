# Code Directory

This directory contains the experiment code and the local dependency material
needed to understand or rebuild the study from the curated release.

## Contents

- `.cargo/`
  - local Cargo configuration used during builds
- `src/`
  - Rust binaries for the main experiment driver and the identity diagnostic
- `scripts/`
  - manuscript-facing orchestration scripts and figure/table generators
- `docs/`
  - current implementation notes for the accumulator and PBS path
- `library_modified/`
  - compact copies of the most relevant modified `tfhe-rs` source files
- `tfhe-rs/`
  - vendored `tfhe-rs` workspace used by the local path dependency in
    `Cargo.toml`

Additional current supporting helpers are kept in:

- `../../02_current_supporting_materials/code/`
  - supporting-only script copies and supporting result drivers

## Reading Order

Recommended order for a new reader:

1. `src/README.md`
2. `scripts/README.md`
3. `docs/README.md`
4. `library_modified/README.md`

## Build Note

The crate in this directory is the executable entry point for the repository.
All paths in `Cargo.toml` are arranged so that running Cargo from this
directory resolves the vendored `tfhe-rs` dependency locally.
