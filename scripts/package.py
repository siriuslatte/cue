"""Package a compiler binary and, optionally, the Luau runtime."""

import argparse
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--output", type=Path, default=Path("dist"))
    parser.add_argument("--runtime", action="store_true")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)

    with ZipFile(args.output / f"cue-{args.target}.zip", "w", ZIP_DEFLATED) as archive:
        archive.write(args.binary, args.binary.name)
        archive.write("LICENSE")

    if args.runtime:
        with ZipFile(args.output / "cue-runtime.zip", "w", ZIP_DEFLATED) as archive:
            for path in sorted(Path("runtime").rglob("*")):
                if path.is_file():
                    archive.write(path, path.relative_to("runtime"))


if __name__ == "__main__":
    main()
