"""CLI entrypoint for running PolyXML comparative benchmarks."""

from __future__ import annotations

import argparse
import sys

from rich.console import Console

from benchmarks.runner import BenchmarkRunner


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="PolyXML Comprehensive Multi-Parser Benchmark Runner"
    )
    parser.add_argument(
        "--workload",
        choices=["all", "sensor", "order", "catalog"],
        default="all",
        help="Select which workload to benchmark (default: all)",
    )
    parser.add_argument(
        "--catalog-sizes",
        type=int,
        nargs="+",
        default=[1000, 10000],
        help="List of catalog item counts to benchmark (default: 1000 10000)",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=25,
        help="Number of timed iterations for each test (default: 25)",
    )
    parser.add_argument(
        "--output-md",
        type=str,
        default=None,
        help="Path to save Markdown summary table",
    )
    parser.add_argument(
        "--output-json",
        type=str,
        default=None,
        help="Path to save raw benchmark results as JSON",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    console = Console(width=140)
    runner = BenchmarkRunner(console=console)

    console.print(
        "\n[bold cyan]🚀 Starting PolyXML Comparative Benchmarks...[/bold cyan]\n"
    )

    all_metrics = []

    if args.workload in ("all", "sensor"):
        console.print("[yellow]Running Sensor Telemetry Workload...[/yellow]")
        sensor_metrics = runner.run_workload_sensor(iterations=args.iterations * 4)
        all_metrics.extend(sensor_metrics)

    if args.workload in ("all", "order"):
        console.print("[yellow]Running Enterprise Order Workload...[/yellow]")
        order_metrics = runner.run_workload_order(
            num_items=10, iterations=args.iterations * 2
        )
        all_metrics.extend(order_metrics)

    if args.workload in ("all", "catalog"):
        for size in args.catalog_sizes:
            console.print(
                f"[yellow]Running Catalog Workload ({size:,} items)...[/yellow]"
            )
            cat_metrics = runner.run_workload_catalog(
                num_items=size, iterations=args.iterations
            )
            all_metrics.extend(cat_metrics)

    # Compute speedup comparisons
    runner.compute_speedups(all_metrics)

    # Render summary table
    console.print("\n")
    runner.display_rich_table(all_metrics)

    # Export if requested
    if args.output_md:
        runner.export_markdown(all_metrics, args.output_md)
        console.print(
            f"\n[green]✓ Markdown report exported to:[/green] {args.output_md}"
        )

    if args.output_json:
        runner.export_json(all_metrics, args.output_json)
        console.print(f"[green]✓ JSON metrics exported to:[/green] {args.output_json}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
