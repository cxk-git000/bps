from __future__ import annotations

import argparse
from pathlib import Path
import re

import matplotlib as mpl
import matplotlib.gridspec as gridspec
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from matplotlib.patches import Circle, FancyArrowPatch, FancyBboxPatch, Rectangle
from matplotlib.ticker import PercentFormatter


REPO_ROOT = Path(__file__).resolve().parents[2]
CANONICAL_ROOT = REPO_ROOT / "results" / "canonical"
ARCHIVE_ROOT = REPO_ROOT / "results" / "archive" / "original_artifact_tree"
OUT_DIR = REPO_ROOT / "paper_assets" / "figures"
TABLE_OUT_DIR = REPO_ROOT / "paper_assets" / "tables"

DOUBLE_COL_IN = 7.16
SINGLE_COL_IN = 3.5

TEXT = "#1f2933"
GRID = "#d9dee6"
EDGE = "#314252"
LIGHT = "#f4f7fb"
BLUE = "#3F72AF"
RED = "#D65A63"
TEAL = "#5BAFA8"
AMBER = "#E9A03B"
GRAY = "#AAB4C3"
DARK_GRAY = "#6B7785"

SCHEME_ORDER = ["standard_pbs", "sdr_pbs", "many_lut"]
SCHEME_LABELS = {
    "standard_pbs": "Standard PBS",
    "sdr_pbs": "SDR-PBS",
    "many_lut": "Many-LUT",
}
SCHEME_COLORS = {
    "standard_pbs": BLUE,
    "sdr_pbs": RED,
    "many_lut": TEAL,
}
SCHEME_MARKERS = {
    "standard_pbs": "o",
    "sdr_pbs": "s",
    "many_lut": "^",
}
SCHEME_LINES = {
    "standard_pbs": "-",
    "sdr_pbs": "--",
    "many_lut": "-.",
}
PAIR_ORDER = [
    "tanh_sech2",
    "sigmoid_sigmoid_deriv",
    "softplus_sigmoid",
    "swish_swish_deriv",
    "gelu_gelu_deriv",
    "elu_elu_deriv",
    "mish_mish_deriv",
]
PAIR_LABELS = {
    "tanh_sech2": "tanh /\n$\\mathrm{sech}^2$",
    "sigmoid_sigmoid_deriv": "sigmoid /\n$\\sigma'$",
    "softplus_sigmoid": "softplus /\n$\\sigma$",
    "swish_swish_deriv": "swish /\nswish'",
    "gelu_gelu_deriv": "GELU /\nGELU'",
    "elu_elu_deriv": "ELU /\nELU'",
    "mish_mish_deriv": "mish /\nmish'",
}
ABLATION_PAIRS = ["sigmoid_sigmoid_deriv", "softplus_sigmoid", "tanh_sech2"]
ABLATION_LABELS = {
    "sigmoid_sigmoid_deriv": r"sigmoid / $\sigma'$",
    "softplus_sigmoid": r"softplus / $\sigma$",
    "tanh_sech2": r"tanh / $\mathrm{sech}^2$",
}
CONFIG_ORDER = ["factor1_offset0", "factor2_offset0", "factor2_offset256", "factor2_offset512"]
CONFIG_LABELS = {
    "factor1_offset0": "F1/O0",
    "factor2_offset0": "F2/O0",
    "factor2_offset256": "F2/O256",
    "factor2_offset512": "F2/O512",
}


mpl.rcParams.update(
    {
        "figure.dpi": 180,
        "savefig.dpi": 600,
        "savefig.format": "pdf",
        "figure.facecolor": "white",
        "axes.facecolor": "white",
        "pdf.fonttype": 42,
        "ps.fonttype": 42,
        "font.family": "serif",
        "font.serif": [
            "Times New Roman",
            "Times",
            "Nimbus Roman",
            "Liberation Serif",
            "DejaVu Serif",
        ],
        "mathtext.fontset": "stix",
        "axes.labelcolor": TEXT,
        "axes.labelsize": 9.0,
        "axes.titlesize": 9.2,
        "axes.titleweight": "normal",
        "axes.linewidth": 0.8,
        "xtick.color": TEXT,
        "xtick.labelsize": 7.8,
        "ytick.color": TEXT,
        "ytick.labelsize": 7.8,
        "text.color": TEXT,
        "legend.fontsize": 7.8,
        "legend.frameon": False,
        "legend.handlelength": 2.0,
        "legend.columnspacing": 1.2,
        "lines.linewidth": 1.6,
        "lines.markersize": 4.4,
    }
)

METHOD_TITLE_FS = 8.1
METHOD_NOTE_FS = 6.8
METHOD_TEXT_FS = 6.9
RESULT_TITLE_FS = 8.4
RESULT_TICK_FS = 7.6


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate manuscript figure assets.")
    parser.add_argument("--canonical-root", type=Path, default=CANONICAL_ROOT)
    parser.add_argument("--archive-root", type=Path, default=ARCHIVE_ROOT)
    parser.add_argument("--out-dir", type=Path, default=OUT_DIR)
    parser.add_argument("--table-dir", type=Path, default=TABLE_OUT_DIR)
    return parser.parse_args()


def configure_paths(args: argparse.Namespace) -> None:
    global CANONICAL_ROOT, ARCHIVE_ROOT, OUT_DIR, TABLE_OUT_DIR
    CANONICAL_ROOT = args.canonical_root.resolve()
    ARCHIVE_ROOT = args.archive_root.resolve()
    OUT_DIR = args.out_dir.resolve()
    TABLE_OUT_DIR = args.table_dir.resolve()
    if not CANONICAL_ROOT.exists():
        raise FileNotFoundError(f"canonical result root not found: {CANONICAL_ROOT}")
    if not ARCHIVE_ROOT.exists():
        raise FileNotFoundError(f"archive result root not found: {ARCHIVE_ROOT}")


def ensure_dirs() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    TABLE_OUT_DIR.mkdir(parents=True, exist_ok=True)


def read_csv(path: Path) -> pd.DataFrame:
    return pd.read_csv(path)


def normalize_scheme_names(df: pd.DataFrame) -> pd.DataFrame:
    result = df.copy()
    return result


def save_figure(fig: plt.Figure, stem: str) -> None:
    for suffix in (".pdf", ".png"):
        kwargs = {"bbox_inches": "tight", "pad_inches": 0.02}
        if suffix == ".png":
            kwargs["dpi"] = 600
        fig.savefig(OUT_DIR / f"{stem}{suffix}", **kwargs)


def save_table(df: pd.DataFrame, stem: str) -> None:
    csv_path = TABLE_OUT_DIR / f"{stem}.csv"
    tex_path = TABLE_OUT_DIR / f"{stem}.tex"
    df.to_csv(csv_path, index=False)
    tex_path.write_text(frame_to_latex(df), encoding="utf-8")


