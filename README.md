# rootscan

Tracks the filesystem structure (paths only — never file contents) of a root
folder and its subdirectories into a single snapshot file, honoring
gitignore-like rules. Built in Rust on top of the `ignore` crate (the walker
engine behind ripgrep) for parallel traversal and correct `.gitignore`
semantics.

## Usage

```
rootscan <ROOT> [OPTIONS]
```

Options:

| Flag | Description |
| --- | --- |
| `-o, --output <PATH>` | Output file. Defaults to `<ROOT>/.rootscan.<ext>` based on the format. Automatically excluded from the scan. |
| `-f, --format <FMT>` | `yaml` (default), `json`, or `text`. |
| `-e, --exclude-file <FILE>` | Extra gitignore-like rule file, interpreted relative to every scanned directory (repeatable). |
| `--no-gitignore` | Do not read `.gitignore` / `.ignore` / global gitignore. |
| `-H, --hidden` | Include hidden entries (dotfiles, dot-directories). |
| `-L, --level <DEPTH>` | Maximum recursion depth below the root. |
| `--sorted` | Sort each directory's children (directories first, case-insensitive). |
| `-C, --dirs-only` | List only directories, omitting files. |

Examples:

```
rootscan ~/projects/app
rootscan . -f json -o /tmp/snapshot.json --sorted
rootscan . -e team.ignore -L 3 -C
```

## Output formats

`yaml` (default) — tree-shaped, files are empty mappings:

```yaml
proj:
  src:
    sub:
      util.rs: {}
    main.rs: {}
  Cargo.toml: {}
```

`json` — the same tree shape, JSON-encoded (key order preserved).
`text` — one relative path per line, directories with a trailing `/`:

```
src/
src/main.rs
Cargo.toml
```

## Ignore rules

By default every `.gitignore`-style rule found during the walk is honored,
even outside a git repository: nested `.gitignore` files, `!` negation,
anchoring, `**`, `.git/info/exclude`, global gitignore, and `.ignore` files.
Use `--no-gitignore` to disable, and `-e FILE` for additional rule files.

## Architecture

Vertical-slice layering; dependencies always point inward:

```
src/
├── main.rs        # entry point: CLI parse -> app::run -> exit code / error chain
├── io/            # Input/Output layer: CLI definition, file writer
├── app/           # Coordination layer: lifecycle (config -> scan -> tree -> serialize -> write)
├── ports/         # Ports: Scanner + Serializer traits, ScanOptions/ScanStats, OutputFormat
├── adapters/      # Adapters: FsWalker (ignore crate), YAML/JSON/text serializers
├── domain/        # Core processing: Entry model, tree building (pure, unit-tested)
└── error.rs       # Central error type with full `caused by` chains
```

Errors abort the lifecycle at the layer that owns them and are printed with
their complete source chain; unexpected failures exit with code 1.

## Build

```
cargo build --release   # 2.5 MB static binary, no runtime dependencies
cargo test              # unit tests for tree building and serializers
```
