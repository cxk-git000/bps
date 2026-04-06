from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from matplotlib.ticker import PercentFormatter


REPO_ROOT = Path(__file__).resolve().parents[2]
CANONICAL_ROOT = REPO_ROOT / "results" / "canonical"
ARCHIVE_ROOT = REPO_ROOT / "results" / "archive" / "original_artifact_tree"
FIG_DIR = REPO_ROOT / "results" / "regenerated" / "10_paper_figures"
TABLE_DIR = REPO_ROOT / "results" / "regenerated" / "11_paper_tables"

IEEE_SINGLE_COL_IN = 3.5
IEEE_DOUBLE_COL_IN = 7.16

TEXT_COLOR = "#1f2933"
GRID_COLOR = "#d9dee6"
AX_FACE = "#ffffff"
BLUE = "#3F72AF"
RED = "#D65A63"
TEAL = "#5BAFA8"
AMBER = "#E9A03B"
EDGE_COLOR = "#314252"
PANEL_LABEL_COLOR = "#6B7785"
PREPOST_COLORS = {
    "pre": RED,
    "post": BLUE,
}

mpl.rcParams.update(
    {
        "figure.dpi": 180,
        "savefig.dpi": 600,
        "savefig.format": "pdf",
        "figure.facecolor": "white",
        "axes.facecolor": AX_FACE,
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
        "axes.labelcolor": TEXT_COLOR,
        "axes.labelsize": 9.0,
        "axes.titlesize": 9.2,
        "axes.titleweight": "normal",
        "axes.linewidth": 0.8,
        "xtick.color": TEXT_COLOR,
        "xtick.labelsize": 7.8,
        "ytick.color": TEXT_COLOR,
        "ytick.labelsize": 7.8,
        "text.color": TEXT_COLOR,
        "legend.fontsize": 7.8,
        "legend.frameon": False,
        "legend.handlelength": 2.0,
        "legend.columnspacing": 1.2,
        "lines.linewidth": 1.6,
        "lines.markersize": 4.4,
    }
)


def ensure_dirs() -> None:
    FIG_DIR.mkdir(parents=True, exist_ok=True)
    TABLE_DIR.mkdir(parents=True, exist_ok=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate figures and tables from canonical BPS results."
    )
    parser.add_argument(
        "--canonical-root",
        type=Path,
        default=CANONICAL_ROOT,
        help="Root containing canonical result folders.",
    )
    parser.add_argument(
        "--archive-root",
        type=Path,
        default=ARCHIVE_ROOT,
        help="Root containing archived historical result folders.",
    )
    parser.add_argument(
        "--figure-dir",
        type=Path,
        default=FIG_DIR,
        help="Output directory for regenerated figures.",
    )
    parser.add_argument(
        "--table-dir",
        type=Path,
        default=TABLE_DIR,
        help="Output directory for regenerated tables.",
    )
    return parser.parse_args()


def configure_paths(args: argparse.Namespace) -> None:
    global CANONICAL_ROOT, ARCHIVE_ROOT, FIG_DIR, TABLE_DIR

    CANONICAL_ROOT = args.canonical_root.resolve()
    ARCHIVE_ROOT = args.archive_root.resolve()
    FIG_DIR = args.figure_dir.resolve()
    TABLE_DIR = args.table_dir.resolve()

    if not CANONICAL_ROOT.exists():
        raise FileNotFoundError(f"canonical result root not found: {CANONICAL_ROOT}")
    if not ARCHIVE_ROOT.exists():
        raise FileNotFoundError(f"archive result root not found: {ARCHIVE_ROOT}")


def read_csv(path: Path) -> pd.DataFrame:
    return normalize_scheme_names(pd.read_csv(path))


def normalize_scheme_names(df: pd.DataFrame) -> pd.DataFrame:
    result = df.copy()
    return result


def scheme_order() -> list[str]:
    return ["standard_pbs", "sdr_pbs", "many_lut"]


def scheme_label_map() -> dict[str, str]:
    return {
        "standard_pbs": "Standard PBS",
        "sdr_pbs": "SDR-PBS",
        "many_lut": "Many-LUT",
    }


def pair_order() -> list[str]:
    return [
        "tanh_sech2",
        "sigmoid_sigmoid_deriv",
        "softplus_sigmoid",
        "swish_swish_deriv",
        "gelu_gelu_deriv",
        "elu_elu_deriv",
        "mish_mish_deriv",
    ]


