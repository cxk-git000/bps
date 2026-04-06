# TDSC Rewrite Checklist for sdr_pbs Manuscript

## Overall Verdict

The current manuscript is already on the right track for TDSC, but it still reads slightly more like a crypto-primitive paper than a TDSC systems-and-dependability paper. The best path is not to enlarge the technical claim. Instead, tighten the scope and make the story:

- problem-first, not mechanism-first;
- dependability-first, not only speed-first;
- specialized-and-honest, not overly general;
- empirically grounded, not rhetorically broad.

The strongest paper identity is:

`a dependable shared-input dual-function evaluation path for activation-derivative pairs in TFHE`

not

`a broadly composable general-purpose multi-output programmable bootstrap`.

## Priority Order

### P0: Must Fix Before Submission

1. Shorten and refocus the title.
2. Rewrite the abstract to make the paper problem-first and more numerically balanced.
3. Reduce repeated contribution claims across abstract, introduction, and conclusion.
4. State earlier and more explicitly that this is a specialized decryption-end construction, not a general multi-output ciphertext interface.
5. Strengthen fairness language around `Many-LUT` being reported at a validated `9-bit` point while `Standard PBS` and `sdr_pbs` are at `10-bit`.
6. Soften claims like `preserving similar` when the paper's own table shows a visible thresholded-error increase.
7. Explain why codebook exact-recovery rates are low without implying failure of the continuous-domain method.
8. Add a clearer limitations paragraph in the conclusion.

### P1: Strongly Recommended

1. Make `dependability` a recurring evaluation theme, not only a keyword.
2. Rename the stability subsection to a more TDSC-like framing.
3. Make the runtime-decomposition section explicitly support the main causal claim.
4. Add one stronger sentence explaining why broader external quantitative comparisons are intentionally not apples-to-apples.
5. Add one application-level end-to-end micro-pipeline experiment that consumes the recovered activation-derivative pair in a downstream score/update task.

### P2: Optional but Helpful

1. Add a short threat/scope paragraph near the start of Section 4.
2. Add a one-sentence offline-cost caveat in the complexity/setup discussion.
3. If space allows, mention bitwidth sensitivity earlier in the setup discussion.

## Section-by-Section Action List

### Title

Current status: `partially correct but too long and too mechanism-heavy`

Problem:

- It foregrounds `programmable bootstrapping` and `joint output recovery`, but not the TDSC-valued outcome.
- It sounds like a primitive paper more than a dependable shared-evaluation paper.

Recommended replacement:

```tex
\title{Dependable Shared-Input Dual-Function Evaluation for Activation-Derivative Pairs in TFHE}
```

Safer alternative if you want to keep the method visible:

```tex
\title{Dependable Shared-Input Dual-Function Evaluation for Activation-Derivative Pairs in TFHE via Joint Programmable Bootstrapping}
```

Verdict: `change it`

### Abstract

Current status: `technically solid but slightly overloaded and too promotional`

Main risks:

- too many internal mechanism words in a row;
- too little explicit balance around the error tradeoff;
- not enough emphasis that the paper is specialized and dependable rather than general-purpose.

Recommended replacement:

```tex
\begin{abstract}
Programmable bootstrapping (PBS) in Torus Fully Homomorphic Encryption (TFHE) is the dominant online primitive for encrypted nonlinear evaluation. In privacy-preserving neural computation, however, nonlinear operators often appear as shared-input pairs rather than isolated functions; a common example is an activation and its derivative evaluated at the same encrypted input. The standard TFHE approach handles such pairs with two independent PBS calls, which duplicates the dominant blind-rotation cost. This paper studies a specialized decryption-end dual-function setting in which one encrypted input is used to recover two correlated outputs after a shared blind-rotation path. We propose \emph{\SDRPBSfull} (\emph{\SDRPBS}), which combines a guarded joint accumulator, one blind rotation, dual coefficient extraction, and parity-aware joint decoding. To improve dependable continuous-function behavior for non-periodic functions, we further introduce a \emph{guard-band stabilization} mechanism that shifts the active interval away from the torus seam and converts small boundary perturbations from catastrophic wraparound events into bounded saturation events. We implement \emph{Standard PBS}, \emph{\SDRPBS}, and \emph{Many-LUT} in a unified \texttt{tfhe-rs}-based framework and evaluate them on seven activation-derivative pairs. At the validated operating points used in the main comparison, \SDRPBS reduces mean online latency from 222.6~ms to 111.5~ms relative to Standard PBS while keeping worst-output RMSE nearly unchanged (0.0051 vs.\ 0.0053) and thresholded error rate within the same order of magnitude (0.54\% vs.\ 0.77\%). Codebook, runtime-decomposition, multi-seed, and long-tail results further indicate that the speedup comes from removing one dominant blind-rotation path without introducing a new catastrophic numerical failure mode.
\end{abstract}
```

