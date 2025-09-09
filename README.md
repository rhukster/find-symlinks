# find-symlinks

Fast symlink finder written in Rust. It scans the current directory tree and reports any symbolic links that resolve to a given target. It features a compact TUI, fast parallel traversal, and optional JSON output.

## Install with Homebrew

- Prerequisite: Homebrew installed.

  - `brew tap rhukster/tap`
  - `brew install find-symlinks`

## Install from Precompiled Packages  

- https://github.com/rhukster/find-symlinks/releases

## Install from source

- Prerequisite: Rust toolchain (`cargo`).
- From this repo:
  - Build: `cargo build --release` → binary at `target/release/find-symlinks`
  - Install into `$HOME/.cargo/bin`: `cargo install --path .`
  - Install into a custom dir (e.g. `~/bin`): `cargo install --path . --root ~/bin`

Note: `--path` must point to the crate source directory (the one containing `Cargo.toml`). Use `--root` to choose where the compiled binary is installed. Ensure your chosen install dir is on your `PATH`.

## Usage

```
Fast symlink finder (Rust)

Usage: find-symlinks [OPTIONS] <TARGET>

Arguments:
  <TARGET>  Absolute path to target to match against

Options:
      --hidden              Scan hidden files and folders (on by default, matches `find`)
      --max-depth <N>       Maximum depth to recurse
      --no-tui              Disable TUI progress output
      --json                Emit JSON array of matches
      --respect-gitignore   Respect .gitignore during scan (off by default)
      --one-filesystem      Do not cross filesystem boundaries
      --threads <N>         Thread count for traversal (default: auto)
      --ignore <GLOB>       Additional ignore glob(s) (gitignore-style). Repeatable
      --ignore-file <PATH>  Additional ignore file(s) to load patterns from. Repeatable
      --include-heavy       Include heavy directories like node_modules, .cache, target (off by default)
      --color <COLOR>       Color output: auto, always, or never [default: auto] [possible values: auto, always, never]
      --no-stream           Disable streaming matches; only show final boxed summary
  -h, --help                Print help
  -V, --version             Print version
```

## Examples

- Basic scan with progress UI:
  - `find-symlinks /absolute/path/to/real/target`
- JSON output (paths relative to the working directory):
  - `find-symlinks /abs/target --json`
- Respect `.gitignore` and limit depth:
  - `find-symlinks /abs/target --respect-gitignore --max-depth 5`
- Provide extra ignore patterns / files:
  - `find-symlinks /abs/target --ignore "*.log" --ignore "tmp/**"`
  - `find-symlinks /abs/target --ignore-file .ignore-additions`
- Avoid heavy directories (default) vs include them:
  - Default excludes: `node_modules`, `.cache`, `target`, `build`, `dist`, `out`, `.git`, `.venv`, `venv`
  - To include: `--include-heavy`
- Disable the TUI and stream plain matches:
  - `find-symlinks /abs/target --no-tui`
- Show only the final boxed summary (no per-line streaming):
  - `find-symlinks /abs/target --no-stream`

## Behavior & Notes

- Hidden files/dirs: scanning is enabled by default (matches GNU `find` defaults).
- `.gitignore`: ignored by default; enable via `--respect-gitignore`.
- Filesystems: traversal may cross filesystems unless `--one-filesystem` is set.
- Output modes:
  - Default: streams matching symlink paths as they’re found, then prints a stats block.
  - `--no-stream`: suppress streaming and print a boxed list + stats at the end.
  - `--json`: prints a JSON array of matching paths (no TUI/stats).
- Performance: multi-threaded traversal and resolution (rayon). Progress bars use `indicatif`.
- Exit codes: non-zero on invalid options or when the target path cannot be resolved.

## Versioning & Build Number (internal reference)

- Set explicit version: `scripts/set-version.sh 0.1.1`
- Bump version: `scripts/bump-version.sh <patch|minor|major>`
- Build number:
  - Auto-incremented and embedded at compile time; `--version` prints e.g. `find-symlinks 0.1.0 (build 7)`.
  - For per-invocation increments, use wrapper: `scripts/build.sh --release` (sets `BUILD_NUMBER` and compiles).
  - CI sets `BUILD_NUMBER` to the GitHub `run_number` for reproducible release artifacts.
