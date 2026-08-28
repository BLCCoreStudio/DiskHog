# Contributing to DiskHog

Thanks for helping improve DiskHog.

## Development

Requirements:

- Rust 1.85 or newer
- Git

Run the same checks used by CI before opening a pull request:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Pull requests

Keep changes focused and small. Add or update tests for behavior changes. DiskHog must remain read-only: features that delete, overwrite, upload, or silently transmit filesystem data are out of scope.

Avoid adding dependencies unless the benefit clearly outweighs the maintenance and supply-chain cost.

## Bug reports

Include the operating system, DiskHog version or commit, command used, expected behavior, and actual behavior. Do not paste secrets or sensitive directory contents.
