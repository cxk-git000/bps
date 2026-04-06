# 05 Repeatability

This folder contains repeated runs under different deterministic seeds.

## Goal

Measure how stable the reported behavior is across multiple seeded executions.

## Layout

Each seed gets its own directory, for example:

- `allpairs_1000_seed_10101`
- `allpairs_1000_seed_20202`
- `allpairs_1000_seed_30303`

Each run directory contains:

- `run.log`
- `run_notes.txt`
- `runtime_breakdown.csv`
- `summary.csv`

## Why This Folder Matters

This folder distinguishes one-off good behavior from behavior that is stable
under repeated seeded execution.