Verdict: `rewrite`

### Introduction

Current status: `good structure, but still repetitive`

Keep:

- the problem motivation;
- the seam-induced failure discussion;
- the specialized-scope clarification.

Rewrite:

- compress the long literature paragraph in the introduction;
- keep only one sentence of broad context and move the rest to Related Work;
- make the last paragraph before contributions more explicit about the evidence chain.

Recommended change:

- Replace the long paragraph beginning with `Recent work has significantly expanded...` with this shorter version:

```tex
Recent work has substantially expanded the expressiveness and efficiency of programmable and functional bootstrapping in TFHE, including higher-precision PBS, full-domain and multi-value constructions, circuit-oriented extensions, and lower-level optimizations such as lookup compression, automorphism-based traversal, amortized bootstrapping, and improved transform layers. These advances strengthen the TFHE bootstrapping toolbox, but they do not directly target the specific problem studied here: dependable recovery of two correlated outputs from one shared encrypted input under one blind-rotation path.
```

Verdict: `tighten, do not expand`

### End-to-End Validation

Current status: `recommended and now supported by the artifact pipeline`

Why it matters:

- It shows that the speedup survives beyond primitive-level timing.
- It fits the paper's true security semantics: decryption-end recovery followed by downstream use.
- It is much safer than claiming a fully composable multi-output ciphertext interface.

Recommended content:

- Add a short subsection titled `Application-Level End-to-End Micro-Pipeline`.
- Use the normalized downstream tasks
  - `score = 0.65 a_norm + 0.35 d_norm`
  - `update = 0.50 - 0.40 (a_norm - 0.50) d_norm`
- Report the representative-pair end-to-end results already generated in
  - `12_end_to_end_micro_pipeline/representative_pairs_1000/end_to_end_summary.csv`
  - `11_tables/table9_end_to_end_micro_pipeline.csv`
  - `10_figures/fig10_end_to_end_micro_pipeline.pdf`

Recommended claim style:

- Emphasize that the end-to-end experiment validates `application-level benefit under the decryption-end workflow studied in this paper`.
- Do not present it as a generic encrypted training pipeline.

Verdict: `include if space allows; strongly helpful for TDSC framing`

### Contributions List

Current status: `correct but slightly repetitive`

Recommended replacement:

```tex
\begin{itemize}
\item We formulate a specialized shared-input dual-function evaluation problem in TFHE and focus on activation-derivative pairs as a practically relevant instance for privacy-preserving neural computation.
\item We design \emph{\SDRPBS}, a decryption-end dual-output evaluation path that combines a guarded joint accumulator, one blind rotation, dual coefficient extraction, and parity-aware joint decoding.
\item We introduce a guard-band stabilization mechanism that suppresses seam-induced catastrophic outliers for non-periodic continuous functions by replacing potential wraparound failures with bounded boundary saturation.
\item We provide a unified \texttt{tfhe-rs}-based implementation and a systematic empirical evaluation, including accuracy-latency tradeoffs, codebook behavior, runtime decomposition, multi-seed repeatability, and long-tail stability.
\end{itemize}
```

Verdict: `rewrite`

### Related Work

Current status: `mostly correct`

What is correct:

- the three-way organization is reasonable;
- the paper already distinguishes itself from single-function PBS and richer functional-bootstrap lines.

What to improve:

- add one stronger fairness disclaimer that broad external numbers are not directly comparable;
- reduce speculative wording in the training subsection.

