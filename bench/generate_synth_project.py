#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path


def generate_project(base: Path, count: int) -> None:
    base.mkdir(parents=True, exist_ok=True)
    for path in base.glob("file*.ts"):
        path.unlink()

    for i in range(1, count + 1):
        name = f"file{i:04d}"
        path = base / f"{name}.ts"
        if i == 1:
            content = "export interface Type0001 { value: number; }\n"
        else:
            prev = f"Type{i-1:04d}"
            content = (
                f"import {{ {prev} }} from \"./file{i-1:04d}\";\n"
                f"export interface Type{i:04d} {{ value: number; prev: {prev}; }}\n"
            )
        path.write_text(content, encoding="utf-8")

    index_path = base / "index.ts"
    index_path.write_text(
        f"import {{ Type{count:04d} }} from \"./file{count:04d}\";\n"
        f"export type Entry = Type{count:04d};\n",
        encoding="utf-8",
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate synthetic TS project for CLI benchmarks")
    parser.add_argument("--count", type=int, default=1000, help="Number of chained files")
    parser.add_argument(
        "--dir",
        default="synth",
        help="Output directory name under the bench directory",
    )
    args = parser.parse_args()

    bench_dir = Path(__file__).resolve().parent
    target = bench_dir / args.dir
    generate_project(target, args.count)
    print(f"Generated {args.count} files in {target}")


if __name__ == "__main__":
    main()
