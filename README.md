# find_symlinks2

Fast Rust rewrite of `find_symlinks` to find all symbolic links under the current directory that resolve to a provided absolute target path. Adds a tiny TUI progress bar and parallel scanning.

Usage:

```
find_symlinks2 /absolute/path/to/target [--hidden] [--max-depth N] [--no-tui] [--json]
```

- Prints matches as lines, identical semantics to the original ("MATCH: <path>").
- Exits non‑zero on incorrect or unresolved target.
- Uses `ignore` crate for fast walking (respects .gitignore by default).
- Verifies a symlink's final resolved path equals the resolved target (`realpath`).
- Traversal runs in parallel via rayon.