Recommended insertion at the end of `Training and Shared-Input Motivation`:

```tex
Because prior works differ substantially in cryptographic setting, implementation maturity, parameterization, output semantics, and hardware environment, we restrict direct quantitative comparison in this paper to \emph{Standard PBS} and \emph{Many-LUT} implemented in the same framework. The broader prior literature is therefore used to position the problem and design space rather than to support direct runtime claims.
```

Verdict: `keep structure, sharpen scope`

### Section 3: Preliminaries and Problem Statement

Current status: `good`

What is already correct:

- the quantization pipeline is clear;
- the specialized problem framing is honest;
- the distinction from a general multi-output interface is helpful.

Minor improvement:

- Add one sentence making clear that the quantization/guard-band model is part of the evaluated system model, not a new cryptographic assumption.

Recommended insertion at the end of Section 3.2:

```tex
These quantization and guard-band choices are part of the evaluated system design rather than new cryptographic assumptions; they define the continuous-domain operating model under which the compared schemes are instantiated.
```

Verdict: `small edit only`

### Section 4: Proposed Scheme

Current status: `technically strong`

Keep:

- accumulator construction;
- parity-aware decoding;
- guard-band mechanism.

Improve:

- state earlier that the construction is specialized and intentionally not generalized;
- make the distinction from Many-LUT slightly cleaner.

Recommended replacement for the paragraph beginning `Compared with Many-LUT...`:

```tex
Compared with Many-LUT, \SDRPBS is not a slot-packing construction. Many-LUT stores the two lookup tables in different accumulator slots and recovers them through slot-wise extraction after a shared blind rotation. \SDRPBS instead uses a local box-4 layout in which the two outputs are encoded as parity-separated codewords within the same interval block. This distinction is central: both methods reuse one blind rotation, but they differ fundamentally in how the two outputs are represented and recovered.
```

Verdict: `keep method, clean wording`

### Section 5: Analysis

Current status: `good, but make the honesty even clearer`

Main suggestion:

- keep the structural-correctness framing;
- do not let the propositions sound stronger than the evidence;
- emphasize measured code displacement rather than perfect symbolic recovery.

This section is already close to the right tone for TDSC.

Verdict: `mostly correct`

### Section 6.1: Implementation and Setup

Current status: `contains the biggest reviewer risk`

Main risk:

- `Many-LUT` is reported at `9-bit`, while `Standard PBS` and `sdr_pbs` are at `10-bit`.
- If this is not framed very carefully, a reviewer can attack the fairness of the main table.

Recommended insertion after the operating-point table paragraph:

```tex
This operating-point choice should be interpreted carefully. The main comparison is fully matched between \emph{Standard PBS} and \emph{\SDRPBS}, which use the same LWE dimension, polynomial size, decomposition parameters, and 10-bit quantization. \emph{Many-LUT} is instead reported at its highest validated stable point under the same 8192-polynomial setting, namely 9-bit quantization. Accordingly, \emph{Many-LUT} serves as a shared-blind-rotation baseline at its validated operating point rather than as a perfectly matched same-bitwidth competitor. We therefore use it to compare recovery structure and practical tradeoffs, while bitwidth sensitivity is examined separately.
```

Verdict: `must add`

### Section 6.2: Main Results

Current status: `good data, slightly risky language`

Main problem:

- `preserving comparable` or `similar` is slightly too strong when thresholded error moves from `0.54%` to `0.77%`.

Safer wording changes:

- change `preserving comparable thresholded error rates and worst-output RMSE`
  to
  `keeping worst-output RMSE nearly unchanged and thresholded error within the same order of magnitude`

- change `These results indicate that sdr_pbs improves efficiency primarily by removing one dominant blind-rotation path rather than by trading away substantial continuous-domain accuracy.`
  to
  `These results indicate that sdr_pbs improves efficiency primarily by removing one dominant blind-rotation path while incurring only a modest accuracy tradeoff at the validated operating point.`

Verdict: `soften claims`

### Section 6.3: Codebook-Recovery Consistency

Current status: `honest but under-explained`

Main risk:

- exact recovery rates around `39%-44%` can alarm reviewers if not interpreted carefully.