def frame_to_latex(df: pd.DataFrame) -> str:
    cols = list(df.columns)
    align = "l" * len(cols)
    lines = [
        "\\small",
        "\\setlength{\\tabcolsep}{4pt}",
        "\\begin{tabular}{" + align + "}",
        "\\toprule",
        " & ".join(latex_escape(c) for c in cols) + " \\\\",
        "\\midrule",
    ]
    for row in df.itertuples(index=False, name=None):
        lines.append(" & ".join(latex_escape(format_cell(v)) for v in row) + " \\\\")
    lines.extend(["\\bottomrule", "\\end{tabular}", ""])
    return "\n".join(lines)


def latex_escape(text: object) -> str:
    value = str(text)
    replacements = {
        "\\": "\\textbackslash{}",
        "&": "\\&",
        "%": "\\%",
        "$": "\\$",
        "#": "\\#",
        "_": "\\_",
        "{": "\\{",
        "}": "\\}",
    }
    for src, dst in replacements.items():
        value = value.replace(src, dst)
    return value


def format_cell(value: object) -> str:
    if isinstance(value, float):
        if abs(value) >= 100:
            return f"{value:.2f}"
        if abs(value) >= 1:
            return f"{value:.4f}"
        if value == 0:
            return "0"
        return f"{value:.4g}"
    return str(value)


def format_axes(ax: plt.Axes) -> None:
    ax.grid(axis="y", color=GRID, linestyle="-", linewidth=0.65, alpha=0.75)
    ax.set_axisbelow(True)
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.tick_params(axis="both", which="major", length=3.0, width=0.8, pad=3)
    ax.margins(x=0.03)


def line_style(scheme: str) -> dict[str, object]:
    return {
        "color": SCHEME_COLORS[scheme],
        "marker": SCHEME_MARKERS[scheme],
        "linestyle": SCHEME_LINES[scheme],
        "markerfacecolor": SCHEME_COLORS[scheme],
        "markeredgecolor": "white",
        "markeredgewidth": 0.9,
    }


def add_panel_labels(axes: object) -> None:
    labels = "abcdefghijklmnopqrstuvwxyz"
    for idx, ax in enumerate(np.atleast_1d(axes).ravel()):
        ax.text(
            -0.10,
            1.02,
            f"({labels[idx]})",
            transform=ax.transAxes,
            fontsize=8.1,
            color=DARK_GRAY,
            va="bottom",
            ha="left",
        )


def rounded_box(
    ax: plt.Axes,
    x: float,
    y: float,
    w: float,
    h: float,
    text: str,
    *,
    fc: str,
    ec: str = EDGE,
    fs: float = 8.6,
    lw: float = 1.0,
    tc: str = TEXT,
) -> None:
    patch = FancyBboxPatch(
        (x, y),
        w,
        h,
        boxstyle="round,pad=0.012,rounding_size=0.025",
        linewidth=lw,
        edgecolor=ec,
        facecolor=fc,
    )
    ax.add_patch(patch)
    ax.text(x + w / 2.0, y + h / 2.0, text, ha="center", va="center", fontsize=fs, color=tc)


def draw_block_row(
    ax: plt.Axes,
    x0: float,
    y0: float,
    labels: list[str],
    *,
    box_w: float,
    box_h: float,
    fills: list[str] | None = None,
    edgecolors: list[str] | None = None,
    linewidths: list[float] | None = None,
    text_colors: list[str] | None = None,
    fontsize: float = 6.8,
) -> list[tuple[float, float]]:
    centers: list[tuple[float, float]] = []
    fills = fills or ["white"] * len(labels)
    edgecolors = edgecolors or [EDGE] * len(labels)
    linewidths = linewidths or [0.85] * len(labels)
    text_colors = text_colors or [TEXT] * len(labels)

    for idx, label in enumerate(labels):
        xi = x0 + idx * box_w
        ax.add_patch(
            Rectangle(
                (xi, y0),
                box_w,
                box_h,
                facecolor=fills[idx],
                edgecolor=edgecolors[idx],
                linewidth=linewidths[idx],
            )
        )
        ax.text(
            xi + box_w / 2.0,
            y0 + box_h / 2.0,
            label,
            ha="center",
            va="center",
            fontsize=fontsize,
            color=text_colors[idx],
        )
        centers.append((xi + box_w / 2.0, y0 + box_h / 2.0))

    return centers


def add_arrow(ax: plt.Axes, start: tuple[float, float], end: tuple[float, float], *, color: str = EDGE, lw: float = 1.3, connectionstyle: str = "arc3") -> None:
    arrow = FancyArrowPatch(
        start,
        end,
        arrowstyle="-|>",
        mutation_scale=10,
        linewidth=lw,
        color=color,
        connectionstyle=connectionstyle,
    )
    ax.add_patch(arrow)


def load_main_summary() -> pd.DataFrame:
    df = normalize_scheme_names(
        read_csv(CANONICAL_ROOT / "04_main_guarded_all_pairs_10000" / "summary.csv")
    )
    df["error_rate"] = df["significant_errors"].astype(float) / df["points"].astype(float)
    df["worst_rmse"] = df[["rmse_err1", "rmse_err2"]].max(axis=1)
    df["latency_ms"] = df["avg_eval_us"] / 1000.0
    return df


def load_prepost() -> pd.DataFrame:
    current = normalize_scheme_names(
        read_csv(CANONICAL_ROOT / "04_main_guarded_all_pairs_10000" / "summary.csv")
    )
    old = pd.concat(
        [
            normalize_scheme_names(
                read_csv(ARCHIVE_ROOT / "02_pre_fix_reference_10000" / "tanh_10000" / "summary.csv")
            ),
            normalize_scheme_names(
                read_csv(ARCHIVE_ROOT / "02_pre_fix_reference_10000" / "sigmoid_10000" / "summary.csv")
            ),
        ],
        ignore_index=True,
    )
    targets = [
        ("tanh_sech2", "standard_pbs"),
        ("tanh_sech2", "sdr_pbs"),
        ("sigmoid_sigmoid_deriv", "standard_pbs"),
        ("sigmoid_sigmoid_deriv", "sdr_pbs"),
        ("softplus_sigmoid", "standard_pbs"),
        ("softplus_sigmoid", "sdr_pbs"),
    ]
    rows = []
    for pair, scheme in targets:
        old_row = old[(old["pair"] == pair) & (old["scheme"] == scheme)].iloc[0]
        new_row = current[(current["pair"] == pair) & (current["scheme"] == scheme)].iloc[0]
        rows.append(
            {
                "pair": pair,
                "scheme": scheme,
                "old_worst_max": max(old_row["max_err1"], old_row["max_err2"]),
                "new_worst_max": max(new_row["max_err1"], new_row["max_err2"]),
            }
        )
    return pd.DataFrame(rows)


