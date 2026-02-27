#!/usr/bin/env python3
"""
Scaffold a new Rust/Axum/SQLx API project from the malky template.

Usage:
    python3 scaffold.py <project_name> [destination_dir]

Arguments:
    project_name     Rust crate name (snake_case, e.g. my_api)
    destination_dir  Where to create the project (default: current directory)

The script scaffolds files directly into <destination_dir> (default: current directory).
"""

import sys
import os
import shutil
from pathlib import Path

PLACEHOLDER = "__APP_NAME__"
SKILL_DIR = Path(__file__).parent
RESOURCES_DIR = SKILL_DIR / "resources"


def substitute(content: str, name: str) -> str:
    return content.replace(PLACEHOLDER, name)


def copy_tree(src: Path, dst: Path, name: str, root: Path = None):
    if root is None:
        root = dst
    dst.mkdir(parents=True, exist_ok=True)
    for item in src.iterdir():
        target = dst / item.name
        if item.is_dir():
            copy_tree(item, target, name, root)
        else:
            raw = item.read_text(encoding="utf-8")
            replaced = substitute(raw, name)
            target.write_text(replaced, encoding="utf-8")
            print(f"  created {target.relative_to(root)}")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    project_name = sys.argv[1]
    dest_base = Path(sys.argv[2]) if len(sys.argv) > 2 else Path.cwd()
    project_dir = dest_base

    print(f"Scaffolding '{project_name}' into {project_dir} ...")
    copy_tree(RESOURCES_DIR, project_dir, project_name)

    print(f"""
Done! Run:

  cargo build
  cargo test                    # requires a running Postgres (see .env.example)

Feature template is in src/features/example/ — rename or duplicate it for real resources.
Register new features in src/features/mod.rs and src/router.rs.
""")


if __name__ == "__main__":
    main()
