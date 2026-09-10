#!/usr/bin/env python3
"""Summarize saved wildlife trials; no simulation or fitted targets are hidden here."""
import gzip
import json
import sys
from pathlib import Path


def summarize(paths):
    print("| Trial | Seed | Open barriers | Years | Lake predator share¹ | Max C/N/P residual | Inner occupied guilds² |")
    print("|---|---:|---|---:|---:|---:|---:|")
    for path in paths:
        opener = gzip.open if path.suffix == ".gz" else open
        with opener(path, "rt") as source:
            rows = json.load(source)
        for row in rows:
            animal = row["after"]["carbon_kg"][1]
            fraction = animal[9] / max(sum(animal[8:11]), 1e-30)
            residual = max(abs(v) for v in row["budget"]["relative_error"][:3])
            occupied = sum(v > 0 for v in row["after"]["occupied_fraction"][2][:7])
            print(f"| {path.name.removesuffix('.gz').removesuffix('.json')} | "
                  f"{row['config']['seed']} | {row['config']['wildlife_open_barriers']} | "
                  f"{row['months'] / 12:g} | {fraction:.4%} | {residual:.3g} | {occupied}/7 |")
    print("\n¹ Predator carbon / (aquatic grazer + predator + migratory river animal carbon).")
    print("\n² Guilds 0–6 with any central-land area above 1e-10 kg C/m²; this is not species richness.")


if __name__ == "__main__":
    paths = [Path(p) for p in sys.argv[1:]] or sorted(
        Path("output/evidence/wildlife").glob("*.json.gz")
    )
    summarize(paths)
