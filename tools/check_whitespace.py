"""Supplemental source whitespace check when no Git worktree exists."""
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEXT_SUFFIXES = {".rs", ".toml", ".json", ".md", ".py", ".lock"}


def main():
    errors = []
    count = 0
    for directory, dirs, files in os.walk(ROOT):
        dirs[:] = [name for name in dirs if name not in {"target", ".git", "__pycache__"}]
        for name in files:
            path = Path(directory) / name
            if path.suffix not in TEXT_SUFFIXES and name not in {"LICENSE", ".gitignore"}:
                continue
            count += 1
            content = path.read_text(encoding="utf-8")
            relative = path.relative_to(ROOT)
            if content and not content.endswith("\n"):
                errors.append(f"{relative}: missing final newline")
            for number, line in enumerate(content.splitlines(), 1):
                if line.rstrip() != line or line.startswith(("<<<<<<< ", "=======", ">>>>>>> ")):
                    errors.append(f"{relative}:{number}: trailing whitespace/conflict marker")
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"Supplemental whitespace check passed: {count} files; not a Git diff check.")


if __name__ == "__main__":
    main()
