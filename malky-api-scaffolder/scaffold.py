#!/usr/bin/env python3
"""
Scaffold a complete Rust API project (workspace + infra + README + justfile).

Usage:
    python3 scaffold.py <project_name> [destination_dir]

Arguments:
    project_name     Snake_case name for the project (e.g. my_api)
    destination_dir  Where to create <project_name>/ (default: current directory)

What gets created:
    <project_name>/
      Cargo.toml                  workspace root (shared deps)
      justfile                    build/run/test recipes
      README.md
      .env.example                pre-filled connection strings
      apps/api/                   Rust crate (via malky-rust-scaffolder)
        Cargo.toml                patched to use { workspace = true }
        src/ migrations/ tests/
      infra/                      compose files (via malky-infra-scaffolder)
        docker-compose.yml
        docker-compose.test.yml
"""

import sys
import subprocess
from pathlib import Path

PLACEHOLDER = "__APP_NAME__"
SKILL_DIR = Path(__file__).parent
RESOURCES_DIR = SKILL_DIR / "resources"

RUST_SCAFFOLD = Path.home() / ".claude/skills/malky-rust-scaffolder/scaffold.py"
INFRA_SCAFFOLD = Path.home() / ".claude/skills/malky-infra-scaffolder/scaffold.py"


def substitute(content: str, name: str) -> str:
    return content.replace(PLACEHOLDER, name)


def write(path: Path, content: str, name: str):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(substitute(content, name), encoding="utf-8")
    print(f"  created {path}")


def run_scaffolder(script: Path, name: str, dest: Path, label: str):
    if not script.exists():
        print(f"  ERROR: {label} not found at {script}")
        print(f"         Install malky-rust-scaffolder and malky-infra-scaffolder first.")
        sys.exit(1)
    print(f"\n[{label}]")
    result = subprocess.run(
        [sys.executable, str(script), name, str(dest)],
        capture_output=False,
    )
    if result.returncode != 0:
        print(f"  ERROR: {label} failed (exit {result.returncode})")
        sys.exit(result.returncode)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    name = sys.argv[1]
    dest_base = Path(sys.argv[2]) if len(sys.argv) > 2 else Path.cwd()
    project_dir = dest_base / name

    if project_dir.exists():
        print(f"Error: {project_dir} already exists.")
        sys.exit(1)

    project_dir.mkdir(parents=True)
    print(f"Scaffolding '{name}' into {project_dir} ...\n")

    # 1. Rust crate into apps/<name>/, then rename to apps/api/
    apps_dir = project_dir / "apps"
    run_scaffolder(RUST_SCAFFOLD, name, apps_dir, "malky-rust-scaffolder")
    (apps_dir / name).rename(apps_dir / "api")
    print(f"  renamed apps/{name}/ → apps/api/")

    # 2. Patch apps/api/Cargo.toml → workspace = true version
    print("\n[workspace patch]")
    ws_api_cargo = RESOURCES_DIR / "api-Cargo.toml"
    target_api_cargo = project_dir / "apps" / "api" / "Cargo.toml"
    write(target_api_cargo, ws_api_cargo.read_text(encoding="utf-8"), name)

    # 3. Root workspace Cargo.toml
    print("\n[workspace root]")
    ws_cargo = RESOURCES_DIR / "Cargo.toml"
    write(project_dir / "Cargo.toml", ws_cargo.read_text(encoding="utf-8"), name)

    # 4. Infra (compose files + .env.example) into project root
    run_scaffolder(INFRA_SCAFFOLD, name, project_dir, "malky-infra-scaffolder")

    # 5. justfile + README + .gitignore + .env.example
    print("\n[justfile + README + .gitignore + .env.example]")
    for filename in ("justfile", "README.md", ".gitignore", ".env.example"):
        src = RESOURCES_DIR / filename
        write(project_dir / filename, src.read_text(encoding="utf-8"), name)

    print(f"""
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  {name} is ready
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  cd {project_dir}
  cp .env.example .env
  just db-up
  just run

Next: rename apps/api/src/features/example/ to your first resource,
      update src/features/mod.rs and src/router.rs accordingly.
""")


if __name__ == "__main__":
    main()
