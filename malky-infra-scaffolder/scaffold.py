#!/usr/bin/env python3
"""
Scaffold the infra/ directory for a new project.

Usage:
    python3 scaffold.py <project_name> [destination_dir]

Arguments:
    project_name     Lowercase name used for compose project name, DB user/pass/name
    destination_dir  Project root where infra/ is written (default: cwd)

Generates:
    <dest>/infra/docker-compose.yml        dev DB  on :6432 (persistent volume)
    <dest>/infra/docker-compose.test.yml   test DB on :6433 (tmpfs, ephemeral)
"""

import sys
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
            replaced = substitute(item.read_text(encoding="utf-8"), name)
            target.write_text(replaced, encoding="utf-8")
            rel = target.relative_to(dst.parent)
            print(f"  created {rel}")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    name = sys.argv[1]
    dest = Path(sys.argv[2]) if len(sys.argv) > 2 else Path.cwd()

    print(f"Scaffolding infra for '{name}' into {dest} ...")
    copy_tree(RESOURCES_DIR, dest, name)

    print(f"""
Done! Next steps:

  # Start dev DB (persistent volume)
  podman compose -f infra/docker-compose.yml up -d

  # Start test DB (ephemeral, for integration tests)
  podman compose -f infra/docker-compose.test.yml up -d

Dev DB:   postgres://{name}:{name}@localhost:6432/{name}
Test DB:  postgres://{name}_test:{name}_test@localhost:6433/postgres
""")


if __name__ == "__main__":
    main()
