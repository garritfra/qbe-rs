# Contributing to qbe-rs

## Quick Start

1. Fork and clone the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run checks: `cargo fmt` and `cargo clippy`
5. Run tests: `cargo test`
6. Update CHANGELOG.md in the "Unreleased" section if applicable
7. Submit a PR with a clear description of changes

## PR Requirements

- All tests must pass
- Code must be formatted with `rustfmt`
- No `clippy` warnings
- Changelog updated for user-facing changes
- PRs should address a single concern

## Pull Request Template

When submitting a PR, please include:
- Issue reference (if applicable)
- Brief description of changes
- Confirmation that tests and linting pass

## Release Process

See [docs/releasing.md](docs/releasing.md) for the release workflow.

## License

By contributing, you agree your contributions will be licensed under either the Apache License 2.0 or MIT license.