Recommended insertion after the first paragraph:

```tex
These exact-recovery percentages should be interpreted as a stringent discrete criterion rather than as a proxy for application-level failure. In the present setting, small torus-domain perturbations may shift the recovered code by a few nearby levels even when the final dequantized continuous-domain error remains very small. For this reason, we report codebook exact recovery together with maximum code displacement and continuous-domain metrics, and we do not treat less-than-perfect exact-code recovery as evidence of a distinct numerical failure mode.
```

Verdict: `must add`

### Section 6.5: Stability and Robustness

Current status: `strong`

Recommended change:

- rename the subsection to:

```tex
\subsection{Dependability Evaluation}
\label{subsec:stability_results}
```

This matches TDSC better than `Stability and Robustness` and aligns with the paper's strongest non-speed selling point.

Verdict: `rename recommended`

### Conclusion

Current status: `good but should state limitations more explicitly`

Recommended replacement for the final paragraph:

```tex
The present work focuses on a specialized but practically important setting: two correlated outputs evaluated on the same encoded encrypted input and jointly recovered at the decryptor under the standard TFHE workflow. Accordingly, the paper does not claim a general composable multi-output ciphertext interface, and the current construction also relies on a structured box-4 layout and validated operating points. Extending the approach toward richer shared-input nonlinear operators, broader output interfaces, and lower offline memory or key costs remains important future work.
```

Verdict: `rewrite`

## Recommended Replacement Snippets

### Title

```tex
\title{Dependable Shared-Input Dual-Function Evaluation for Activation-Derivative Pairs in TFHE}
```

### Abstract Result Sentence

```tex
At the validated operating points used in the main comparison, \SDRPBS reduces mean online latency from 222.6~ms to 111.5~ms relative to Standard PBS while keeping worst-output RMSE nearly unchanged (0.0051 vs.\ 0.0053) and thresholded error rate within the same order of magnitude (0.54\% vs.\ 0.77\%).
```

### Fairness Disclaimer

```tex
This operating-point choice should be interpreted carefully. The main comparison is fully matched between \emph{Standard PBS} and \emph{\SDRPBS}, which use the same LWE dimension, polynomial size, decomposition parameters, and 10-bit quantization. \emph{Many-LUT} is instead reported at its highest validated stable point under the same 8192-polynomial setting, namely 9-bit quantization. Accordingly, \emph{Many-LUT} serves as a shared-blind-rotation baseline at its validated operating point rather than as a perfectly matched same-bitwidth competitor.
```

### Codebook Interpretation

```tex
These exact-recovery percentages should be interpreted as a stringent discrete criterion rather than as a proxy for application-level failure. Small torus-domain perturbations may shift the recovered code by a few nearby levels even when the final dequantized continuous-domain error remains small. We therefore report codebook exact recovery together with maximum code displacement and continuous-domain metrics, and we do not treat less-than-perfect exact-code recovery as evidence of a distinct numerical failure mode.
```

### Limitation Statement

```tex
The present work focuses on a specialized but practically important setting: two correlated outputs evaluated on the same encoded encrypted input and jointly recovered at the decryptor under the standard TFHE workflow. Accordingly, the paper does not claim a general composable multi-output ciphertext interface, and the current construction also relies on a structured box-4 layout and validated operating points.
```

## Correctness Summary

### What is already correct

- The paper's real problem is clear.
- The specialized scope is already partially acknowledged.
- The guard-band mechanism is a strong TDSC-facing contribution.
- The evaluation already contains the right kinds of evidence: runtime decomposition, codebook checks, multi-seed tests, and long-tail tests.

### What is risky if left unchanged

- The title is too internal-mechanism oriented.
- The abstract overclaims similarity in accuracy.
- The Many-LUT fairness issue is under-explained.
- The codebook exact-recovery numbers can be misread.
- The conclusion does not foreground limitations strongly enough.

### Final judgment

After the changes above, the paper's positioning becomes much more correct for TDSC. The technical core does not need to change. The key fix is editorial and argumentative: make the manuscript read like a dependable encrypted-computation paper with a specialized TFHE design, rather than like a broad new primitive paper.
