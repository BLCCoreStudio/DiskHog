# DiskHog

**Find the files and directories consuming the most disk space.**

DiskHog is a small, read-only Rust CLI for finding the files and directories using the most disk space.

It is Linux-first, dependency-free at runtime/build time beyond the Rust standard library, and designed to stay predictable: no deletion, no telemetry, no backend, and no symlink traversal.

## Status

**DiskHog v0.1.0 is available as the first public release.**

A prebuilt Linux x86_64 archive and SHA-256 checksum are available on the [GitHub Releases page](https://github.com/BLCCoreStudio/DiskHog/releases/tag/v0.1.0).

## Features

- Scan a directory with `diskhog .`
- Show the 20 largest entries by default
- Change the result count with `--limit N`
- Show only files with `--files`
- Show only directories with `--dirs`
- Limit displayed traversal depth with `--depth N`
- Human-readable B/KiB/MiB/GiB/TiB sizes
- Report unreadable paths without crashing the whole scan
- Never follow symbolic links, preventing symlink recursion loops
- Sort results from largest to smallest

On Unix systems DiskHog uses allocated filesystem blocks, which better reflects real disk consumption for sparse files. On non-Unix platforms it falls back to the file length reported by the standard library.

## Install on Linux x86_64

Download these files from the [v0.1.0 release](https://github.com/BLCCoreStudio/DiskHog/releases/tag/v0.1.0):

- `diskhog-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `diskhog-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256`

Verify and extract:

```bash
sha256sum -c diskhog-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -xzf diskhog-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
./diskhog --version
```

You can optionally place `diskhog` somewhere on your `PATH`, such as `~/.local/bin`.

## Build from source

```bash
git clone https://github.com/BLCCoreStudio/DiskHog.git
cd DiskHog
cargo build --release --locked
```

The binary will be at `target/release/diskhog` (`diskhog.exe` on Windows).

## Usage

```text
diskhog [OPTIONS] [PATH]
```

If `PATH` is omitted, DiskHog scans the current directory.

```bash
# Largest files and directories under the current directory
diskhog .

# Top 50
diskhog --limit 50 .

# Files only
diskhog --files .

# Directories only
diskhog --dirs .

# Display only entries up to two levels below the root
diskhog --depth 2 .
```

`--depth` controls what is displayed, not how directory totals are calculated. A directory shown at depth 1 still includes the space used by deeper descendants.

## Example output

```text
      SIZE  TYPE  PATH
      ----  ----  ----
   2.4 GiB  dir   ./target
 812.0 MiB  file  ./archive.img
 128.0 MiB  dir   ./assets
```

Exact values depend on the filesystem.

## Safety and privacy

DiskHog only reads filesystem metadata and directory entries needed for analysis. It has no delete command, does not modify scanned content, does not follow symlinks, and contains no network, telemetry, account, or server functionality.

Permission errors and other unreadable paths are written as warnings to stderr while readable paths continue to be analyzed.

## Development checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

CI runs these checks on Linux and also runs the test suite on Linux, macOS, and Windows.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting guidance.

## License

MIT. See [LICENSE](LICENSE).

Built by **BLC Core Studio**.