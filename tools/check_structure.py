"""Validate the bounded, declared module homes; this is not a general Rust parser."""
import json
from pathlib import Path, PurePosixPath
import re

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def local_path(root, relative):
    path = (root / relative).resolve()
    require(path.is_relative_to(root.resolve()), f"path escapes repository: {relative}")
    return path


def check_structure(root, policy):
    require(type(policy.get("schema_version")) is int and policy["schema_version"] == 1,
            "unsupported structure schema")
    homes = policy["required_module_roots"]
    exceptions = policy["loose_file_exceptions"]
    for relative, exception in exceptions.items():
        require(isinstance(exception.get("reason"), str) and exception["reason"].strip(),
                "file exception needs an architectural reason")
        require(local_path(root, exception["evidence"]).is_file(), "file exception needs an evidence record")
        require(local_path(root, relative).is_file(), "stale file exception")
    count = 0
    for crate, paths in homes.items():
        require(re.fullmatch(r"astero-[a-z0-9-]+", crate), "invalid crate name")
        source = local_path(root, f"crates/{crate}/src")
        require(source.is_dir(), f"missing crate source: {crate}")
        require(isinstance(paths, list) and len(paths) == len(set(paths)), "duplicate/invalid module homes")
        for name in paths:
            require(isinstance(name, str) and all(re.fullmatch(r"[a-z][a-z0-9_]*", part)
                    for part in name.split("/")), f"invalid module home: {name}")
            relative = PurePosixPath(name)
            if len(relative.parts) > 1:
                require(str(relative.parent) in paths, f"missing parent home: {name}")
            directory = local_path(source, name)
            require((directory / "mod.rs").is_file(), f"missing module root: {crate}/{name}/mod.rs")
            require(not directory.with_suffix(".rs").exists(), f"loose file conflicts with module home: {crate}/{name}.rs")
            if len(relative.parts) == 1:
                parents = [source / "lib.rs", source / "main.rs"]
            else:
                parents = [directory.parent / "mod.rs"]
            declaration = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+"
                                     + re.escape(relative.name) + r"\s*;", re.MULTILINE)
            require(any(p.is_file() and declaration.search(p.read_text(encoding="utf-8")) for p in parents),
                    f"module home is not declared by parent: {crate}/{name}")
            count += 1
    # Only src/<growth-name>.rs is prohibited globally; nested legitimate leaf names remain possible.
    for source in (root / "crates").glob("*/src"):
        for path in source.rglob("*.rs"):
            relative = path.relative_to(root).as_posix()
            forbidden = (path.parent == source and path.name in policy["forbidden_top_level_filenames"])
            discouraged = path.name in policy["discouraged_filenames"]
            require(not (forbidden or discouraged) or relative in exceptions,
                    f"loose/dumping-ground file needs explicit architectural justification: {relative}")
    return count


def main():
    policy = json.loads((ROOT / "knowledge/architecture/module_structure.json").read_text(encoding="utf-8"))
    count = check_structure(ROOT, policy)
    print(f"Structure policy passed: {count} declared module homes.")


if __name__ == "__main__":
    main()
