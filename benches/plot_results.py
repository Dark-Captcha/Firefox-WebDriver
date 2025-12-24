#!/usr/bin/env python3
"""
Generate benchmark plots from CSV results.

Usage: python3 benches/plot_results.py

Input:  benches/results/benchmark_results.csv
Output: docs/images/benchmark_spawn.png
        docs/images/benchmark_ops.png
"""

import csv
import os
from pathlib import Path

# Try to import matplotlib, provide helpful error if missing
try:
    import matplotlib.pyplot as plt
    import matplotlib.ticker as ticker
except ImportError:
    print("Error: matplotlib not installed")
    print("Install with: pip install matplotlib")
    exit(1)

# Paths
RESULTS_FILE = Path("benches/results/benchmark_results.csv")
IMAGES_DIR = Path("docs/images")

# Ensure output directory exists
IMAGES_DIR.mkdir(parents=True, exist_ok=True)


def load_results():
    """Load benchmark results from CSV."""
    if not RESULTS_FILE.exists():
        print(f"Error: {RESULTS_FILE} not found")
        print("Run benchmarks first: cargo run --release --example bench_runner")
        exit(1)

    results = []
    with open(RESULTS_FILE, "r") as f:
        reader = csv.DictReader(f)
        for row in reader:
            results.append(
                {
                    "windows": int(row["windows"]),
                    "duration_secs": int(row["duration_secs"]),
                    "spawn_time_ms": float(row["spawn_time_ms"]),
                    "total_ops": int(row["total_ops"]),
                    "ops_per_sec": float(row["ops_per_sec"]),
                    "errors": int(row["errors"]),
                }
            )
    return results


def plot_spawn_time(results):
    """Plot window spawn time by window count."""
    # Get unique window counts and their spawn times (use first occurrence)
    spawn_data = {}
    for r in results:
        if r["windows"] not in spawn_data:
            spawn_data[r["windows"]] = r["spawn_time_ms"]

    windows = sorted(spawn_data.keys())
    times = [spawn_data[w] for w in windows]

    fig, ax = plt.subplots(figsize=(10, 6))
    bars = ax.bar(windows, times, color="#4CAF50", edgecolor="#2E7D32", linewidth=1.5)

    # Add value labels on bars
    for bar, time in zip(bars, times):
        height = bar.get_height()
        ax.annotate(
            f"{time:.0f}ms",
            xy=(bar.get_x() + bar.get_width() / 2, height),
            xytext=(0, 3),
            textcoords="offset points",
            ha="center",
            va="bottom",
            fontsize=11,
            fontweight="bold",
        )

    ax.set_xlabel("Number of Windows", fontsize=12)
    ax.set_ylabel("Spawn Time (ms)", fontsize=12)
    ax.set_title(
        "Firefox WebDriver - Window Spawn Performance", fontsize=14, fontweight="bold"
    )
    ax.set_xticks(windows)
    ax.grid(axis="y", alpha=0.3)

    plt.tight_layout()
    plt.savefig(IMAGES_DIR / "benchmark_spawn.png", dpi=150, bbox_inches="tight")
    plt.close()
    print(f"  ✓ {IMAGES_DIR / 'benchmark_spawn.png'}")


def plot_ops_per_second(results):
    """Plot operations per second by window count and duration."""
    fig, ax = plt.subplots(figsize=(12, 7))

    # Group by window count
    window_counts = sorted(set(r["windows"] for r in results))
    durations = sorted(set(r["duration_secs"] for r in results))

    colors = ["#2196F3", "#FF9800", "#4CAF50", "#E91E63"]
    width = 0.2
    x_positions = range(len(durations))

    for i, wc in enumerate(window_counts):
        ops = []
        for d in durations:
            matching = [
                r for r in results if r["windows"] == wc and r["duration_secs"] == d
            ]
            if matching:
                ops.append(matching[0]["ops_per_sec"])
            else:
                ops.append(0)

        offset = (i - len(window_counts) / 2 + 0.5) * width
        bars = ax.bar(
            [x + offset for x in x_positions],
            ops,
            width,
            label=f"{wc} windows",
            color=colors[i % len(colors)],
            edgecolor="black",
            linewidth=0.5,
        )

        # Add value labels
        for bar, op in zip(bars, ops):
            if op > 0:
                height = bar.get_height()
                ax.annotate(
                    f"{op:.0f}",
                    xy=(bar.get_x() + bar.get_width() / 2, height),
                    xytext=(0, 3),
                    textcoords="offset points",
                    ha="center",
                    va="bottom",
                    fontsize=9,
                )

    ax.set_xlabel("Test Duration (seconds)", fontsize=12)
    ax.set_ylabel("Operations per Second", fontsize=12)
    ax.set_title(
        "Firefox WebDriver - Sustained Operations Performance",
        fontsize=14,
        fontweight="bold",
    )
    ax.set_xticks(x_positions)
    ax.set_xticklabels([f"{d}s" for d in durations])
    ax.legend(loc="upper right")
    ax.grid(axis="y", alpha=0.3)

    # Format y-axis with commas
    ax.yaxis.set_major_formatter(ticker.StrMethodFormatter("{x:,.0f}"))

    plt.tight_layout()
    plt.savefig(IMAGES_DIR / "benchmark_ops.png", dpi=150, bbox_inches="tight")
    plt.close()
    print(f"  ✓ {IMAGES_DIR / 'benchmark_ops.png'}")


def main():
    print("Loading benchmark results...")
    results = load_results()
    print(f"  Found {len(results)} data points")
    print("")
    print("Generating plots...")
    plot_spawn_time(results)
    plot_ops_per_second(results)
    print("")
    print("Done!")


if __name__ == "__main__":
    main()