def load_timing() -> pd.DataFrame:
    multiseed = pd.concat(
        [
            normalize_scheme_names(read_csv(path))
            for path in CANONICAL_ROOT.glob("05_repeatability_multiseed_1000/allpairs_1000_seed_*/summary.csv")
        ],
        ignore_index=True,
    )
    summary = (
        multiseed.groupby("scheme", as_index=False)
        .agg(
            avg_input_us=("avg_input_us", "mean"),
            avg_core_us=("avg_core_us", "mean"),
            avg_decode_us=("avg_decode_us", "mean"),
            avg_eval_us=("avg_eval_us", "mean"),
        )
        .set_index("scheme")
        .reindex(SCHEME_ORDER)
        .dropna(how="all")
        .reset_index()
    )
    summary["avg_eval_ms"] = summary["avg_eval_us"] / 1000.0
    return summary


def load_guard_ablation() -> pd.DataFrame:
    rows = []
    for path in CANONICAL_ROOT.glob("06_guardband_ablation/*/*/summary.csv"):
        df = normalize_scheme_names(read_csv(path))
        df["ablation_pair"] = path.parents[1].name
        df["ablation_config"] = path.parent.name
        rows.append(df)
    result = pd.concat(rows, ignore_index=True)
    result["worst_max"] = result[["max_err1", "max_err2"]].max(axis=1)
    result["error_rate"] = result["significant_errors"].astype(float) / result["points"].astype(float)
    return result


def parse_scheme_setup() -> pd.DataFrame:
    text = (CANONICAL_ROOT / "04_main_guarded_all_pairs_10000" / "run_notes.txt").read_text(encoding="utf-8")
    rows = []
    for prefix, scheme_key in [
        ("standard=", "standard_pbs"),
        ("sdr_pbs=", "sdr_pbs"),
        ("many_lut=", "many_lut"),
    ]:
        line = next((line for line in text.splitlines() if line.startswith(prefix)), None)
        if line is None:
            continue
        bits = int(re.search(r"bits: (\d+)", line).group(1))
        lwe_dim = int(re.search(r"LweDimension\((\d+)\)", line).group(1))
        poly_size = int(re.search(r"PolynomialSize\((\d+)\)", line).group(1))
        pbs_base_log = int(re.search(r"DecompositionBaseLog\((\d+)\)", line).group(1))
        pbs_level = int(re.search(r"DecompositionLevelCount\((\d+)\)", line).group(1))
        guard_match = re.search(r"InputGuardLayout \{ total_factor: (\d+), input_offset: (\d+) \}", line)
        many_match = re.search(r"ManyLutLayout \{ total_factor: (\d+), slot_count: (\d+), input_offset: (\d+), used_slots: \[([0-9, ]+)\] \}", line)
        if guard_match:
            layout = f"guard factor={guard_match.group(1)}, offset={guard_match.group(2)}"
        elif many_match:
            layout = (
                f"{many_match.group(2)} slots, factor={many_match.group(1)}, "
                f"offset={many_match.group(3)}, used={many_match.group(4)}"
            )
        else:
            layout = "n/a"
        rows.append(
            {
                "scheme": SCHEME_LABELS[scheme_key],
                "bits": bits,
                "lwe_dim": lwe_dim,
                "poly_size": poly_size,
                "pbs_base_log": pbs_base_log,
                "pbs_level": pbs_level,
                "layout": layout,
            }
        )
    return pd.DataFrame(rows)


def make_paper_tables() -> None:
    setup_df = parse_scheme_setup().rename(
        columns={
            "scheme": "Scheme",
            "bits": "Bits",
            "lwe_dim": "LWE dim.",
            "poly_size": "Poly size",
            "pbs_base_log": "PBS base",
            "pbs_level": "PBS levels",
            "layout": "Layout",
        }
    )
    save_table(setup_df, "tbl01_scheme_setup")

    main_df = load_main_summary().copy()
    main_df["error_rate_pct"] = main_df["error_rate"] * 100.0
    main_df["worst_rmse"] = main_df["worst_rmse"].map(lambda v: f"{v:.4f}")
    main_df["latency_ms"] = main_df["latency_ms"].map(lambda v: f"{v:.2f}")
    main_df["error_rate_pct"] = main_df["error_rate_pct"].map(lambda v: f"{v:.2f}")
    main_df = main_df.assign(
        scheme=main_df["scheme"].map(SCHEME_LABELS),
        pair=main_df["pair"].map(lambda p: PAIR_LABELS[p].replace("\n", " ")),
    )[
        ["pair", "scheme", "bits", "error_rate_pct", "worst_rmse", "latency_ms"]
    ].rename(
        columns={
            "pair": "Pair",
            "scheme": "Scheme",
            "bits": "Bits",
            "error_rate_pct": "Error rate (%)",
            "worst_rmse": "Worst RMSE",
            "latency_ms": "Time (ms)",
        }
    )
    save_table(main_df, "tbl02_main_results")

    timing_df = load_timing().copy()
    timing_df = timing_df.assign(
        scheme=timing_df["scheme"].map(SCHEME_LABELS),
        avg_input_us=timing_df["avg_input_us"].map(lambda v: f"{v:.2f}"),
        avg_core_us=timing_df["avg_core_us"].map(lambda v: f"{v:.2f}"),
        avg_decode_us=timing_df["avg_decode_us"].map(lambda v: f"{v:.2f}"),
        total_ms=timing_df["avg_eval_ms"].map(lambda v: f"{v:.2f}"),
    )[
        ["scheme", "avg_input_us", "avg_core_us", "avg_decode_us", "total_ms"]
    ].rename(
        columns={
            "scheme": "Scheme",
            "avg_input_us": "Input (us)",
            "avg_core_us": "Core (us)",
            "avg_decode_us": "Decode (us)",
            "total_ms": "Total (ms)",
        }
    )
    save_table(timing_df, "tbl03_timing_summary")

    codebook_df = read_csv(CANONICAL_ROOT / "07_codebook_recovery_validation" / "allpairs" / "codebook_summary.csv").copy()
    codebook_df["exact_recovery_rate"] = codebook_df["exact_recovery"] / codebook_df["total_inputs"]
    codebook_df["exact_recovery_pct"] = codebook_df["exact_recovery_rate"] * 100.0
    codebook_df["exact_recovery_pct"] = codebook_df["exact_recovery_pct"].map(lambda v: f"{v:.2f}")
    codebook_df["avg_eval_us"] = codebook_df["avg_eval_us"].map(lambda v: f"{v:.2f}")
    codebook_df = codebook_df.assign(
        scheme=codebook_df["scheme"].map(SCHEME_LABELS),
        pair=codebook_df["pair"].map(lambda p: PAIR_LABELS[p].replace("\n", " ")),
    )[
        ["pair", "scheme", "bits", "exact_recovery_pct", "max_code_err1", "max_code_err2", "avg_eval_us"]
    ].rename(
        columns={
            "pair": "Pair",
            "scheme": "Scheme",
            "bits": "Bits",
            "exact_recovery_pct": "Exact (%)",
            "max_code_err1": "Max err f1",
            "max_code_err2": "Max err f2",
            "avg_eval_us": "Time (us)",
        }
    )
    save_table(codebook_df, "tbl04_codebook_summary")


