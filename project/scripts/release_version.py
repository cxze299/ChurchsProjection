"""Shared 1.0 release numbering for local and Actions builds."""
import argparse
from pathlib import Path
import re
import subprocess


def normalize_version(value):
    match = re.fullmatch(r"v?(\d+)\.(\d+)(?:\.(\d+))?", value.strip())
    if not match:
        raise ValueError(f"版本号应为 1.0 或 1.0.0 格式：{value}")
    return ".".join(str(int(part or 0)) for part in match.groups())


def next_version(tags):
    patches = []
    for tag in tags:
        match = re.fullmatch(r"v?1\.0(?:\.(\d+))?", tag.strip())
        if match:
            patches.append(int(match[1] or 0))
    return f"1.0.{max(patches, default=-1) + 1}"


def local_version():
    root = Path(__file__).resolve().parents[1]
    tags = subprocess.check_output(["git", "tag", "--list"], cwd=root, text=True)
    return next_version(tags.splitlines())


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", default="")
    args = parser.parse_args()
    print(normalize_version(args.version) if args.version else local_version())
