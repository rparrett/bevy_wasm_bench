import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import statsmodels.formula.api as smf
import re
from matplotlib.backends.backend_pdf import PdfPages

DATA_CSV = "../results/42674c1_win11_i713700KF_4080super.csv"


def build_formula(response_col, cat_vars, baselines_dict):
    """
    Construct a formula like:
    "frame_time ~ C(opt_level, Treatment('Three')) + C(wasm_opt) + ... + C(strip, Treatment('None'))"
    """
    terms = []
    for col in cat_vars:
        if col in baselines_dict:
            baseline = baselines_dict[col]
            terms.append(f"C({col}, Treatment('{baseline}'))")
        else:
            terms.append(f"C({col})")
    return f"{response_col} ~ " + " + ".join(terms)


# Regex for cleaning up coefficient names (e.g. "C(codegen_units, Treatment('Default'))[T.One]" -> "codegen_units One")
pattern = re.compile(r"^C\(([^,]+)(?:,\s*Treatment\('[^']+'\))?\)\[T\.(.*)\]$")


def clean_term_name(term):
    match = pattern.match(term)
    if match:
        var_name = match.group(1)  # e.g. codegen_units
        level_name = match.group(2)  # e.g. One
        return f"{var_name} {level_name}"
    return term


# Map p-values to R-style significance codes
def get_significance_code(p):
    if p < 0.001:
        return "***"
    elif p < 0.01:
        return "**"
    elif p < 0.05:
        return "*"
    elif p < 0.1:
        return "."
    else:
        return " "


# -----------------------------------------------------------------
# Create a scatter plot figure for size_gzipped vs frame_time
# -----------------------------------------------------------------
def create_scatter_plot(cat_col, data):
    fig, ax = plt.subplots(figsize=(6, 4))
    sns.set_theme(style="whitegrid")
    scatter_plot = sns.scatterplot(
        data=data, x="size_gzipped", y="frame_time", hue=cat_col, palette="deep", ax=ax
    )
    scatter_plot.set(
        title=f"size_gzipped vs. frame_time colored by {cat_col}",
        xlabel="Gzipped Size (bytes)",
        ylabel="Frame Time (ms)",
    )
    ax.legend(title=cat_col)
    fig.tight_layout()
    return fig


# -----------------------------------------------------------------
# Create a coefficient plot figure for a given response
# -----------------------------------------------------------------
def create_lm_coef_plot(response_col, data):
    formula_str = build_formula(response_col, cat_vars, baselines)
    print("Formula:", formula_str)

    model = smf.ols(formula_str, data=data).fit()
    print(f"\nModel summary for {response_col}:")
    print(model.summary())

    coefs = pd.DataFrame(
        {
            "term": model.params.index,
            "estimate": model.params.values,
            "p_value": model.pvalues.values,
        }
    )
    conf_int = model.conf_int()
    coefs["conf_low"] = conf_int[0].values
    coefs["conf_high"] = conf_int[1].values

    # Drop the intercept
    coefs = coefs[coefs["term"] != "Intercept"].copy()

    coefs["term"] = coefs["term"].apply(clean_term_name)
    coefs.sort_values("estimate", ascending=False, inplace=True)

    fig, ax = plt.subplots(figsize=(6, 4))
    sns.set_theme(style="whitegrid")

    y_positions = range(len(coefs))
    ax.errorbar(
        x=coefs["estimate"],
        y=y_positions,
        xerr=[
            coefs["estimate"] - coefs["conf_low"],
            coefs["conf_high"] - coefs["estimate"],
        ],
        fmt="o",
        ecolor="black",
        capsize=3,
        color="blue",
    )
    ax.axvline(x=0, color="red", linestyle="--")

    coefs["signif_code"] = coefs["p_value"].apply(get_significance_code)
    for i, row in enumerate(coefs.itertuples(index=False)):
        ax.text(x=row.estimate, y=i + 0.1, s=row.signif_code, color="red", ha="center")

    ax.set_yticks(list(y_positions))
    ax.set_yticklabels(coefs["term"])
    ax.set_title(f"Coefficients for {response_col}")
    ax.set_xlabel("Estimate")
    ax.set_ylabel("Predictor")
    fig.tight_layout()
    return fig


#
# Load and process data
#

data = pd.read_csv(DATA_CSV, sep=",", header=0, keep_default_na=False, na_values=[])

cat_vars = ["opt_level", "wasm_opt", "lto", "codegen_units", "strip", "panic"]

for cat_col in cat_vars:
    data[cat_col] = data[cat_col].astype("category")
    data["total_build_time"] = data["build_time"] + data["wasm_opt_time"]

print(data["strip"].unique())

baselines = {
    "opt_level": "Three",
    "wasm_opt": "None",
    "strip": "None",
    "lto": "Off",
    "codegen_units": "Default",
    "panic": "Unwind",
}

#
# Generate all plots into a single multi-page PDF
#

pdf_filename = "plots.pdf"
with PdfPages(pdf_filename) as pdf:
    for cat_col in cat_vars:
        fig = create_scatter_plot(cat_col, data)
        pdf.savefig(fig)
        plt.close(fig)

    for response_col in [
        "frame_time",
        "size_gzipped",
        "build_time",
        "wasm_opt_time",
        "total_build_time",
    ]:
        fig = create_lm_coef_plot(response_col, data)
        pdf.savefig(fig)
        plt.close(fig)

print(f"Saved to {pdf_filename}.")