def plot_method_overview() -> None:
    fig, axes = plt.subplots(
        1,
        3,
        figsize=(DOUBLE_COL_IN, 2.85),
        gridspec_kw={"width_ratios": [1.0, 1.02, 1.20]},
    )
    for ax in axes:
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        ax.axis("off")

    ax0, ax1, ax2 = axes

    ax0.text(0.08, 0.88, "continuous input", fontsize=METHOD_TITLE_FS, color=DARK_GRAY, ha="left", va="center")
    block_labels = [r"$\cdots$", r"$k{-}1$", r"$k$", r"$k{+}1$", r"$k{+}2$", r"$\cdots$"]
    fills = ["#F3F6FA", "#E8EEF7", "#D9E7F8", "#E8EEF7", "#E8EEF7", "#F3F6FA"]
    edgecolors = [EDGE, EDGE, BLUE, EDGE, EDGE, EDGE]
    linewidths = [0.85, 0.85, 1.25, 0.85, 0.85, 0.85]
    centers = draw_block_row(
        ax0,
        0.08,
        0.47,
        block_labels,
        box_w=0.13,
        box_h=0.16,
        fills=fills,
        edgecolors=edgecolors,
        linewidths=linewidths,
        fontsize=7.2,
    )
    ax0.plot([centers[2][0]], [0.79], marker="o", color=RED, markersize=4.5)
    add_arrow(ax0, (centers[2][0], 0.76), (centers[2][0], 0.64), color=EDGE, lw=1.0)
    ax0.text(centers[2][0], 0.84, "x", fontsize=7.6, color=TEXT, ha="center")
    ax0.text(0.08, 0.30, r"$k = \mathrm{encode\_input\_index}(x)$", fontsize=METHOD_NOTE_FS, color=DARK_GRAY, ha="left")

    ax1.text(0.06, 0.88, "shared output codebooks", fontsize=METHOD_TITLE_FS, color=DARK_GRAY, ha="left", va="center")
    ax1.text(0.06, 0.64, r"$f_1$", fontsize=7.6, color=BLUE, ha="left", va="center")
    ax1.text(0.06, 0.36, r"$f_2$", fontsize=7.6, color=RED, ha="left", va="center")
    labels_f1 = [r"$q_1[k{-}1]$", r"$q_1[k]$", r"$q_1[k{+}1]$", r"$q_1[k{+}2]$"]
    labels_f2 = [r"$q_2[k{-}1]$", r"$q_2[k]$", r"$q_2[k{+}1]$", r"$q_2[k{+}2]$"]
    fills_f1 = ["#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF"]
    fills_f2 = ["#FFFFFF", "#FFFFFF", "#FFFFFF", "#FFFFFF"]
    edges_f1 = [EDGE, BLUE, EDGE, EDGE]
    edges_f2 = [EDGE, RED, EDGE, EDGE]
    draw_block_row(ax1, 0.17, 0.55, labels_f1, box_w=0.17, box_h=0.15, fills=fills_f1, edgecolors=edges_f1, linewidths=[0.85, 1.25, 0.85, 0.85], fontsize=6.8)
    draw_block_row(ax1, 0.17, 0.27, labels_f2, box_w=0.17, box_h=0.15, fills=fills_f2, edgecolors=edges_f2, linewidths=[0.85, 1.25, 0.85, 0.85], fontsize=6.8)
    ax1.text(0.425, 0.78, r"shared index $k$", fontsize=METHOD_NOTE_FS, color=DARK_GRAY, ha="center")

    ax2.text(0.08, 0.88, "scheme-specific plaintext", fontsize=METHOD_TITLE_FS, color=DARK_GRAY, ha="left", va="center")
    ax2.text(0.08, 0.78, r"given shared $k$", fontsize=METHOD_TEXT_FS, color=TEXT, ha="left")
    scheme_rows = [
        (0.68, BLUE, "Standard PBS", r"$m=(k+o)\Delta$"),
        (0.47, RED, "SDR-PBS", r"$m=4(k+o)\Delta$"),
        (0.26, TEAL, "Many-LUT", r"$m=(k+o)\Delta$"),
    ]
    for y, color, scheme, formula in scheme_rows:
        ax2.add_patch(Rectangle((0.08, y - 0.03), 0.02, 0.06, facecolor=color, edgecolor="none"))
        ax2.text(0.14, y, scheme, fontsize=METHOD_TITLE_FS, color=DARK_GRAY, ha="left", va="center")
        rounded_box(ax2, 0.50, y - 0.058, 0.38, 0.116, formula, fc="white", ec=color, fs=7.6, lw=1.05)
    ax2.text(0.50, 0.10, r"$o$ = input offset, $\Delta$ = torus step", fontsize=METHOD_NOTE_FS, color=DARK_GRAY, ha="left")

    add_panel_labels(axes)
    fig.subplots_adjust(left=0.05, right=0.99, top=0.94, bottom=0.10, wspace=0.12)
    save_figure(fig, "fig01_system_overview")
    plt.close(fig)


def plot_input_encoding() -> None:
    fig, axes = plt.subplots(1, 2, figsize=(DOUBLE_COL_IN, 2.35), gridspec_kw={"width_ratios": [1.05, 1.0]})
    ax0, ax1 = axes

    ax0.set_xlim(-4.2, 4.2)
    ax0.set_ylim(0, 1)
    ax0.spines["top"].set_visible(False)
    ax0.spines["right"].set_visible(False)
    ax0.spines["left"].set_visible(False)
    ax0.set_yticks([])
    ax0.set_xticks([-4, -2, 0, 2, 4])
    ax0.set_xlabel("continuous input x", labelpad=2)
    for left in range(-4, 4):
        fc = "#EEF2F7" if left != 1 else "#D9E7F8"
        ax0.add_patch(Rectangle((left, 0.42), 1.0, 0.20, facecolor=fc, edgecolor=EDGE, linewidth=0.7))
    ax0.plot([1.35], [0.80], marker="o", color=RED, markersize=4.5)
    ax0.annotate("", xy=(1.35, 0.65), xytext=(1.35, 0.78), arrowprops=dict(arrowstyle="-|>", color=EDGE, lw=0.9))
    ax0.text(1.35, 0.87, "x", ha="center", va="bottom", fontsize=7.6, color=DARK_GRAY)
    ax0.text(1.50, 0.52, "k", ha="center", va="center", fontsize=8.0, color=TEXT)

    ax1.set_xlim(0, 1)
    ax1.set_ylim(0, 1)
    ax1.axis("off")
    ax1.text(0.08, 0.83, "shared code index", fontsize=7.2, color=DARK_GRAY, ha="left")
    for y, label, color in [(0.58, "f1 codebook", BLUE), (0.28, "f2 codebook", RED)]:
        ax1.text(0.08, y + 0.10, label, fontsize=7.1, color=DARK_GRAY, ha="left")
        for idx in range(6):
            x0 = 0.32 + idx * 0.10
            fc = "#F7FAFD"
            if idx == 3:
                fc = "#EEF4FB" if color == BLUE else "#FCEDEE"
            ax1.add_patch(Rectangle((x0, y), 0.075, 0.12, facecolor=fc, edgecolor=EDGE, linewidth=0.7))
            ax1.text(x0 + 0.0375, y + 0.06, "k" if idx == 3 else "·", ha="center", va="center", fontsize=7.5, color=TEXT if idx == 3 else DARK_GRAY)
        ax1.add_patch(Rectangle((0.32 + 3 * 0.10, y), 0.075, 0.12, facecolor="none", edgecolor=color, linewidth=1.2))
    ax1.annotate("", xy=(0.657, 0.70), xytext=(0.657, 0.83), arrowprops=dict(arrowstyle="-|>", color=EDGE, lw=0.9))
    ax1.annotate("", xy=(0.657, 0.40), xytext=(0.657, 0.53), arrowprops=dict(arrowstyle="-|>", color=EDGE, lw=0.9))

    add_panel_labels(axes)
    fig.subplots_adjust(left=0.07, right=0.985, bottom=0.18, top=0.92, wspace=0.18)
    save_figure(fig, "fig02_input_encoding")
    plt.close(fig)


