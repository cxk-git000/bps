# 12 End-to-End Micro-Pipeline

This folder contains the application-level end-to-end validation runs for the
decryption-end SDR-PBS workflow.

## Purpose

- Validate that the shared blind-rotation benefit persists beyond primitive-level timing.
- Measure downstream score/update error after decrypting the recovered activation-derivative pair.
- Keep the evaluation aligned with the paper's actual security semantics rather than claiming a general composable multi-output interface.

## Contents

- `representative_pairs_1000/`
  - representative-pair end-to-end runs for
    - `softplus_sigmoid`
    - `sigmoid_sigmoid_deriv`
    - `gelu_gelu_deriv`
  - includes `end_to_end_summary.csv`, `runtime_breakdown.csv`, `encoding_coverage.txt`, `run_notes.txt`, and `run.log`
