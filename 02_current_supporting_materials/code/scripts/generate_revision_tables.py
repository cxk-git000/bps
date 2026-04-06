from __future__ import annotations

from pathlib import Path

import pandas as pd


BUNDLE_ROOT = Path(__file__).resolve().parents[3]
MANUSCRIPT_ROOT = BUNDLE_ROOT / "01_manuscript_direct_materials"
SUPPORT_ROOT = BUNDLE_ROOT / "02_current_supporting_materials"
PAPER_TABLE_DIR = MANUSCRIPT_ROOT / "manuscript_assets" / "tables"

CODEBOOK_PATH = (
    SUPPORT_ROOT
    / "results"
    / "revision_20260405"
    / "14_codebook_displacement_repr3"
    / "codebook_summary.csv"
)
BITWIDTH_ROOT = MANUSCRIPT_ROOT / "results" / "canonical" / "08_bitwidth_sensitivity"

PAIR_LABELS = {
    "tanh_sech2": r"tanh / $\mathrm{sech}^2$",
    "softplus_sigmoid": r"softplus / $\sigma$",
    "gelu_gelu_deriv": r"GELU / GELU$'$",
}

SCHEME_LABELS = {
    "standard_pbs": "Standard PBS",
    "sdr_pbs": "SDR-PBS",
    "many_lut": "Many-LUT",
}

SCHEME_ORDER = ["standard_pbs", "sdr_pbs", "many_lut"]
PAIR_ORDER = ["tanh_sech2", "softplus_sigmoid", "gelu_gelu_deriv"]


def latex_escape(value: object) -> str:
    text = str(value)
    replacements = {
        "&": "\\&",
        "%": "\\%",
        "#": "\\#",
    }
    for source, target in replacements.items():
        text = text.replace(source, target)
    return text


def to_latex_table(df: pd.DataFrame) -> str:
    align = "l" * len(df.columns)
    lines = [
        "\\footnotesize",
        "\\setlength{\\tabcolsep}{4pt}",
        "\\renewcommand{\\arraystretch}{1.08}",
        "\\begin{tabular}{" + align + "}",
        "\\toprule",
        " & ".join(latex_escape(col) for col in df.columns) + " \\\\",
        "\\midrule",
    ]
    for row in df.itertuples(index=False, name=None):
        lines.append(" & ".join(latex_escape(cell) for cell in row) + " \\\\")
    lines.extend(["\\bottomrule", "\\end{tabular}", ""])
    return "\n".join(lines)


def save_table(df: pd.DataFrame, stem: str) -> None:
    PAPER_TABLE_DIR.mkdir(parents=True, exist_ok=True)
    csv_path = PAPER_TABLE_DIR / f"{stem}.csv"
    tex_path = PAPER_TABLE_DIR / f"{stem}.tex"
    df.to_csv(csv_path, index=False)
    tex_path.write_text(to_latex_table(df), encoding="utf-8")


def build_codebook_displacement_table() -> pd.DataFrame:
    df = pd.read_csv(CODEBOOK_PATH).copy()
    df = df[
        df["pair"].isin(PAIR_ORDER) & df["scheme"].isin(["standard_pbs", "sdr_pbs"])
    ].copy()
    df["pair"] = pd.Categorical(df["pair"], categories=PAIR_ORDER, ordered=True)
    df["scheme"] = pd.Categorical(
        df["scheme"], categories=["standard_pbs", "sdr_pbs"], ordered=True
    )
    df = df.sort_values(["pair", "scheme"]).reset_index(drop=True)

    df["Exact (%)"] = (df["exact_recovery"] / df["total_inputs"] * 100.0).map(
        lambda v: f"{v:.2f}"
    )
    df["Both $|\\tau|\\leq 1$ (%)"] = (
        df["joint_le1"] / df["total_inputs"] * 100.0
    ).map(lambda v: f"{v:.2f}")
    df["Both $|\\tau|\\leq 2$ (%)"] = (
        df["joint_le2"] / df["total_inputs"] * 100.0
    ).map(lambda v: f"{v:.2f}")
    df["Avg. $|\\tau_1|$"] = df["mean_code_err1"].map(lambda v: f"{v:.3f}")
    df["Avg. $|\\tau_2|$"] = df["mean_code_err2"].map(lambda v: f"{v:.3f}")

    return df.assign(
        Pair=df["pair"].map(PAIR_LABELS),
        Scheme=df["scheme"].map(SCHEME_LABELS),
        Bits=df["bits"].astype(int),
    )[
        [
            "Pair",
            "Scheme",
            "Bits",
            "Exact (%)",
            "Both $|\\tau|\\leq 1$ (%)",
            "Both $|\\tau|\\leq 2$ (%)",
            "Avg. $|\\tau_1|$",
            "Avg. $|\\tau_2|$",
        ]
    ]


def build_manylut_matched_table() -> pd.DataFrame:
    frames = []
    for pair in PAIR_ORDER:
        path = BITWIDTH_ROOT / pair / "bits_10" / "summary.csv"
        frames.append(pd.read_csv(path))
    df = pd.concat(frames, ignore_index=True)
    df = df[df["scheme"].isin(SCHEME_ORDER)].copy()
    df["pair"] = pd.Categorical(df["pair"], categories=PAIR_ORDER, ordered=True)
    df["scheme"] = pd.Categorical(df["scheme"], categories=SCHEME_ORDER, ordered=True)
    df = df.sort_values(["pair", "scheme"]).reset_index(drop=True)

    worst_rmse = df[["rmse_err1", "rmse_err2"]].max(axis=1)
    if "sigerr_1p0" in df.columns:
        threshold_rate = df["sigerr_1p0"] / df["points"] * 100.0
    else:
        threshold_rate = df["significant_errors"] / df["points"] * 100.0

    return df.assign(
        Pair=df["pair"].map(PAIR_LABELS),
        Scheme=df["scheme"].map(SCHEME_LABELS),
        Bits=df["bits"].astype(int),
        **{
            "Worst-out. RMSE": worst_rmse.map(lambda v: f"{v:.5f}"),
            "Thr. err. (%)": threshold_rate.map(lambda v: f"{v:.2f}"),
            "Latency (ms)": (df["avg_eval_us"] / 1000.0).map(lambda v: f"{v:.2f}"),
        },
    )[
        ["Pair", "Scheme", "Bits", "Worst-out. RMSE", "Thr. err. (%)", "Latency (ms)"]
    ]


def main() -> None:
    save_table(build_codebook_displacement_table(), "tbl06_code_displacement_summary")
    save_table(build_manylut_matched_table(), "tbl07_manylut_matched_10bit")
    print(f"Revision tables written to {PAPER_TABLE_DIR}")


if __name__ == "__main__":
    main()