def pair_display_map(multiline: bool = True) -> dict[str, str]:
    if multiline:
        return {
            "tanh_sech2": "tanh /\n$\\mathrm{sech}^2$",
            "sigmoid_sigmoid_deriv": "sigmoid /\n$\\sigma'$",
            "softplus_sigmoid": "softplus /\n$\\sigma$",
            "swish_swish_deriv": "swish /\nswish'",
            "gelu_gelu_deriv": "GELU /\nGELU'",
            "elu_elu_deriv": "ELU /\nELU'",
            "mish_mish_deriv": "mish /\nmish'",
        }
    return {
        "tanh_sech2": "tanh / $\\mathrm{sech}^2$",
        "sigmoid_sigmoid_deriv": "sigmoid / $\\sigma'$",
        "softplus_sigmoid": "softplus / $\\sigma$",
        "swish_swish_deriv": "swish / swish'",
        "gelu_gelu_deriv": "GELU / GELU'",
        "elu_elu_deriv": "ELU / ELU'",
        "mish_mish_deriv": "mish / mish'",
    }


def config_display_map() -> dict[str, str]:
    return {
        "factor1_offset0": "F1/O0",
        "factor2_offset0": "F2/O0",
        "factor2_offset256": "F2/O256",
        "factor2_offset512": "F2/O512",
    }


def scheme_color_map() -> dict[str, str]:
    return {
        "standard_pbs": BLUE,
        "sdr_pbs": RED,
        "many_lut": TEAL,
    }


def scheme_marker_map() -> dict[str, str]:
    return {
        "standard_pbs": "o",
        "sdr_pbs": "s",
        "many_lut": "^",
    }


def scheme_linestyle_map() -> dict[str, str]:
    return {
        "standard_pbs": "-",
        "sdr_pbs": "--",
        "many_lut": "-.",
    }


def threshold_columns(df: pd.DataFrame) -> dict[str, str]:
    return {
        "0.5%": "sigerr_0p5",
        "1.0%": "sigerr_1p0" if "sigerr_1p0" in df.columns else "significant_errors",
        "2.0%": "sigerr_2p0",
    }


def add_error_rate(df: pd.DataFrame, column: str, out_name: str) -> pd.DataFrame:
    result = df.copy()
    result[out_name] = result[column].astype(float) / result["points"].astype(float)
    return result


def save_table(df: pd.DataFrame, name: str) -> None:
    df.to_csv(TABLE_DIR / name, index=False)
    tex_name = Path(name).with_suffix(".tex")
    display = latex_table_view(df)
    write_latex_table(display, TABLE_DIR / tex_name)


def sigerr1_value(row: pd.Series) -> float:
    if "sigerr_1p0" in row.index:
        return float(row["sigerr_1p0"])
    return float(row["significant_errors"])


def format_axes(ax: plt.Axes) -> None:
    ax.grid(axis="y", color=GRID_COLOR, linestyle="-", linewidth=0.65, alpha=0.75)
    ax.set_axisbelow(True)
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.tick_params(axis="both", which="major", length=3.0, width=0.8, pad=3)
    ax.margins(x=0.03)


def style_figure(fig: plt.Figure) -> None:
    fig.patch.set_facecolor("white")


def style_line_kwargs(scheme: str) -> dict[str, object]:
    return {
        "color": scheme_color_map()[scheme],
        "marker": scheme_marker_map()[scheme],
        "linestyle": scheme_linestyle_map()[scheme],
        "markerfacecolor": scheme_color_map()[scheme],
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
            va="bottom",
            ha="left",
            color=PANEL_LABEL_COLOR,
        )


def bar_style_kwargs(color: str) -> dict[str, object]:
    return {
        "color": color,
        "edgecolor": EDGE_COLOR,
        "linewidth": 0.45,
        "alpha": 0.96,
    }


def add_top_legend(
    fig: plt.Figure,
    handles: list[object],
    labels: list[str],
    *,
    ncol: int,
    y: float,
) -> None:
    fig.legend(handles, labels, loc="upper center", ncol=ncol, bbox_to_anchor=(0.5, y))


def format_float_for_table(value: float) -> str:
    abs_value = abs(value)
    if abs_value >= 100:
        return f"{value:.2f}"
    if abs_value >= 1:
        return f"{value:.4f}"
    if abs_value == 0:
        return "0"
    return f"{value:.4g}"


def latex_table_view(df: pd.DataFrame) -> pd.DataFrame:
    display = df.copy()
    pair_labels = pair_display_map(multiline=False)
    configs = config_display_map()
    schemes = scheme_label_map()
    for col in display.columns:
        if col in {"pair", "bitwidth_pair", "ablation_pair"}:
            display[col] = display[col].map(lambda x: pair_labels.get(x, x))
        elif col in {"config", "ablation_config"}:
            display[col] = display[col].map(lambda x: configs.get(x, x))
        elif col == "scheme":
            display[col] = display[col].map(lambda x: schemes.get(x, x))
        elif pd.api.types.is_float_dtype(display[col]):
            display[col] = display[col].map(format_float_for_table)
    return display