def plot_scheme_comparison() -> None:
    fig, axes = plt.subplots(1, 3, figsize=(DOUBLE_COL_IN, 2.65))
    scheme_specs = [
        ("Standard PBS", BLUE, "2 independent blind rotations"),
        ("SDR-PBS", RED, "1 blind rotation + dual extraction"),
        ("Many-LUT", TEAL, "1 shared blind rotation"),
    ]
    for ax, (title, color, footnote) in zip(axes, scheme_specs):
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        ax.axis("off")
        ax.add_patch(Circle((0.10, 0.52), 0.05, facecolor="#F7FAFD", edgecolor=EDGE, linewidth=1.0))
        ax.text(0.10, 0.52, r"$c_k$", ha="center", va="center", fontsize=9.5)
        ax.text(0.50, 0.91, title, ha="center", va="center", fontsize=METHOD_TITLE_FS, color=TEXT)
        add_arrow(ax, (0.15, 0.52), (0.24, 0.52))
        if title == "Standard PBS":
            split_x = 0.25
            add_arrow(ax, (split_x, 0.52), (0.34, 0.66))
            add_arrow(ax, (split_x, 0.52), (0.34, 0.38))
            ax.add_patch(Rectangle((0.35, 0.57), 0.16, 0.16, facecolor=color, alpha=0.14, edgecolor=color, linewidth=1.0))
            ax.add_patch(Rectangle((0.35, 0.29), 0.16, 0.16, facecolor=color, alpha=0.14, edgecolor=color, linewidth=1.0))
            ax.text(0.43, 0.65, "PBS 1", ha="center", va="center", fontsize=7.3)
            ax.text(0.43, 0.37, "PBS 2", ha="center", va="center", fontsize=7.3)
            add_arrow(ax, (0.51, 0.65), (0.82, 0.68))
            add_arrow(ax, (0.51, 0.37), (0.82, 0.34))
        elif title == "SDR-PBS":
            ax.add_patch(Rectangle((0.31, 0.40), 0.30, 0.24, facecolor=color, alpha=0.14, edgecolor=color, linewidth=1.0))
            ax.text(0.46, 0.53, "rotate", ha="center", va="center", fontsize=7.5)
            ax.text(0.46, 0.43, "dual extract", ha="center", va="center", fontsize=7.1, color=DARK_GRAY)
            add_arrow(ax, (0.61, 0.57), (0.82, 0.68))
            add_arrow(ax, (0.61, 0.47), (0.82, 0.34))
        else:
            ax.add_patch(Rectangle((0.30, 0.40), 0.34, 0.24, facecolor=color, alpha=0.12, edgecolor=color, linewidth=1.0))
            ax.add_patch(Rectangle((0.30, 0.40), 0.17, 0.24, facecolor="none", edgecolor=color, linewidth=0.9))
            ax.text(0.385, 0.52, "slot 0", ha="center", va="center", fontsize=7.2)
            ax.text(0.555, 0.52, "slot 1", ha="center", va="center", fontsize=7.2)
            add_arrow(ax, (0.64, 0.57), (0.82, 0.68))
            add_arrow(ax, (0.64, 0.47), (0.82, 0.34))
        rounded_box(ax, 0.82, 0.62, 0.10, 0.09, "f1", fc="#EEF4FB", ec=BLUE, fs=7.4, lw=0.9)
        rounded_box(ax, 0.82, 0.29, 0.10, 0.09, "f2", fc="#FCEDEE", ec=RED, fs=7.4, lw=0.9)
        ax.text(0.50, 0.12, footnote, ha="center", va="center", fontsize=METHOD_NOTE_FS, color=DARK_GRAY)

    add_panel_labels(axes)
    fig.subplots_adjust(left=0.05, right=0.99, top=0.94, bottom=0.10, wspace=0.18)
    save_figure(fig, "fig03_scheme_comparison")
    plt.close(fig)


