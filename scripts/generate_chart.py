#!/usr/bin/env python3
"""Generate benchmark comparison charts for zkVM proving times."""

import matplotlib
matplotlib.use('Agg')  # Non-interactive backend for headless generation
import matplotlib.pyplot as plt
import numpy as np
import os

# Benchmark data (seconds)
# Updated: 2024 - TargetSum W=1 (155 chains, Poseidon2)
# Note: Proving times are from previous measurements, need re-measurement with new encoding
data = {
    "SP1": {
        "proving_time": 71.4,  # Previous measurement - needs re-measurement
        "cycles": 60_424_086,  # ~60M cycles (TargetSum W=1, 155 chains)
        "execution_time": 2.65,  # 2.65s
        "status": "completed",
    },
    "Zisk": {
        "proving_time": 1580.3,  # Previous measurement - build currently broken
        "cycles": 158_022,  # Previous measurement - build currently broken
        "execution_time": 0.0034,  # 3.4ms
        "status": "wip",
    },
    "OpenVM": {
        "proving_time": 294.5,  # ~4.9 minutes, macOS Apple Silicon
        "cycles": None,  # N/A - OpenVM uses different architecture
        "execution_time": 0.150,  # 150.4ms (input generation, not execution)
        "status": "completed",
    },
    "RISC Zero": {
        "proving_time": 600,  # >10 min (timeout estimate)
        "cycles": 5_728_806,  # ~5.7M user cycles (TargetSum W=1)
        "execution_time": 0.275,  # 275ms (dev mode)
        "status": "timeout",
    },
}

# Create output directory
os.makedirs("charts", exist_ok=True)


def create_proving_time_chart():
    """Create bar chart comparing proving times."""
    fig, ax = plt.subplots(figsize=(10, 6))

    # Filter out entries without proving time data
    zkvm_names = [name for name in data.keys() if data[name]["proving_time"] is not None]
    proving_times = [data[name]["proving_time"] for name in zkvm_names]
    statuses = [data[name]["status"] for name in zkvm_names]

    # Colors based on status
    colors = []
    for status in statuses:
        if status == "completed":
            colors.append("#4CAF50")  # Green for completed
        elif status == "timeout":
            colors.append("#FF9800")  # Orange for timeout
        elif status == "wip":
            colors.append("#9E9E9E")  # Gray for WIP
        else:
            colors.append("#2196F3")  # Blue for others

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
        Patch(facecolor="#9E9E9E", edgecolor="black", label="WIP (needs re-measurement)"),
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

    # Filter out entries without cycle data
    zkvm_names = [name for name in data.keys() if data[name]["cycles"] is not None]
    cycles = [data[name]["cycles"] for name in zkvm_names]

    # Colors based on cycle count (blue for low <1M, red for high >=1M)
    colors = ["#2196F3" if c < 1_000_000 else "#F44336" for c in cycles]

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

    # zkVMs with proving time data only
    zkvm_names = [name for name in data.keys() if data[name]["proving_time"] is not None]
    proving_times = [data[name]["proving_time"] for name in zkvm_names]
    statuses = [data[name]["status"] for name in zkvm_names]

    # zkVMs with cycle data only
    zkvm_names_with_cycles = [name for name in data.keys() if data[name]["cycles"] is not None]
    cycles = [data[name]["cycles"] for name in zkvm_names_with_cycles]

    # Proving time chart - colors based on status
    colors = []
    for s in statuses:
        if s == "completed":
            colors.append("#4CAF50")
        elif s == "timeout":
            colors.append("#FF9800")
        elif s == "wip":
            colors.append("#9E9E9E")
        else:
            colors.append("#2196F3")
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

    # Cycles chart (only zkVMs with cycle data)
    cycle_colors = ["#2196F3" if c < 1_000_000 else "#F44336" for c in cycles]
    bars2 = ax2.bar(zkvm_names_with_cycles, cycles, color=cycle_colors, edgecolor="black", linewidth=1.2)

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
    ax2.set_title("VM Cycles (RISC-V based)", fontsize=13, fontweight="bold")
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

    # Only plot zkVMs with both cycle and proving time data
    for name, d in data.items():
        if d["cycles"] is None or d["proving_time"] is None:
            continue  # Skip zkVMs without complete data
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
        # Add label next to point (use fixed offset in points, not data-based)
        time_label = f"{d['proving_time']:.1f}s" if d["proving_time"] < 60 else f"{d['proving_time']/60:.1f}min"
        if d["status"] == "timeout":
            time_label = f">{time_label}"
        ax.annotate(
            f"{name}\n{time_label}",
            (d["cycles"], d["proving_time"]),
            xytext=(10, 5),
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