def latex_escape(value: object) -> str:
    text = str(value)
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
        text = text.replace(src, dst)
    return text


def write_latex_table(df: pd.DataFrame, path: Path) -> None:
    cols = list(df.columns)
    align = "l" * len(cols)
    lines = [
        "\\small",
        "\\setlength{\\tabcolsep}{4pt}",
        "\\begin{tabular}{" + align + "}",
        "\\toprule",
        " & ".join(latex_escape(col) for col in cols) + " \\\\",
        "\\midrule",
    ]
    for row in df.itertuples(index=False, name=None):
        lines.append(" & ".join(latex_escape(item) for item in row) + " \\\\")
    lines.extend(["\\bottomrule", "\\end{tabular}", ""])
    path.write_text("\n".join(lines), encoding="utf-8")


def save_figure(fig: plt.Figure, stem: str) -> None:
    for suffix in (".pdf", ".png"):
        kwargs = {"bbox_inches": "tight", "pad_inches": 0.02}
        if suffix == ".png":
            kwargs["dpi"] = 600
        fig.savefig(FIG_DIR / f"{stem}{suffix}", **kwargs)


def load_multiseed() -> pd.DataFrame:
    rows = []
    for path in CANONICAL_ROOT.glob(
        "05_repeatability_multiseed_1000/allpairs_1000_seed_*/summary.csv"
    ):
        df = read_csv(path)
        seed = path.parent.name.split("_")[-1]
        df["seed"] = seed
        rows.append(df)
    if not rows:
        raise FileNotFoundError("missing multiseed summaries")
    return pd.concat(rows, ignore_index=True)


def plot_multiseed(multiseed: pd.DataFrame) -> None:
    df = add_error_rate(multiseed, "sigerr_1p0", "error_rate")
    summary = (
        df.groupby(["pair", "scheme"], as_index=False)
        .agg(
            mean_error_rate=("error_rate", "mean"),
            std_error_rate=("error_rate", "std"),
            mean_rmse1=("rmse_err1", "mean"),
            mean_rmse2=("rmse_err2", "mean"),
            mean_eval_us=("avg_eval_us", "mean"),
            std_eval_us=("avg_eval_us", "std"),
        )
        .fillna(0.0)
    )
    save_table(summary, "table1_multiseed_summary.csv")

    pairs = [pair for pair in pair_order() if pair in set(summary["pair"])]
    pair_labels = pair_display_map(multiline=True)
    x = np.arange(len(pairs))
    width = 0.22
    colors = scheme_color_map()
    labels = scheme_label_map()
    fig, ax = plt.subplots(figsize=(IEEE_DOUBLE_COL_IN, 3.05))
    style_figure(fig)
    for index, scheme in enumerate(scheme_order()):
        local = summary[summary["scheme"] == scheme].set_index("pair").reindex(pairs)
        ax.bar(
            x + (index - 1) * width,
            local["mean_error_rate"] * 100.0,
            width,
            label=labels[scheme],
            **bar_style_kwargs(colors[scheme]),
        )
    ax.set_xticks(x)
    ax.set_xticklabels([pair_labels[pair] for pair in pairs])
    ax.set_ylabel("Thresholded error rate (%)")
    ax.yaxis.set_major_formatter(PercentFormatter(decimals=1))
    ax.set_ylim(bottom=0.0)
    format_axes(ax)
    ax.legend(ncol=3, loc="upper center", bbox_to_anchor=(0.5, 1.13))
    fig.tight_layout(pad=0.85)
    save_figure(fig, "fig1_multiseed_error_rate")
    plt.close(fig)


def load_tail() -> pd.DataFrame:
    rows = []
    for path in CANONICAL_ROOT.glob("09_tail_stability_100k/*/summary.csv"):
        df = read_csv(path)
        rows.append(df)
    if not rows:
        raise FileNotFoundError("missing tail stability summaries")
    return pd.concat(rows, ignore_index=True)


