#!/usr/bin/env python3
"""PolyXML Multi-Language Version Synchronization Utility.

Ensures strict version parity across all language manifests:
- Cargo.toml (workspace root)
- crates/polyxml-python/pyproject.toml
- crates/polyxml-js/package.json
- bindings/java/pom.xml
- bindings/cpp/CMakeLists.txt
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

CARGO_PATH = REPO_ROOT / "Cargo.toml"
PYPROJECT_PATH = REPO_ROOT / "crates" / "polyxml-python" / "pyproject.toml"
PACKAGE_JSON_PATH = REPO_ROOT / "crates" / "polyxml-js" / "package.json"
POM_PATH = REPO_ROOT / "bindings" / "java" / "pom.xml"
CMAKE_PATH = REPO_ROOT / "bindings" / "cpp" / "CMakeLists.txt"


def normalize_version(ver: str) -> str:
    """Normalize version string by stripping leading 'v'."""
    v = ver.strip().lstrip("v")
    if not re.match(r"^\d+\.\d+\.\d+.*$", v):
        raise ValueError(f"Invalid semantic version format: '{ver}'")
    return v


def get_cargo_version() -> str:
    content = CARGO_PATH.read_text(encoding="utf-8")
    m = re.search(r'(?ms)\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', content)
    if not m:
        raise RuntimeError("Could not find [workspace.package].version in Cargo.toml")
    return m.group(1)


def get_pyproject_version() -> str:
    content = PYPROJECT_PATH.read_text(encoding="utf-8")
    m = re.search(r'(?ms)\[project\].*?^version\s*=\s*"([^"]+)"', content)
    if not m:
        raise RuntimeError("Could not find [project].version in pyproject.toml")
    return m.group(1)


def get_package_json_version() -> str:
    content = PACKAGE_JSON_PATH.read_text(encoding="utf-8")
    m = re.search(r'"version"\s*:\s*"([^"]+)"', content)
    if not m:
        raise RuntimeError("Could not find 'version' in package.json")
    return m.group(1)


def get_pom_version() -> str:
    content = POM_PATH.read_text(encoding="utf-8")
    m = re.search(r'<artifactId>polyxml</artifactId>\s*<version>([^<]+)</version>', content)
    if not m:
        raise RuntimeError("Could not find <artifactId>polyxml</artifactId> <version> in pom.xml")
    return m.group(1)


def get_cmake_version() -> str:
    content = CMAKE_PATH.read_text(encoding="utf-8")
    m = re.search(r'project\(polyxml_cpp\s+VERSION\s+([0-9A-Za-z.\-]+)', content)
    if not m:
        raise RuntimeError("Could not find project(polyxml_cpp VERSION ...) in CMakeLists.txt")
    return m.group(1)


def check_versions() -> bool:
    """Check that all manifests match the workspace Cargo.toml version."""
    canonical = get_cargo_version()
    versions = {
        "Cargo.toml": canonical,
        "pyproject.toml": get_pyproject_version(),
        "package.json": get_package_json_version(),
        "pom.xml": get_pom_version(),
        "CMakeLists.txt": get_cmake_version(),
    }

    all_matched = True
    print(f"Canonical workspace version: {canonical}")
    for name, ver in versions.items():
        matched = (ver == canonical)
        status = "MATCH" if matched else "MISMATCH"
        print(f"  [{status}] {name:16}: {ver}")
        if not matched:
            all_matched = False

    return all_matched


def set_versions(new_ver: str) -> None:
    """Atomically update all manifests to new_ver."""
    v = normalize_version(new_ver)
    print(f"Synchronizing all manifests to version: {v}")

    # 1. Cargo.toml
    cargo_text = CARGO_PATH.read_text(encoding="utf-8")
    cargo_text_new = re.sub(
        r'(\[workspace\.package\].*?^version\s*=\s*)"[^"]+"',
        rf'\g<1>"{v}"',
        cargo_text,
        flags=re.MULTILINE | re.DOTALL,
    )
    CARGO_PATH.write_text(cargo_text_new, encoding="utf-8")
    print(f"  Updated {CARGO_PATH.relative_to(REPO_ROOT)}")

    # 2. pyproject.toml
    py_text = PYPROJECT_PATH.read_text(encoding="utf-8")
    py_text_new = re.sub(
        r'(\[project\].*?^version\s*=\s*)"[^"]+"',
        rf'\g<1>"{v}"',
        py_text,
        flags=re.MULTILINE | re.DOTALL,
    )
    PYPROJECT_PATH.write_text(py_text_new, encoding="utf-8")
    print(f"  Updated {PYPROJECT_PATH.relative_to(REPO_ROOT)}")

    # 3. package.json
    pkg_text = PACKAGE_JSON_PATH.read_text(encoding="utf-8")
    pkg_text_new = re.sub(
        r'("version"\s*:\s*)"[^"]+"',
        rf'\g<1>"{v}"',
        pkg_text,
    )
    PACKAGE_JSON_PATH.write_text(pkg_text_new, encoding="utf-8")
    print(f"  Updated {PACKAGE_JSON_PATH.relative_to(REPO_ROOT)}")

    # 4. pom.xml
    pom_text = POM_PATH.read_text(encoding="utf-8")
    pom_text_new = re.sub(
        r'(<artifactId>polyxml</artifactId>\s*<version>)[^<]+(</version>)',
        rf'\g<1>{v}\g<2>',
        pom_text,
    )
    POM_PATH.write_text(pom_text_new, encoding="utf-8")
    print(f"  Updated {POM_PATH.relative_to(REPO_ROOT)}")

    # 5. CMakeLists.txt
    cmake_text = CMAKE_PATH.read_text(encoding="utf-8")
    cmake_text_new = re.sub(
        r'(project\(polyxml_cpp\s+VERSION\s+)[0-9A-Za-z.\-]+',
        rf'\g<1>{v}',
        cmake_text,
    )
    CMAKE_PATH.write_text(cmake_text_new, encoding="utf-8")
    print(f"  Updated {CMAKE_PATH.relative_to(REPO_ROOT)}")

    # 6. crates/polyxml-c/Cargo.toml (dependency version requirement)
    c_cargo_path = REPO_ROOT / "crates" / "polyxml-c" / "Cargo.toml"
    if c_cargo_path.exists():
        c_text = c_cargo_path.read_text(encoding="utf-8")
        c_text_new = re.sub(
            r'(polyxml\s*=\s*\{[^}]*version\s*=\s*)"[^"]+"',
            rf'\g<1>"{v}"',
            c_text,
        )
        c_cargo_path.write_text(c_text_new, encoding="utf-8")
        print(f"  Updated {c_cargo_path.relative_to(REPO_ROOT)}")

    print("All manifests successfully synchronized.")


def main() -> int:
    parser = argparse.ArgumentParser(description="Synchronize PolyXML multi-language versions.")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="Verify all manifests match Cargo.toml version.")
    group.add_argument("--set", metavar="VERSION", help="Set new version across all manifests.")
    group.add_argument("--get", action="store_true", help="Print canonical workspace version.")

    args = parser.parse_args()

    if args.get:
        print(get_cargo_version())
        return 0

    if args.check:
        success = check_versions()
        if not success:
            print("\nError: Version mismatch detected across manifests!", file=sys.stderr)
            return 1
        return 0

    if args.set:
        try:
            set_versions(args.set)
            return 0
        except Exception as e:
            print(f"Error setting version: {e}", file=sys.stderr)
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