def plot_input_encoding() -> None:
    fig, axes = plt.subplots(
        3,
        1,
        figsize=(DOUBLE_COL_IN, 5.10),
        gridspec_kw={"height_ratios": [1.0, 1.15, 1.0]},
    )
    for ax in axes:
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        ax.axis("off")

    ax0, ax1, ax2 = axes
    title_x = 0.06
    note_x = 0.94
    row_x = 0.18
    label_x = 0.06
    std_box_w = 0.118
    std_box_h = 0.16
    std_total_w = 6 * std_box_w

    ax0.text(title_x, 0.90, "Standard PBS", fontsize=METHOD_TITLE_FS, color=TEXT, ha="left", va="center")
    ax0.text(note_x, 0.90, "two guarded accumulators", fontsize=METHOD_NOTE_FS, color=DARK_GRAY, ha="right", va="center")
    ax0.text(label_x, 0.63, "A1", fontsize=METHOD_TEXT_FS, color=BLUE, ha="left", va="center")
    ax0.text(label_x, 0.33, "A2", fontsize=METHOD_TEXT_FS, color=RED, ha="left", va="center")
    std_f1 = [r"$q_1[0]$", r"$q_1[k{-}1]$", r"$q_1[k]$", r"$q_1[k{+}1]$", r"$q_1[k{+}2]$", r"$q_1[L{-}1]$"]
    std_f2 = [r"$q_2[0]$", r"$q_2[k{-}1]$", r"$q_2[k]$", r"$q_2[k{+}1]$", r"$q_2[k{+}2]$", r"$q_2[L{-}1]$"]
    std_fill_f1 = ["#EAF1FB", "#F7FAFD", "#EEF4FB", "#F7FAFD", "#F7FAFD", "#EAF1FB"]
    std_fill_f2 = ["#FEF0F1", "#FEF7F7", "#FCEDEE", "#FEF7F7", "#FEF7F7", "#FEF0F1"]
    std_edges_f1 = [EDGE, EDGE, BLUE, EDGE, EDGE, EDGE]
    std_edges_f2 = [EDGE, EDGE, RED, EDGE, EDGE, EDGE]
    draw_block_row(
        ax0,
        row_x,
        0.53,
        std_f1,
        box_w=std_box_w,
        box_h=std_box_h,
        fills=std_fill_f1,
        edgecolors=std_edges_f1,
        linewidths=[0.85, 0.85, 1.25, 0.85, 0.85, 0.85],
        fontsize=6.9,
    )
    draw_block_row(
        ax0,
        row_x,
        0.23,
        std_f2,
        box_w=std_box_w,
        box_h=std_box_h,
        fills=std_fill_f2,
        edgecolors=std_edges_f2,
        linewidths=[0.85, 0.85, 1.25, 0.85, 0.85, 0.85],
        fontsize=6.9,
    )

    ax1.text(title_x, 0.90, "SDR-PBS", fontsize=METHOD_TITLE_FS, color=TEXT, ha="left", va="center")
    ax1.text(
        note_x,
        0.90,
        "single accumulator, 4 coefficients per interval",
        fontsize=METHOD_NOTE_FS,
        color=DARK_GRAY,
        ha="right",
        va="center",
    )
    start_x = row_x
    group_gap = 0.03
    box_w = 0.054
    labels = [r"$2q_1$", r"$2q_2{+}1$", r"$2q_1$", r"$2q_2{+}1$"]
    for idx, tag in enumerate([r"$k{-}1$", r"$k$", r"$k{+}1$"]):
        group_x = start_x + idx * (4 * box_w + group_gap)
        edge = RED if idx == 1 else EDGE
        lw = 1.25 if idx == 1 else 0.9
        fills = ["#EEF4FB", "#FCEDEE", "#EEF4FB", "#FCEDEE"]
        edges = [edge] * 4
        widths = [lw] * 4
        draw_block_row(
            ax1,
            group_x,
            0.42,
            labels,
            box_w=box_w,
            box_h=0.18,
            fills=fills,
            edgecolors=edges,
            linewidths=widths,
            fontsize=5.6,
        )
        ax1.add_patch(
            Rectangle(
                (group_x, 0.42),
                4 * box_w,
                0.18,
                facecolor="none",
                edgecolor=edge,
                linewidth=lw,
            )
        )
        ax1.text(
            group_x + 2 * box_w,
            0.66,
            f"interval {tag}",
            fontsize=METHOD_NOTE_FS,
            color=DARK_GRAY,
            ha="center",
        )
    ax1.text(row_x, 0.22, "coeff 0 / 2 store even-coded f1 values", fontsize=METHOD_TEXT_FS, color=BLUE, ha="left")
    ax1.text(row_x, 0.12, "coeff 1 / 3 store odd-coded f2 values", fontsize=METHOD_TEXT_FS, color=RED, ha="left")

    ax2.text(title_x, 0.90, "Many-LUT", fontsize=METHOD_TITLE_FS, color=TEXT, ha="left", va="center")
    ax2.text(note_x, 0.90, "one accumulator, two output slots", fontsize=METHOD_NOTE_FS, color=DARK_GRAY, ha="right", va="center")
    ax2.add_patch(Rectangle((row_x - 0.03, 0.17), std_total_w + 0.06, 0.58, facecolor="white", edgecolor=EDGE, linewidth=0.9))
    ax2.text(label_x, 0.61, "slot 0", fontsize=METHOD_TEXT_FS, color=BLUE, ha="left", va="center")
    ax2.text(label_x, 0.31, "slot 1", fontsize=METHOD_TEXT_FS, color=RED, ha="left", va="center")
    draw_block_row(
        ax2,
        row_x,
        0.52,
        std_f1,
        box_w=std_box_w,
        box_h=0.14,
        fills=std_fill_f1,
        edgecolors=std_edges_f1,
        linewidths=[0.85, 0.85, 1.25, 0.85, 0.85, 0.85],
        fontsize=6.8,
    )
    draw_block_row(
        ax2,
        row_x,
        0.24,
        std_f2,
        box_w=std_box_w,
        box_h=0.14,
        fills=std_fill_f2,
        edgecolors=std_edges_f2,
        linewidths=[0.85, 0.85, 1.25, 0.85, 0.85, 0.85],
        fontsize=6.8,
    )

    add_panel_labels(axes)
    fig.subplots_adjust(left=0.06, right=0.99, top=0.94, bottom=0.09, hspace=0.22)
    save_figure(fig, "fig02_input_encoding")
    plt.close(fig)