def plot_tail(tail: pd.DataFrame) -> None:
    table = tail.copy()
    table["worst_p99"] = table[["p99_err1", "p99_err2"]].max(axis=1)
    table["worst_p999"] = table[["p999_err1", "p999_err2"]].max(axis=1)
    table["worst_max"] = table[["max_err1", "max_err2"]].max(axis=1)
    save_table(
        table[
            [
                "pair",
                "scheme",
                "points",
                "sigerr_1p0",
                "invalid_outputs",
                "worst_p99",
                "worst_p999",
                "worst_max",
                "avg_eval_us",
            ]
        ],
        "table2_tail_stability.csv",
    )

    pairs = [pair for pair in ["softplus_sigmoid", "sigmoid_sigmoid_deriv", "gelu_gelu_deriv"] if pair in set(table["pair"])]
    metrics = [("worst_p99", "p99"), ("worst_p999", "p99.9"), ("worst_max", "max")]
    colors = scheme_color_map()
    labels = scheme_label_map()
    pair_labels = pair_display_map(multiline=False)
    fig, axes = plt.subplots(1, len(pairs), figsize=(IEEE_DOUBLE_COL_IN, 3.18), sharey=True)
    style_figure(fig)
    global_max = 0.0
    for pair in pairs:
        local_pair = table[table["pair"] == pair]
        for scheme in ["standard_pbs", "sdr_pbs"]:
            local = local_pair[local_pair["scheme"] == scheme]
            global_max = max(global_max, max(float(local.iloc[0][column]) for column, _ in metrics))
    for pair_index, pair in enumerate(pairs):
        ax = np.atleast_1d(axes)[pair_index]
        local_pair = table[table["pair"] == pair]
        x = np.arange(len(metrics))
        width = 0.28
        for index, scheme in enumerate(["standard_pbs", "sdr_pbs"]):
            local = local_pair[local_pair["scheme"] == scheme]
            ax.bar(
                x + (index - 0.5) * width,
                [float(local.iloc[0][column]) for column, _ in metrics],
                width,
                label=labels[scheme],
                **bar_style_kwargs(colors[scheme]),
            )
        ax.set_xticks(x)
        ax.set_xticklabels([title for _, title in metrics])
        ax.set_title(pair_labels[pair], pad=6)
        ax.set_ylim(0.0, global_max * 1.18)
        format_axes(ax)
    np.atleast_1d(axes)[0].set_ylabel("Absolute error")
    add_panel_labels(axes)
    handles, legend_labels = np.atleast_1d(axes)[0].get_legend_handles_labels()
    add_top_legend(fig, handles, legend_labels, ncol=2, y=1.07)
    fig.tight_layout(pad=0.85, w_pad=1.05)
    save_figure(fig, "fig2_tail_stability")
    plt.close(fig)


def load_guard_ablation() -> pd.DataFrame:
    rows = []
    for path in CANONICAL_ROOT.glob("06_guardband_ablation/*/*/summary.csv"):
        df = read_csv(path)
        pair = path.parents[1].name
        config = path.parent.name
        df["ablation_pair"] = pair
        df["ablation_config"] = config
        rows.append(df)
    if not rows:
        raise FileNotFoundError("missing guard ablation summaries")
    return pd.concat(rows, ignore_index=True)


