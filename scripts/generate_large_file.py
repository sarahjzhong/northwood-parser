"""Create an ASCII input for a manual large-file test without loading it into RAM."""

import argparse
from pathlib import Path
import shutil

GIB = 1024 ** 3
MIB = 1024 ** 2


def generate(filename, size, pattern):
    filename.parent.mkdir(parents=True, exist_ok=True)
    if size < 12:
        raise ValueError("file must be at least 12 bytes")
    if shutil.disk_usage(filename.parent).free < size + 2 * GIB:
        raise ValueError("not enough disk space to write the file and leave 2 GiB free")

    if pattern == "streaming":
        chunk = (b" " * 8191 + b"\n") * 128
        remaining = size - 12
    else:
        chunk = b"a\n" * (MIB // 2)
        remaining = size

    # exclusive creation avoids overwriting an existing fixture
    with filename.open("xb") as writer:
        if pattern == "streaming":
            writer.write(b"alpha\n")
        written = 0
        next_progress = GIB
        while remaining:
            amount = min(remaining, len(chunk))
            writer.write(chunk[:amount])
            remaining -= amount
            written += amount
            if written >= next_progress:
                print(f"wrote {written / GIB:.0f} GiB", flush=True)
                next_progress += GIB
        if pattern == "streaming":
            writer.write(b"\nomega")

    if pattern == "streaming":
        expected = filename.with_suffix(".expected.txt")
        expected.write_text(f"alpha 0\nomega {size - 5}\n", encoding="utf-8")
        print(f"expected output: {expected}")
    else:
        print(f"expected word: a; positions: 0, 2, ... {size - 2}")
    print(f"created {filename} ({filename.stat().st_size:,} bytes)")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sizes = parser.add_mutually_exclusive_group(required=True)
    sizes.add_argument("--size-gib", type=int)
    sizes.add_argument("--size-mib", type=int)
    parser.add_argument("--pattern", choices=["streaming", "index-growth"], default="streaming")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    size = args.size_gib * GIB if args.size_gib is not None else args.size_mib * MIB
    generate(args.output, size, args.pattern)


if __name__ == "__main__":
    main()
