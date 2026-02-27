#!/usr/bin/env python3
"""
Scaffold a new Rust/Axum/SQLx API project from the malky template.

Usage:
    python3 scaffold.py <project_name> [destination_dir]

Arguments:
    project_name     Rust crate name (snake_case, e.g. my_api)
    destination_dir  Where to create the project (default: current directory)

The script creates <destination_dir>/<project_name>/ with all files in place.
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


def copy_tree(src: Path, dst: Path, name: str):
    dst.mkdir(parents=True, exist_ok=True)
    for item in src.iterdir():
        target = dst / item.name
        if item.is_dir():
            copy_tree(item, target, name)
        else:
            raw = item.read_text(encoding="utf-8")
            replaced = substitute(raw, name)
            target.write_text(replaced, encoding="utf-8")
            print(f"  created {target.relative_to(dst.parent.parent if dst.parent.name == name else dst.parent)}")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    project_name = sys.argv[1]
    dest_base = Path(sys.argv[2]) if len(sys.argv) > 2 else Path.cwd()
    project_dir = dest_base / project_name

    if project_dir.exists():
        print(f"Error: {project_dir} already exists.")
        sys.exit(1)

    print(f"Scaffolding '{project_name}' into {project_dir} ...")
    copy_tree(RESOURCES_DIR, project_dir, project_name)

    print(f"""
Done! Next steps:

  cd {project_dir}
  cargo build
  cargo test                    # requires a running Postgres (see .env.example)

Feature template is in src/features/example/ — rename or duplicate it for real resources.
Register new features in src/features/mod.rs and src/router.rs.
""")


if __name__ == "__main__":
    main()