def plot_guard_ablation(guard_df: pd.DataFrame) -> None:
    df = guard_df.copy()
    df["worst_max"] = df[["max_err1", "max_err2"]].max(axis=1)
    df["error_rate"] = df["sigerr_1p0"] / df["points"]
    save_table(
        df[
            [
                "ablation_pair",
                "ablation_config",
                "scheme",
                "worst_max",
                "error_rate",
                "avg_eval_us",
            ]
        ],
        "table3_guard_ablation.csv",
    )

    pairs = [pair for pair in ["sigmoid_sigmoid_deriv", "softplus_sigmoid", "tanh_sech2"] if pair in set(df["ablation_pair"])]
    config_order = ["factor1_offset0", "factor2_offset0", "factor2_offset256", "factor2_offset512"]
    labels = scheme_label_map()
    pair_labels = pair_display_map(multiline=False)
    config_labels = config_display_map()
    for metric, ylabel, filename in [
        ("worst_max", "Worst-case absolute error", "fig3_guard_ablation_max.png"),
        ("error_rate", "Thresholded error rate", "fig4_guard_ablation_rate.png"),
    ]:
        fig, axes = plt.subplots(1, len(pairs), figsize=(IEEE_DOUBLE_COL_IN, 3.15), sharey=True)
        style_figure(fig)
        max_value = 0.0
        min_positive = None
        for pair in pairs:
            local_pair = df[df["ablation_pair"] == pair]
            for scheme in ["standard_pbs", "sdr_pbs"]:
                local = (
                    local_pair[local_pair["scheme"] == scheme]
                    .set_index("ablation_config")
                    .reindex(config_order)
                )
                series = local[metric].dropna().astype(float)
                if series.empty:
                    continue
                max_value = max(max_value, float(series.max()))
                positive = series[series > 0]
                if not positive.empty:
                    candidate = float(positive.min())
                    min_positive = candidate if min_positive is None else min(min_positive, candidate)
        for ax, pair in zip(np.atleast_1d(axes), pairs):
            local_pair = df[df["ablation_pair"] == pair]
            x = np.arange(len(config_order))
            for scheme in ["standard_pbs", "sdr_pbs"]:
                local = (
                    local_pair[local_pair["scheme"] == scheme]
                    .set_index("ablation_config")
                    .reindex(config_order)
                )
                ax.plot(
                    x,
                    local[metric].to_numpy(),
                    label=labels[scheme],
                    **style_line_kwargs(scheme),
                )
            ax.set_xticks(x)
            ax.set_xticklabels([config_labels[c] for c in config_order])
            ax.set_title(pair_labels[pair], pad=6)
            if metric == "worst_max":
                ax.set_yscale("log")
                if min_positive is not None and max_value > 0:
                    ax.set_ylim(min_positive * 0.85, max_value * 1.25)
            else:
                ax.yaxis.set_major_formatter(PercentFormatter(xmax=1.0, decimals=1))
                ax.set_ylim(0.0, max_value * 1.16)
            format_axes(ax)
        np.atleast_1d(axes)[0].set_ylabel(ylabel)
        add_panel_labels(axes)
        handles, legend_labels = np.atleast_1d(axes)[0].get_legend_handles_labels()
        add_top_legend(fig, handles, legend_labels, ncol=2, y=1.07)
        fig.tight_layout(pad=0.85, w_pad=1.0)
        save_figure(fig, Path(filename).stem)
        plt.close(fig)


