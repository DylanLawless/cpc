# cpc: copy code

Copy files and directories into **one clipboard entry** with filenames and an optional exact token count under a selected encoding. A fast everyday command for sharing source code with AI assistants.

```console
$ cpc src/ tests/
Copied 38 files | 154,280 chars | 39,614 tokens (o200k_base)
```

*Illustrative output; actual counts depend on selected files.*

- macOS, Linux desktop and Windows
- A local native Rust executable; no accounts, telemetry, backend or runtime API requests
- Mixed file and directory arguments, with deduplication and predictable ordering
- Respect `.gitignore`, `.ignore`, hidden entries and built-in exclusions on recursive scans
- Original plain-text `==== filename ====` headers, enabled by default
- Local `o200k_base` and `cl100k_base` token counts; disable for speed
- One-line terminal receipt; raw content goes to the clipboard or stdout

## Install

**Build from source** (Rust 1.85 or later):

```sh
git clone YOUR-REPOSITORY-URL
cd cpc
cargo install --path .
```

Replace `YOUR-REPOSITORY-URL` with the published Git URL. During local development, run `cargo run -- <arguments>` without installing.

For downloadable binaries, create a tagged GitHub Release using the included release workflow after cross-platform CI succeeds. Installing a release binary needs no Rust toolchain.

### Linux clipboard support

`cpc` uses `wl-copy` under Wayland or `xclip`/`xsel` under X11. Install one for your desktop, for example:

```sh
# Debian/Ubuntu, Wayland
sudo apt install wl-clipboard

# Debian/Ubuntu, X11
sudo apt install xclip
```

For SSH sessions or headless environments, use `--stdout`. On macOS the tool uses `pbcopy`; on Windows it uses the native clipboard API.

## Usage

```sh
cpc main.py
cpc main.py config.json tests/
cpc src/
cpc .
cpc src/ --no-headers
cpc src/ --no-tokens
cpc src/ --tokenizer cl100k_base
cpc src/ --stdout > context.txt
cpc --config examples/config.toml src/
cpc . --hidden
cpc . --no-ignore
cpc --help
```

By default, files are copied like this:

```text
==== src/main.py ====
print('hello')

==== src/config.py ====
DEBUG = False
```

The program adds one newline after each file, exactly as the original Bash `cpc` command does. It does not normalise the original content's line endings.

## Configuration

**Optional.** Without a file, sensible defaults apply. Copy [examples/config.toml](examples/config.toml) to:

- macOS: `~/.config/cpc/config.toml`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/cpc/config.toml`
- Windows: `%APPDATA%\cpc\config.toml`

```toml
[output]
headers = true

[selection]
respect_ignore_files = true
include_hidden = false
exclude = []

[report]
tokenizer = "o200k_base" # or "cl100k_base", "off"

[limits]
max_files = 5000
max_bytes = 50000000
```

Settings resolve in this order: built-in defaults, configuration file, CLI switches. Bad settings are errors rather than silently ignored.

**Selection rules:** Recursive scans respect Git ignore rules and built-in exclusions for common dependencies, output folders and likely credential files. An *explicitly named file* is included even if hidden or ignored, including `.env`. Known sensitive filenames produce a one-line `sensitive input` warning in the receipt. Double-check what you paste. Symlinked directories are not followed; explicit symlinks to regular files are allowed. Binary or non-UTF-8 files discovered within directories are skipped and counted, but explicitly selecting one is an error. A missing or unreadable file, or exceeding a limit, aborts before the clipboard is changed.

## Token counts

Token counts include the **entire formatted payload**, including headers and inserted separators. `o200k_base` and `cl100k_base` count local raw-text encoding tokens, not complete billed API message tokens. Different models, message wrappers, system prompts and special-token processing can produce different API usage totals. Choose the encoding to match your workflow; we do not claim to infer it from a model name.

`--no-tokens` (or `tokenizer = "off"`) skips tokeniser initialisation and counting. Character counts are always Unicode scalar values, not bytes, words or grapheme clusters.

## Trust and privacy

The tool only reads files you select, runs entirely locally, and does **not** make network requests at runtime. It neither executes files nor installs an updater. There is no telemetry. Clipboard content is accessible to the system's clipboard services and to applications into which you paste it. Exclusions are a best-effort safeguard, **not** a secret scanner.

## Develop and test

See [docs/BUILD_AND_TEST.md](docs/BUILD_AND_TEST.md) for commands, platform tests, release instructions, and the migration procedure from a `.bash_profile` function.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
```

## Release status

This source archive includes the implementation, tests, CI and release workflows. Native compilation and cross-platform runtime testing have **not** been performed in the environment used to assemble the archive because Rust/Cargo were unavailable there. Run the documented validation steps and generate/commit `Cargo.lock` before publishing a release. Do not treat the repository as a verified binary distribution until CI passes.

## Licence

MIT. See [LICENSE](LICENSE).
