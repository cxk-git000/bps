# Review Copy Of Modified Library Files

This directory is a compact review-oriented subset of the vendored
`code/tfhe-rs/` workspace.

## Purpose

Reviewers often want to inspect only the specific library files that differ from
an upstream-style baseline. The full vendored dependency is buildable, but this
directory is much faster to browse.

## Files

- `tfhe-rs/tfhe/src/core_crypto/algorithms/lwe_programmable_bootstrapping/mod.rs`
  - contains the accumulator-closure fix

- `tfhe-rs/tfhe/src/core_crypto/algorithms/lwe_programmable_bootstrapping/fft64_pbs.rs`
  - contains the dual-coefficient extraction helper used by SDR-PBS

## Relationship To `code/tfhe-rs/`

- `code/tfhe-rs/` is the buildable vendored dependency
- `code/library_modified/` is the compact explanation layer