def plot_guardband_layout() -> None:
    fig, axes = plt.subplots(3, 1, figsize=(DOUBLE_COL_IN, 4.60))
    for ax in axes:
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        ax.axis("off")

    ax0, ax1, ax2 = axes
    row_x = 0.16
    box_w = 0.142
    box_h = 0.18
    label_x = 0.08

    ax0.text(0.08, 0.88, "unguarded placement", fontsize=METHOD_TITLE_FS, color=TEXT, ha="left", va="center")
    blocks0 = [r"$q[k{-}1]$", r"$q[k]$", r"$q[k{+}1]$", "wrap", r"$\cdots$"]
    fills0 = ["#EEF4FB", "#D9E7F8", "#EEF4FB", "#FCEDEE", "#F5F7FA"]
    edges0 = [EDGE, BLUE, EDGE, RED, EDGE]
    draw_block_row(
        ax0,
        row_x,
        0.40,
        blocks0,
        box_w=box_w,
        box_h=box_h,
        fills=fills0,
        edgecolors=edges0,
        linewidths=[0.9, 1.2, 0.9, 1.1, 0.9],
        fontsize=7.0,
    )
    ax0.text(row_x, 0.22, "active interval sits next to the torus seam", fontsize=METHOD_TEXT_FS, color=DARK_GRAY, ha="left")
    ax0.text(row_x + 3.5 * box_w, 0.64, "seam", fontsize=METHOD_NOTE_FS, color=RED, ha="center")

    ax1.text(0.08, 0.88, "guarded standard / SDR-PBS", fontsize=METHOD_TITLE_FS, color=TEXT, ha="left", va="center")
    blocks1 = [r"$q[0]$", r"$q[k{-}1]$", r"$q[k]$", r"$q[k{+}1]$", r"$q[L{-}1]$"]
    fills1 = ["#FCEDEE", "#EEF4FB", "#D9E7F8", "#EEF4FB", "#E7F4F1"]
    edges1 = [EDGE, EDGE, BLUE, EDGE, EDGE]
    draw_block_row(
        ax1,
        row_x,
        0.40,
        blocks1,
        box_w=box_w,
        box_h=box_h,
        fills=fills1,
        edgecolors=edges1,
        linewidths=[0.85, 0.85, 1.2, 0.85, 0.85],
        fontsize=7.0,
    )
    ax1.text(row_x, 0.22, "replicated boundary values absorb local displacement", fontsize=METHOD_TEXT_FS, color=DARK_GRAY, ha="left")
    center_k = row_x + 2.5 * box_w
    add_arrow(ax1, (center_k, 0.76), (center_k, 0.61), color=BLUE, lw=1.0)
    ax1.text(center_k, 0.79, r"input uses $k+o$", fontsize=METHOD_TEXT_FS, color=BLUE, ha="center")

    ax2.text(0.08, 0.88, "guarded Many-LUT slots", fontsize=METHOD_TITLE_FS, color=TEXT, ha="left", va="center")
    ax2.text(label_x, 0.63, "slot 0", fontsize=METHOD_TEXT_FS, color=BLUE, ha="left", va="center")
    ax2.text(label_x, 0.31, "slot 1", fontsize=METHOD_TEXT_FS, color=RED, ha="left", va="center")
    slot_blocks = [r"$q[0]$", r"$q[k{-}1]$", r"$q[k]$", r"$q[k{+}1]$", r"$q[L{-}1]$"]
    slot_fills = ["#FCEDEE", "#EEF4FB", "#D9E7F8", "#EEF4FB", "#E7F4F1"]
    slot_edges = [EDGE, EDGE, BLUE, EDGE, EDGE]
    draw_block_row(
        ax2,
        row_x,
        0.54,
        slot_blocks,
        box_w=box_w,
        box_h=0.16,
        fills=slot_fills,
        edgecolors=slot_edges,
        linewidths=[0.85, 0.85, 1.2, 0.85, 0.85],
        fontsize=7.0,
    )
    draw_block_row(
        ax2,
        row_x,
        0.24,
        [r"$r[0]$", r"$r[k{-}1]$", r"$r[k]$", r"$r[k{+}1]$", r"$r[L{-}1]$"],
        box_w=box_w,
        box_h=0.16,
        fills=["#FCEDEE", "#FEF7F7", "#FCEDEE", "#FEF7F7", "#E7F4F1"],
        edgecolors=[EDGE, EDGE, RED, EDGE, EDGE],
        linewidths=[0.85, 0.85, 1.2, 0.85, 0.85],
        fontsize=7.0,
    )
    ax2.text(row_x, 0.09, "each slot keeps independent left / right guards", fontsize=METHOD_TEXT_FS, color=DARK_GRAY, ha="left")

    add_panel_labels(axes)
    fig.subplots_adjust(left=0.06, right=0.99, top=0.95, bottom=0.09, hspace=0.22)
    save_figure(fig, "fig04_guardband_layout")
    plt.close(fig)


def plot_main_results() -> None:
    df = load_main_summary()
    pairs = [pair for pair in PAIR_ORDER if pair in set(df["pair"])]
    labels = [PAIR_LABELS[pair] for pair in pairs]
    x = np.arange(len(pairs))
    width = 0.22
    display_colors = {
        "standard_pbs": BLUE,
        "sdr_pbs": RED,
        "many_lut": AMBER,
    }

    fig = plt.figure(figsize=(DOUBLE_COL_IN, 5.55))
    gs = gridspec.GridSpec(2, 2, figure=fig, height_ratios=[1.0, 1.12], hspace=0.42, wspace=0.26)
    ax0 = fig.add_subplot(gs[0, 0])
    ax1 = fig.add_subplot(gs[0, 1])
    ax2 = fig.add_subplot(gs[1, :])

    for idx, scheme in enumerate(SCHEME_ORDER):
        local = df[df["scheme"] == scheme].set_index("pair").reindex(pairs)
        xpos = x + (idx - 1) * width
        bar_kwargs = {
            "color": display_colors[scheme],
            "edgecolor": EDGE,
            "linewidth": 0.45,
            "alpha": 0.96,
            "label": SCHEME_LABELS[scheme],
        }
        ax0.bar(xpos, local["error_rate"] * 100.0, width, **bar_kwargs)
        ax1.bar(xpos, local["worst_rmse"], width, **bar_kwargs)
        ax2.bar(xpos, local["latency_ms"], width, **bar_kwargs)

    for ax in (ax0, ax1, ax2):
        ax.set_xticks(x)
        format_axes(ax)
    ax0.set_xticklabels([])
    ax1.set_xticklabels([])
    ax2.set_xticklabels(labels)
    ax2.tick_params(axis="x", labelsize=RESULT_TICK_FS, pad=3)

    ax0.set_ylabel("Thresholded error rate (%)")
    ax0.yaxis.set_major_formatter(PercentFormatter(decimals=1))
    ax0.set_title("Thresholded error rate", pad=6, fontsize=RESULT_TITLE_FS, color=TEXT)

    ax1.set_ylabel("Worst-output RMSE")
    ax1.set_title("Worst output RMSE", pad=6, fontsize=RESULT_TITLE_FS, color=TEXT)

    ax2.set_ylabel("Average online latency (ms)")
    ax2.set_title("Online latency", pad=6, fontsize=RESULT_TITLE_FS, color=TEXT)

    add_panel_labels([ax0, ax1, ax2])
    handles, legend_labels = ax0.get_legend_handles_labels()
    fig.legend(handles, legend_labels, loc="upper center", ncol=3, bbox_to_anchor=(0.5, 0.985))
    fig.text(
        0.5,
        0.905,
        "Standard/SDR-PBS: 10-bit; Many-LUT: 9-bit operating point",
        ha="center",
        va="center",
        fontsize=METHOD_NOTE_FS,
        color=DARK_GRAY,
    )
    fig.subplots_adjust(left=0.08, right=0.985, bottom=0.11, top=0.86)
    save_figure(fig, "fig05_main_results")
    plt.close(fig)


def plot_repair_effect() -> None:
    prepost = load_prepost()
    pairs = ["tanh_sech2", "sigmoid_sigmoid_deriv", "softplus_sigmoid"]
    labels = [PAIR_LABELS[pair] for pair in pairs]
    width = 0.32

    fig, axes = plt.subplots(1, 2, figsize=(DOUBLE_COL_IN, 3.00), sharey=True)
    pre_color = RED
    post_color = BLUE
    for ax, scheme in zip(axes, ["standard_pbs", "sdr_pbs"]):
        local = prepost[prepost["scheme"] == scheme].set_index("pair").reindex(pairs)
        x = np.arange(len(pairs))
        ax.bar(x - width / 2, local["old_worst_max"], width, color=pre_color, edgecolor=EDGE, linewidth=0.45, label="Pre-fix")
        ax.bar(x + width / 2, local["new_worst_max"], width, color=post_color, edgecolor=EDGE, linewidth=0.45, label="Guarded")
        ax.set_xticks(x)
        ax.set_xticklabels(labels)
        ax.set_yscale("log")
        ax.set_title(SCHEME_LABELS[scheme], pad=6, fontsize=RESULT_TITLE_FS, color=TEXT)
        format_axes(ax)

    axes[0].set_ylabel("Worst-case absolute error")
    handles, legend_labels = axes[0].get_legend_handles_labels()
    add_panel_labels(axes)
    fig.legend(handles, legend_labels, loc="upper center", ncol=2, bbox_to_anchor=(0.5, 1.02))
    fig.subplots_adjust(left=0.09, right=0.985, bottom=0.15, top=0.84, wspace=0.12)
    save_figure(fig, "fig06_repair_effect")
    plt.close(fig)