def load_prepost() -> pd.DataFrame:
    current = read_csv(CANONICAL_ROOT / "04_main_guarded_all_pairs_10000" / "summary.csv")
    old = pd.concat(
        [
            read_csv(
                ARCHIVE_ROOT / "02_pre_fix_reference_10000" / "tanh_10000" / "summary.csv"
            ),
            read_csv(
                ARCHIVE_ROOT
                / "02_pre_fix_reference_10000"
                / "sigmoid_10000"
                / "summary.csv"
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
                "old_sig": old_row["significant_errors"],
                "new_sig": sigerr1_value(new_row),
                "old_worst_max": max(old_row["max_err1"], old_row["max_err2"]),
                "new_worst_max": max(new_row["max_err1"], new_row["max_err2"]),
            }
        )
    return pd.DataFrame(rows)


def plot_prepost(prepost: pd.DataFrame) -> None:
    save_table(prepost, "table4_prepost_compare.csv")
    pair_labels = pair_display_map(multiline=True)
    scheme_labels = scheme_label_map()
    pairs = ["tanh_sech2", "sigmoid_sigmoid_deriv", "softplus_sigmoid"]
    width = 0.30
    fig, axes = plt.subplots(1, 2, figsize=(IEEE_DOUBLE_COL_IN, 3.10), sharey=True)
    style_figure(fig)
    for idx, scheme in enumerate(["standard_pbs", "sdr_pbs"]):
        ax = np.atleast_1d(axes)[idx]
        local = prepost[prepost["scheme"] == scheme].set_index("pair").reindex(pairs)
        x = np.arange(len(pairs))
        ax.bar(
            x - width / 2,
            local["old_worst_max"],
            width,
            label="Pre-fix",
            **bar_style_kwargs(PREPOST_COLORS["pre"]),
        )
        ax.bar(
            x + width / 2,
            local["new_worst_max"],
            width,
            label="Guarded",
            **bar_style_kwargs(PREPOST_COLORS["post"]),
        )
        ax.set_xticks(x)
        ax.set_xticklabels([pair_labels[p] for p in pairs])
        ax.set_title(scheme_labels[scheme], pad=6)
        ax.set_yscale("log")
        format_axes(ax)
    np.atleast_1d(axes)[0].set_ylabel("Worst-case absolute error")
    add_panel_labels(axes)
    handles, legend_labels = np.atleast_1d(axes)[0].get_legend_handles_labels()
    add_top_legend(fig, handles, legend_labels, ncol=2, y=1.07)
    fig.tight_layout(pad=0.85, w_pad=1.0)
    save_figure(fig, "fig5_prepost_compare")
    plt.close(fig)


def load_timing(multiseed: pd.DataFrame) -> pd.DataFrame:
    summary = (
        multiseed.groupby("scheme", as_index=False)
        .agg(
            avg_input_us=("avg_input_us", "mean"),
            avg_core_us=("avg_core_us", "mean"),
            avg_decode_us=("avg_decode_us", "mean"),
            avg_eval_us=("avg_eval_us", "mean"),
        )
    )
    return (
        summary.set_index("scheme")
        .reindex(scheme_order())
        .dropna(how="all")
        .reset_index()
    )


def plot_timing(timing_df: pd.DataFrame) -> None:
    save_table(timing_df, "table5_timing_breakdown.csv")
    fig, axes = plt.subplots(1, 2, figsize=(IEEE_DOUBLE_COL_IN, 3.05))
    style_figure(fig)
    x = np.arange(len(timing_df))
    labels = [scheme_label_map()[scheme] for scheme in timing_df["scheme"]]
    total_ms = timing_df["avg_eval_us"] / 1000.0
    scheme_colors = [scheme_color_map()[scheme] for scheme in timing_df["scheme"]]

    ax0 = axes[0]
    ax0.bar(
        x,
        total_ms,
        color=scheme_colors,
        edgecolor=EDGE_COLOR,
        linewidth=0.45,
        alpha=0.96,
    )
    ax0.set_xticks(x)
    ax0.set_xticklabels(labels, rotation=0, ha="center")
    ax0.set_title("Total latency", pad=6)
    ax0.set_ylabel("Average eval. time (ms)")
    format_axes(ax0)

    ax1 = axes[1]
    width = 0.22
    ax1.bar(
        x - width,
        timing_df["avg_input_us"],
        width,
        label="Input",
        **bar_style_kwargs(AMBER),
    )
    ax1.bar(
        x,
        timing_df["avg_core_us"],
        width,
        label="Core",
        **bar_style_kwargs(BLUE),
    )
    ax1.bar(
        x + width,
        timing_df["avg_decode_us"],
        width,
        label="Decode",
        **bar_style_kwargs(RED),
    )
    ax1.set_xticks(x)
    ax1.set_xticklabels(labels, rotation=0, ha="center")
    ax1.set_title("Stage latency (log scale)", pad=6)
    ax1.set_ylabel("Stage time (us)")
    ax1.set_yscale("log")
    format_axes(ax1)
    add_panel_labels(axes)
    handles, legend_labels = ax1.get_legend_handles_labels()
    add_top_legend(fig, handles, legend_labels, ncol=3, y=1.05)
    fig.tight_layout(pad=0.85, w_pad=1.1)
    save_figure(fig, "fig6_timing_breakdown")
    plt.close(fig)


def load_codebook() -> pd.DataFrame:
    path = (
        CANONICAL_ROOT
        / "07_codebook_recovery_validation"
        / "allpairs"
        / "codebook_summary.csv"
    )
    if not path.exists():
        raise FileNotFoundError("missing codebook summary")
    df = read_csv(path)
    df["exact_recovery_rate"] = df["exact_recovery"] / df["total_inputs"]
    return df


def plot_codebook(codebook_df: pd.DataFrame) -> None:
    save_table(codebook_df, "table6_codebook_correctness.csv")
    pairs = [pair for pair in pair_order() if pair in set(codebook_df["pair"])]
    pair_labels = pair_display_map(multiline=True)
    x = np.arange(len(pairs))
    width = 0.22
    colors = scheme_color_map()
    labels = scheme_label_map()
    fig, ax = plt.subplots(figsize=(IEEE_DOUBLE_COL_IN, 3.05))
    style_figure(fig)
    for index, scheme in enumerate(scheme_order()):
        local = codebook_df[codebook_df["scheme"] == scheme].set_index("pair").reindex(pairs)
        if local["exact_recovery_rate"].isnull().all():
            continue
        ax.bar(
            x + (index - 1) * width,
            local["exact_recovery_rate"] * 100.0,
            width,
            label=labels[scheme],
            **bar_style_kwargs(colors[scheme]),
        )
    ax.set_xticks(x)
    ax.set_xticklabels([pair_labels[pair] for pair in pairs])
    ax.set_ylabel("Exact recovery rate (%)")
    ax.yaxis.set_major_formatter(PercentFormatter(decimals=0))
    ax.set_ylim(0.0, 60.0)
    format_axes(ax)
    ax.legend(ncol=3, loc="upper center", bbox_to_anchor=(0.5, 1.13))
    fig.tight_layout(pad=0.85)
    save_figure(fig, "fig7_codebook_exact_recovery")
    plt.close(fig)


def load_bitwidth() -> pd.DataFrame:
    rows = []
    for path in CANONICAL_ROOT.glob("08_bitwidth_sensitivity/*/bits_*/summary.csv"):
        df = read_csv(path)
        pair = path.parents[1].name
        bits = int(path.parent.name.split("_")[-1])
        df["bitwidth_pair"] = pair
        df["bitwidth"] = bits
        rows.append(df)
    if not rows:
        raise FileNotFoundError("missing bitwidth summaries")
    return pd.concat(rows, ignore_index=True)


def load_end_to_end() -> pd.DataFrame:
    path = (
        CANONICAL_ROOT
        / "12_end_to_end_micro_pipeline"
        / "representative_pairs_1000"
        / "end_to_end_summary.csv"
    )
    if not path.exists():
        raise FileNotFoundError("missing end-to-end summary")
    return read_csv(path)


def plot_bitwidth(bitwidth_df: pd.DataFrame) -> None:
    df = bitwidth_df.copy()
    df["worst_rmse"] = df[["rmse_err1", "rmse_err2"]].max(axis=1)
    df["error_rate"] = df["sigerr_1p0"] / df["points"]
    save_table(
        df[
            [
                "bitwidth_pair",
                "bitwidth",
                "scheme",
                "worst_rmse",
                "error_rate",
                "avg_eval_us",
            ]
        ],
        "table7_bitwidth_sensitivity.csv",
    )

    pairs = [pair for pair in ["gelu_gelu_deriv", "softplus_sigmoid", "tanh_sech2"] if pair in set(df["bitwidth_pair"])]
    labels = scheme_label_map()
    pair_labels = pair_display_map(multiline=False)
    fig, axes = plt.subplots(2, len(pairs), figsize=(IEEE_DOUBLE_COL_IN, 5.25), sharex=True)
    style_figure(fig)
    bitwidth_ticks = sorted(df["bitwidth"].astype(int).unique())
    max_rmse = 0.0
    max_error = 0.0
    for pair in pairs:
        local_pair = df[df["bitwidth_pair"] == pair]
        max_rmse = max(max_rmse, float(local_pair["worst_rmse"].max()))
        max_error = max(max_error, float((local_pair["error_rate"] * 100.0).max()))
    for col, pair in enumerate(pairs):
        local_pair = df[df["bitwidth_pair"] == pair]
        for scheme in scheme_order():
            local = local_pair[local_pair["scheme"] == scheme].sort_values("bitwidth")
            if local.empty:
                continue
            axes[0, col].plot(
                local["bitwidth"],
                local["worst_rmse"],
                label=labels[scheme],
                **style_line_kwargs(scheme),
            )
            axes[1, col].plot(
                local["bitwidth"],
                local["error_rate"] * 100.0,
                label=labels[scheme],
                **style_line_kwargs(scheme),
            )
        axes[0, col].set_title(pair_labels[pair], pad=5)
        axes[0, col].set_ylim(0.0, max_rmse * 1.10)
        axes[1, col].set_ylim(0.0, max_error * 1.12)
        axes[0, col].set_xticks(bitwidth_ticks)
        axes[1, col].set_xticks(bitwidth_ticks)
        if col == 0:
            axes[0, col].set_ylabel("Worst-output RMSE")
            axes[1, col].set_ylabel("Thresholded error rate (%)")
        else:
            axes[0, col].set_ylabel("")
            axes[1, col].set_ylabel("")
        axes[1, col].set_xlabel("Bit width")
        axes[1, col].yaxis.set_major_formatter(PercentFormatter(decimals=1))
        format_axes(axes[0, col])
        format_axes(axes[1, col])
    handles, legend_labels = axes[0, 0].get_legend_handles_labels()
    add_panel_labels(axes)
    add_top_legend(fig, handles, legend_labels, ncol=3, y=1.03)
    fig.tight_layout(pad=0.85, w_pad=1.0, h_pad=1.2)
    save_figure(fig, "fig8_bitwidth_sensitivity")
    plt.close(fig)


def plot_end_to_end(e2e_df: pd.DataFrame) -> None:
    export = e2e_df[
        [
            "pair",
            "scheme",
            "bits",
            "score_significant_errors",
            "score_rmse",
            "update_significant_errors",
            "update_rmse",
            "avg_downstream_us",
            "avg_e2e_us",
        ]
    ].copy()
    save_table(export, "table9_end_to_end_micro_pipeline.csv")

    pairs = [
        pair
        for pair in ["sigmoid_sigmoid_deriv", "softplus_sigmoid", "gelu_gelu_deriv"]
        if pair in set(e2e_df["pair"])
    ]
    pair_labels = pair_display_map(multiline=False)
    labels = scheme_label_map()
    colors = scheme_color_map()
    x = np.arange(len(pairs))
    width = 0.22

    fig, axes = plt.subplots(1, 2, figsize=(IEEE_DOUBLE_COL_IN, 2.85))
    style_figure(fig)

    ax0, ax1 = axes
    for index, scheme in enumerate(scheme_order()):
        local = e2e_df[e2e_df["scheme"] == scheme].set_index("pair").reindex(pairs)
        if local.empty:
            continue
        ax0.bar(
            x + (index - 1) * width,
            local["avg_e2e_us"] / 1000.0,
            width,
            label=labels[scheme],
            **bar_style_kwargs(colors[scheme]),
        )
        ax1.bar(
            x + (index - 1) * width,
            local["score_rmse"],
            width,
            label=labels[scheme],
            **bar_style_kwargs(colors[scheme]),
        )

    ax0.set_xticks(x)
    ax0.set_xticklabels([pair_labels[pair] for pair in pairs], rotation=0, ha="center")
    ax0.set_ylabel("End-to-end latency (ms)")
    ax0.set_title("Application-level latency", pad=6)
    format_axes(ax0)

    ax1.set_xticks(x)
    ax1.set_xticklabels([pair_labels[pair] for pair in pairs], rotation=0, ha="center")
    ax1.set_ylabel("Score RMSE")
    ax1.set_title("Downstream score error", pad=6)
    format_axes(ax1)

    handles, legend_labels = ax0.get_legend_handles_labels()
    add_top_legend(fig, handles, legend_labels, ncol=3, y=1.05)
    fig.tight_layout(pad=0.85, w_pad=1.0)
    save_figure(fig, "fig10_end_to_end_micro_pipeline")
    plt.close(fig)


def plot_threshold_sensitivity(multiseed: pd.DataFrame) -> None:
    cols = threshold_columns(multiseed)
    rows = []
    for scheme, group in multiseed.groupby("scheme"):
        for label, column in cols.items():
            if column not in group.columns:
                continue
            rows.append(
                {
                    "scheme": scheme,
                    "threshold": label,
                    "error_rate": group[column].astype(float).sum() / group["points"].astype(float).sum(),
                }
            )
    threshold_df = pd.DataFrame(rows)
    save_table(threshold_df, "table8_threshold_sensitivity.csv")

    fig, ax = plt.subplots(figsize=(IEEE_SINGLE_COL_IN, 2.65))
    style_figure(fig)
    labels = scheme_label_map()
    threshold_order = ["0.5%", "1.0%", "2.0%"]
    x = np.arange(len(threshold_order))
    for scheme in scheme_order():
        local = threshold_df[threshold_df["scheme"] == scheme].set_index("threshold").reindex(threshold_order)
        if local.empty:
            continue
        ax.plot(
            x,
            local["error_rate"] * 100.0,
            label=labels[scheme],
            **style_line_kwargs(scheme),
        )
    ax.set_xticks(x)
    ax.set_xticklabels(threshold_order)
    ax.set_ylabel("Aggregated error rate (%)")
    ax.yaxis.set_major_formatter(PercentFormatter(decimals=1))
    ax.set_ylim(bottom=0.0)
    format_axes(ax)
    ax.legend(
        loc="upper center",
        bbox_to_anchor=(0.5, 1.09),
        ncol=3,
        columnspacing=0.8,
        handletextpad=0.5,
    )
    fig.tight_layout(pad=0.85)
    save_figure(fig, "fig9_threshold_sensitivity")
    plt.close(fig)


def main() -> None:
    args = parse_args()
    configure_paths(args)
    ensure_dirs()

    multiseed = load_multiseed()
    plot_multiseed(multiseed)
    plot_timing(load_timing(multiseed))
    plot_threshold_sensitivity(multiseed)

    guard_df = load_guard_ablation()
    plot_guard_ablation(guard_df)

    codebook_df = load_codebook()
    plot_codebook(codebook_df)

    bitwidth_df = load_bitwidth()
    plot_bitwidth(bitwidth_df)

    e2e_df = load_end_to_end()
    plot_end_to_end(e2e_df)

    prepost_df = load_prepost()
    plot_prepost(prepost_df)

    tail_df = load_tail()
    plot_tail(tail_df)

    print(f"Canonical results read from {CANONICAL_ROOT}")
    print(f"Historical archive read from {ARCHIVE_ROOT}")
    print(f"Figures written to {FIG_DIR}")
    print(f"Tables written to {TABLE_DIR}")


if __name__ == "__main__":
    main()
