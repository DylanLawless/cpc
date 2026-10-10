# cpc: copy code

**The missing copy command.**

`Copy` and `paste`. Two of the first things we learn to do on a computer.

Copy files and entire directories into one labelled clipboard entry, ready to paste anywhere. `cpc` preserves filenames and reports character and token counts.

<img src="https://github.com/user-attachments/assets/2bd2daed-713d-413f-a1cc-baf2d7ffc24c"
     alt="And now I simply grant it root access instead of copying and pasting."
     width="700">

## Example: sharing code with AI

When an AI assistant asks for several source files, copying them individually is tedious. You may also prefer to share selected files without granting an agent direct access to your filesystem.

`cpc` collects the requested files in one command:

```console
$ cpc main.py config.py src/
Copied 14 files | 54,280 chars | 13,614 tokens (o200k_base)
```

The clipboard contains:

```text
==== main.py ====
...

==== config.py ====
...

==== src/utils.py ====
...
```

Paste the result into your AI conversation. You control which files to share, and the assistant can identify each file by its path.

- **One command:** Copy files, directories or both, with duplicate files removed.
- **Ready to paste:** Include filenames by default and check token counts before sharing.
- **Local:** Runs on macOS, Linux desktops and Windows, without accounts, telemetry or a backend.

***Pro tip:** ask your AI to list all the files it needs in one line. 
Then copy them all at once like this:* `cpc file1 file2 directory/ file*`.


## Installation

Requires [Rust and Cargo](https://rustup.rs/) (Rust 1.85 or later).

Install directly from GitHub:

```sh
cargo install --git https://github.com/DylanLawless/cpc.git --bin cpc
```

Or clone the repository anywhere and install from source:

```sh
git clone https://github.com/DylanLawless/cpc.git
cd cpc
cargo install --path .
```

Cargo installs the executable in `~/.cargo/bin/` on macOS/Linux or `%USERPROFILE%\.cargo\bin\` on Windows. Ensure the directory is on `PATH`, then verify:

```sh
cpc --version
cpc --help
```

## Usage

```sh
cpc main.py                          # One file
cpc main.py config.json tests/       # Mixed files and directories
cpc src/                             # Entire directory
cpc .                                # Current directory
cpc src/ --no-headers                # Omit filename headers
cpc src/ --no-tokens                 # Disable token counting
cpc src/ --tokenizer cl100k_base     # Use another encoding
cpc src/ --stdout > context.txt      # Write to a file instead
cpc . --hidden                       # Include hidden entries
cpc . --no-ignore                    # Disable ignore-file rules
cpc --config path/to/config.toml src/
```

By default, `cpc` adds a `==== filename ====` header and one newline after each file. File contents and their original line endings are preserved. The summary goes to the terminal's standard error stream; `--stdout` writes only the formatted content to standard output.

### Linux clipboard

On Linux desktops, install `wl-clipboard` for Wayland or `xclip`/`xsel` for X11. For example, on Debian/Ubuntu:

```sh
sudo apt install wl-clipboard  # Wayland
# or
sudo apt install xclip         # X11
```

Use `--stdout` in SSH or headless environments. macOS uses `pbcopy`; Windows uses the native clipboard API.

## Configuration (optional)

`cpc` works without configuration. Defaults include filename headers, `o200k_base` token counting, Git-aware exclusions, a 5,000-file limit and a 50,000,000-byte payload limit.

To change the defaults, **after cloning the repository**, run from its root on macOS/Linux:

```sh
mkdir -p ~/.config/cpc
cp examples/config.toml ~/.config/cpc/config.toml
```

On Linux, use `$XDG_CONFIG_HOME/cpc/config.toml` if `XDG_CONFIG_HOME` is set. On Windows, copy `examples/config.toml` to `%APPDATA%\cpc\config.toml`. If you installed directly through Cargo, create the configuration file using [this example](examples/config.toml).

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

CLI options take precedence over configuration, which overrides built-in defaults. Invalid settings produce an error.

## Updating and uninstalling

If you cloned the repository, update from its root:

```sh
git pull
cargo install --path .
```

For a direct installation from GitHub:

```sh
cargo install --git https://github.com/DylanLawless/cpc.git --bin cpc --force
```

Uninstall either Cargo installation with:

```sh
cargo uninstall copy-code-cli
```

The source repository and personal configuration remain until you delete them.

## File selection and privacy

Directory scans respect `.gitignore`, `.ignore`, hidden-file settings and built-in exclusions for common dependency, build and credential files. Explicitly named files override these exclusions, including `.env`. Known sensitive filenames trigger a warning, so check what you paste. Directory symlinks are not followed.

Binary or non-UTF-8 files discovered during directory scans are skipped. Explicitly selecting such a file produces an error. Missing or unreadable files and exceeded limits also produce errors before the clipboard is changed.

All processing is local. `cpc` does not upload files, execute their contents, send telemetry or update itself. Clipboard contents may be accessible to other applications. Filename exclusions do not constitute a secret scanner.

## Token counts

Estimates your token count before pasting files.
Use the default or choose the encoding in `[report]` of your `config.toml`:

| Model family | `tokenizer` | Notes |
| --- | --- | --- |
| OpenAI GPT-4o, GPT-4.1 and compatible models | `"o200k_base"` | Native encoding for these models |
| OpenAI GPT-4 and GPT-3.5 Turbo | `"cl100k_base"` | Native encoding for these older models |
| Claude (Anthropic), Gemini (Google) | `"o200k_base"` or `"off"` | OpenAI-encoding proxy only; native tokenisers are not supported |
| Llama, Qwen, DeepSeek, Mistral | `"o200k_base"` or `"off"` | OpenAI-encoding proxy only; model-specific tokenisers are not supported |

`cpc` counts the complete formatted text, including filenames and separators. Counts are exact **for the selected encoding**, but may differ from a model's input-token usage. `"off"` or `--no-tokens` disables counting. Character counts use Unicode scalar values.

## Development and testing

See [docs/BUILD_AND_TEST.md](docs/BUILD_AND_TEST.md) for platform tests and release instructions.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
```

**Release status:** The initial source package has not yet been verified on all target operating systems. Run the checks above, generate and commit `Cargo.lock`, and confirm CI passes before publishing release binaries.

## Licence

MIT. See [LICENSE](LICENSE).
