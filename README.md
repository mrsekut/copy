# Copy - File Downloader

A Rust CLI tool to interactively browse and download files from GitHub repositories.

## Features

- Lists repositories for a default GitHub user
- Interactive file selection using fzf
- Downloads selected files using curl
- Preserves directory structure when copying files

## Installation

### Prerequisites

- Rust (install via [rustup](https://rustup.rs/))
- fzf (install via `brew install fzf` on macOS)
- curl (usually pre-installed on most systems)

### Building from Source

```bash
cargo install --git https://github.com/mrsekut/copy
```

## Usage

1. Run the program:

```bash
copy
```

2. Select a repository from the list (uses fzf)

3. Select a file from the repository (uses fzf)

4. The selected file will be downloaded to your current directory, preserving its path structure.

## License

MIT
