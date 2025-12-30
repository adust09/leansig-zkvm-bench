#!/usr/bin/env python3
"""Generate benchmark comparison charts for zkVM proving times."""

import matplotlib.pyplot as plt
import numpy as np
import os

# Benchmark data (seconds)
data = {
    "SP1": {
        "proving_time": 71.4,  # CPU, M3 Max
        "cycles": 135_801,
        "execution_time": 0.018,  # 18ms
        "status": "completed",
    },
    "Zisk": {
        "proving_time": 1580.3,  # ~26.3 minutes
        "cycles": 158_022,
        "execution_time": 0.0034,  # 3.4ms
        "status": "completed",
    },
    "RISC Zero": {
        "proving_time": 600,  # >10 min (timeout, using 10 min as lower bound)
        "cycles": 11_010_048,
        "execution_time": 0.233,  # 233ms
        "status": "timeout",
    },
}

# Create output directory
os.makedirs("charts", exist_ok=True)


def create_proving_time_chart():
    """Create bar chart comparing proving times."""
    fig, ax = plt.subplots(figsize=(10, 6))

    zkvm_names = list(data.keys())
    proving_times = [data[name]["proving_time"] for name in zkvm_names]
    statuses = [data[name]["status"] for name in zkvm_names]

    # Colors based on status
    colors = []
    for status in statuses:
        if status == "completed":
            colors.append("#4CAF50")  # Green for completed
        elif status == "timeout":
            colors.append("#FF9800")  # Orange for timeout
        else:
            colors.append("#9E9E9E")  # Gray for WIP

    bars = ax.bar(zkvm_names, proving_times, color=colors, edgecolor="black", linewidth=1.2)

    # Add value labels on bars
    for bar, time, status in zip(bars, proving_times, statuses):
        height = bar.get_height()
        label = f"{time:.1f}s"
        if time >= 60:
            label = f"{time/60:.1f}min"
        if status == "timeout":
            label = f">{label}"
        ax.text(
            bar.get_x() + bar.get_width() / 2.0,
            height,
            label,
            ha="center",
            va="bottom",
            fontsize=12,
            fontweight="bold",
        )

    ax.set_ylabel("Proving Time (seconds)", fontsize=12)
    ax.set_xlabel("zkVM", fontsize=12)
    ax.set_title("XMSS Signature Verification - Proving Time Comparison", fontsize=14, fontweight="bold")

    # Add legend
    from matplotlib.patches import Patch

    legend_elements = [
        Patch(facecolor="#4CAF50", edgecolor="black", label="Completed"),
        Patch(facecolor="#FF9800", edgecolor="black", label="Timeout (>10 min)"),
    ]
    ax.legend(handles=legend_elements, loc="upper right")

    # Grid
    ax.yaxis.grid(True, linestyle="--", alpha=0.7)
    ax.set_axisbelow(True)

    plt.tight_layout()
    plt.savefig("charts/proving_time_comparison.png", dpi=150, bbox_inches="tight")
    plt.savefig("charts/proving_time_comparison.svg", bbox_inches="tight")
    print("Saved: charts/proving_time_comparison.png")
    print("Saved: charts/proving_time_comparison.svg")
    plt.close()


def create_cycles_chart():
    """Create bar chart comparing VM cycles."""
    fig, ax = plt.subplots(figsize=(10, 6))

    zkvm_names = list(data.keys())
    cycles = [data[name]["cycles"] for name in zkvm_names]

    colors = ["#2196F3", "#2196F3", "#F44336"]  # Blue for low cycles, red for high

    bars = ax.bar(zkvm_names, cycles, color=colors, edgecolor="black", linewidth=1.2)

    # Add value labels on bars
    for bar, cycle in zip(bars, cycles):
        height = bar.get_height()
        if cycle >= 1_000_000:
            label = f"{cycle/1_000_000:.1f}M"
        else:
            label = f"{cycle/1_000:.0f}K"
        ax.text(
            bar.get_x() + bar.get_width() / 2.0,
            height,
            label,
            ha="center",
            va="bottom",
            fontsize=12,
            fontweight="bold",
        )

    ax.set_ylabel("VM Cycles", fontsize=12)
    ax.set_xlabel("zkVM", fontsize=12)
    ax.set_title("XMSS Signature Verification - VM Cycles Comparison", fontsize=14, fontweight="bold")

    # Use log scale for better visualization
    ax.set_yscale("log")
    ax.yaxis.grid(True, linestyle="--", alpha=0.7)
    ax.set_axisbelow(True)

    plt.tight_layout()
    plt.savefig("charts/cycles_comparison.png", dpi=150, bbox_inches="tight")
    plt.savefig("charts/cycles_comparison.svg", bbox_inches="tight")
    print("Saved: charts/cycles_comparison.png")
    print("Saved: charts/cycles_comparison.svg")
    plt.close()


