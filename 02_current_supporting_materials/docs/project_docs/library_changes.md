# Library Changes

The experiment crate uses a vendored `tfhe-rs` dependency under `code/tfhe-rs/`.

The most relevant modified files are mirrored in `code/library_modified/`.

## Directly Used Changes

- `generate_programmable_bootstrap_glwe_lut`
  - used by both the main experiment path and the diagnostic path
- `blind_rotate_and_extract_two_coefficients`
  - used by the SDR-PBS path in both the main experiment and the diagnostics

## Why This Matters

These changes are not decorative copies. They correspond to the actual vendored sources used by the local path dependency in `code/Cargo.toml`.

## Additional Notes

- current implementation notes are available in `../implementation_notes/`
- the buildable mirrored files remain under `../../01_manuscript_direct_materials/code/library_modified/`