def plot_timing_breakdown() -> None:
    timing = load_timing()
    fig, axes = plt.subplots(1, 2, figsize=(DOUBLE_COL_IN, 3.05))
    x = np.arange(len(timing))
    labels = [SCHEME_LABELS[s] for s in timing["scheme"]]
    display_colors = {
        "standard_pbs": BLUE,
        "sdr_pbs": RED,
        "many_lut": AMBER,
    }

    ax0, ax1 = axes
    ax0.bar(
        x,
        timing["avg_eval_ms"],
        color=[display_colors[s] for s in timing["scheme"]],
        edgecolor=EDGE,
        linewidth=0.45,
        alpha=0.96,
    )
    ax0.set_xticks(x)
    ax0.set_xticklabels(labels)
    ax0.set_ylabel("Average online latency (ms)")
    ax0.set_title("Total latency", pad=6, fontsize=RESULT_TITLE_FS, color=TEXT)
    format_axes(ax0)
    ax0.tick_params(axis="x", labelsize=RESULT_TICK_FS, pad=3)

    width = 0.22
    ax1.bar(x - width, timing["avg_input_us"], width, color=AMBER, edgecolor=EDGE, linewidth=0.45, label="Input")
    ax1.bar(x, timing["avg_core_us"], width, color=BLUE, edgecolor=EDGE, linewidth=0.45, label="Core")
    ax1.bar(x + width, timing["avg_decode_us"], width, color=RED, edgecolor=EDGE, linewidth=0.45, label="Decode")
    ax1.set_xticks(x)
    ax1.set_xticklabels(labels)
    ax1.set_ylabel("Stage time (us)")
    ax1.set_yscale("log")
    ax1.set_title("Stage latency", pad=6, fontsize=RESULT_TITLE_FS, color=TEXT)
    format_axes(ax1)
    ax1.tick_params(axis="x", labelsize=RESULT_TICK_FS, pad=3)

    handles, legend_labels = ax1.get_legend_handles_labels()
    add_panel_labels(axes)
    fig.legend(handles, legend_labels, loc="upper center", ncol=3, bbox_to_anchor=(0.5, 1.02))
    fig.subplots_adjust(left=0.09, right=0.985, bottom=0.17, top=0.84, wspace=0.36)
    save_figure(fig, "fig07_timing_breakdown")
    plt.close(fig)


def plot_guardband_ablation() -> None:
    df = load_guard_ablation()
    fig, axes = plt.subplots(2, len(ABLATION_PAIRS), figsize=(DOUBLE_COL_IN, 4.90), sharex=False)

    x = np.arange(len(CONFIG_ORDER))
    for col, pair in enumerate(ABLATION_PAIRS):
        top = axes[0, col]
        bottom = axes[1, col]
        local_pair = df[df["ablation_pair"] == pair]
        pair_max_rm = 0.0
        pair_min_rm = None
        pair_max_rate = 0.0
        for scheme in ["standard_pbs", "sdr_pbs"]:
            local = (
                local_pair[local_pair["scheme"] == scheme]
                .set_index("ablation_config")
                .reindex(CONFIG_ORDER)
            )
            worst_vals = local["worst_max"].astype(float)
            rate_vals = local["error_rate"].astype(float) * 100.0
            top.plot(x, worst_vals, label=SCHEME_LABELS[scheme], **line_style(scheme))
            bottom.plot(x, rate_vals, label=SCHEME_LABELS[scheme], **line_style(scheme))
            if not worst_vals.empty:
                pair_max_rm = max(pair_max_rm, float(worst_vals.max()))
                pos = worst_vals[worst_vals > 0]
                if not pos.empty:
                    candidate = float(pos.min())
                    pair_min_rm = candidate if pair_min_rm is None else min(pair_min_rm, candidate)
            if not rate_vals.empty:
                pair_max_rate = max(pair_max_rate, float(rate_vals.max()))
        top.set_title(ABLATION_LABELS[pair], pad=5, fontsize=RESULT_TITLE_FS, color=TEXT)
        top.set_xticks(x)
        top.set_xticklabels([CONFIG_LABELS[c] for c in CONFIG_ORDER])
        bottom.set_xticks(x)
        bottom.set_xticklabels([CONFIG_LABELS[c] for c in CONFIG_ORDER])
        top.tick_params(axis="x", labelsize=RESULT_TICK_FS, pad=2)
        bottom.tick_params(axis="x", labelsize=RESULT_TICK_FS, pad=2)
        top.set_yscale("log")
        if pair_min_rm is not None and pair_max_rm > 0:
            top.set_ylim(pair_min_rm * 0.82, pair_max_rm * 1.25)
        if pair_max_rate <= 1.0:
            local_rate_top = 1.0
            bottom.set_yticks([0.0, 0.5, 1.0])
        elif pair_max_rate <= 3.0:
            local_rate_top = 3.0
            bottom.set_yticks([0.0, 1.0, 2.0, 3.0])
        else:
            local_rate_top = np.ceil(pair_max_rate)
        bottom.set_ylim(0.0, local_rate_top)
        bottom.yaxis.set_major_formatter(PercentFormatter(decimals=1))
        format_axes(top)
        format_axes(bottom)

    axes[0, 0].set_ylabel("Worst-case error")
    axes[1, 0].set_ylabel("Error rate (%)")
    handles, legend_labels = axes[0, 0].get_legend_handles_labels()
    add_panel_labels(axes)
    fig.legend(handles, legend_labels, loc="upper center", ncol=2, bbox_to_anchor=(0.5, 1.01))
    fig.text(
        0.5,
        0.045,
        "F = guard factor, O = input offset",
        ha="center",
        va="center",
        fontsize=METHOD_NOTE_FS,
        color=DARK_GRAY,
    )
    fig.subplots_adjust(left=0.10, right=0.985, bottom=0.11, top=0.90, wspace=0.42, hspace=0.36)
    save_figure(fig, "fig08_guardband_ablation")
    plt.close(fig)


def main() -> None:
    args = parse_args()
    configure_paths(args)
    ensure_dirs()
    plot_method_overview()
    plot_input_encoding()
    plot_scheme_comparison()
    plot_guardband_layout()
    plot_main_results()
    plot_repair_effect()
    plot_timing_breakdown()
    plot_guardband_ablation()
    make_paper_tables()
    print(f"Paper figure assets written to {OUT_DIR}")
    print(f"Paper table assets written to {TABLE_OUT_DIR}")


if __name__ == "__main__":
    main()
