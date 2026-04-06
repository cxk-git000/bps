# Source Files

This directory contains the two Rust binaries that drive the published
experiments.

## Files

- `main.rs`
  - Main experiment driver for the paper-scale evaluations.
  - Implements Standard PBS, SDR-PBS, and many-LUT evaluation flows.
  - Produces summary metrics, runtime breakdowns, and codebook outputs.

- `diagnose.rs`
  - Low-level diagnostic binary used before the main experiments.
  - Focuses on identity-style checks to isolate accumulator layout and blind
    rotation behavior.
  - Used to generate the `01_identity_diagnostics` artifact folder.

## Relationship Between The Two Binaries

- `diagnose.rs` answers: "Is the low-level PBS path behaving structurally as
  expected?"
- `main.rs` answers: "How do the three schemes behave on actual function-pair
  evaluation tasks?"

That split is important because the repository is organized around diagnosis
first and large-scale comparison second.