def create_combined_chart():
    """Create a combined chart with proving time and cycles."""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

    zkvm_names = list(data.keys())
    proving_times = [data[name]["proving_time"] for name in zkvm_names]
    cycles = [data[name]["cycles"] for name in zkvm_names]
    statuses = [data[name]["status"] for name in zkvm_names]

    # Proving time chart
    colors = ["#4CAF50" if s == "completed" else "#FF9800" for s in statuses]
    bars1 = ax1.bar(zkvm_names, proving_times, color=colors, edgecolor="black", linewidth=1.2)

    for bar, time, status in zip(bars1, proving_times, statuses):
        height = bar.get_height()
        label = f"{time:.1f}s" if time < 60 else f"{time/60:.1f}min"
        if status == "timeout":
            label = f">{label}"
        ax1.text(
            bar.get_x() + bar.get_width() / 2.0,
            height,
            label,
            ha="center",
            va="bottom",
            fontsize=11,
            fontweight="bold",
        )

    ax1.set_ylabel("Proving Time (seconds)", fontsize=12)
    ax1.set_xlabel("zkVM", fontsize=12)
    ax1.set_title("Proving Time", fontsize=13, fontweight="bold")
    ax1.yaxis.grid(True, linestyle="--", alpha=0.7)
    ax1.set_axisbelow(True)

    # Cycles chart
    cycle_colors = ["#2196F3" if c < 1_000_000 else "#F44336" for c in cycles]
    bars2 = ax2.bar(zkvm_names, cycles, color=cycle_colors, edgecolor="black", linewidth=1.2)

    for bar, cycle in zip(bars2, cycles):
        height = bar.get_height()
        label = f"{cycle/1_000_000:.1f}M" if cycle >= 1_000_000 else f"{cycle/1_000:.0f}K"
        ax2.text(
            bar.get_x() + bar.get_width() / 2.0,
            height,
            label,
            ha="center",
            va="bottom",
            fontsize=11,
            fontweight="bold",
        )

    ax2.set_ylabel("VM Cycles", fontsize=12)
    ax2.set_xlabel("zkVM", fontsize=12)
    ax2.set_title("VM Cycles", fontsize=13, fontweight="bold")
    ax2.set_yscale("log")
    ax2.yaxis.grid(True, linestyle="--", alpha=0.7)
    ax2.set_axisbelow(True)

    fig.suptitle(
        "leanSig XMSS Verification - zkVM Benchmark Comparison",
        fontsize=15,
        fontweight="bold",
        y=1.02,
    )

    plt.tight_layout()
    plt.savefig("charts/benchmark_comparison.png", dpi=150, bbox_inches="tight")
    plt.savefig("charts/benchmark_comparison.svg", bbox_inches="tight")
    print("Saved: charts/benchmark_comparison.png")
    print("Saved: charts/benchmark_comparison.svg")
    plt.close()


def create_efficiency_chart():
    """Create scatter plot showing cycles vs proving time."""
    fig, ax = plt.subplots(figsize=(10, 7))

    for name, d in data.items():
        color = "#4CAF50" if d["status"] == "completed" else "#FF9800"
        marker = "o" if d["status"] == "completed" else "^"
        ax.scatter(
            d["cycles"],
            d["proving_time"],
            s=200,
            c=color,
            marker=marker,
            edgecolors="black",
            linewidth=1.5,
            label=name,
            zorder=5,
        )
        # Add label next to point
        offset_x = d["cycles"] * 0.1
        offset_y = d["proving_time"] * 0.05
        time_label = f"{d['proving_time']:.1f}s" if d["proving_time"] < 60 else f"{d['proving_time']/60:.1f}min"
        if d["status"] == "timeout":
            time_label = f">{time_label}"
        ax.annotate(
            f"{name}\n{time_label}",
            (d["cycles"], d["proving_time"]),
            xytext=(offset_x, offset_y),
            textcoords="offset points",
            fontsize=10,
            fontweight="bold",
            ha="left",
        )

    ax.set_xlabel("VM Cycles", fontsize=12)
    ax.set_ylabel("Proving Time (seconds)", fontsize=12)
    ax.set_title(
        "zkVM Efficiency: Cycles vs Proving Time",
        fontsize=14,
        fontweight="bold",
    )
    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.grid(True, linestyle="--", alpha=0.7)

    plt.tight_layout()
    plt.savefig("charts/efficiency_scatter.png", dpi=150, bbox_inches="tight")
    plt.savefig("charts/efficiency_scatter.svg", bbox_inches="tight")
    print("Saved: charts/efficiency_scatter.png")
    print("Saved: charts/efficiency_scatter.svg")
    plt.close()


if __name__ == "__main__":
    print("Generating zkVM benchmark charts...")
    print()

    create_proving_time_chart()
    create_cycles_chart()
    create_combined_chart()
    create_efficiency_chart()

    print()
    print("All charts generated successfully!")
    print("Check the 'charts/' directory for output files.")
