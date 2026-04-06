# 09 Tail Stability 100k

This folder contains long-run stability evaluations.

## Goal

Measure whether rare large-error events remain suppressed over substantially
longer runs than the main 10000-point evaluation.

## Layout

Selected function pairs each have a dedicated 100000-point directory containing:

- `run.log`
- `run_notes.txt`
- `runtime_breakdown.csv`
- `summary.csv`

## Why This Folder Matters

Short and medium runs can hide rare events. This folder is the tail-behavior
check that complements the main result and the repeatability study.
