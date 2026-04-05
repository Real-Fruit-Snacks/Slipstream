# Contributing to Slipstream

Thank you for your interest in contributing to Slipstream! This document provides guidelines and instructions for contributing.

## Development Environment Setup

### Prerequisites

- **Rust toolchain:** Install via [rustup](https://rustup.rs/) (stable channel, 1.70+)
- **OpenSSH:** Client version 6.8+ (for ControlMaster support)
- **Git:** For version control

### Getting Started

```bash
# Fork and clone the repository
git clone https://github.com/<your-username>/Slipstream.git
cd Slipstream

# Build
cargo build

# Run tests
cargo test

# Run the full lint suite
cargo fmt --check
cargo clippy -- -D warnings
```

## Code Style

All code must pass the following checks before submission:

- **Formatting:** `cargo fmt` — all code must be formatted with rustfmt
- **Linting:** `cargo clippy -- -D warnings` — zero warnings allowed
- **Tests:** `cargo test` — all tests must pass

Run all three before submitting a PR:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## Testing Requirements

- All existing tests must continue to pass: `cargo test`
- New features must include tests
- Integration tests go in the `tests/` directory
- Unit tests go in the source file using `#[cfg(test)]` modules

## Pull Request Process

1. **Fork** the repository and create a feature branch:
   ```bash
   git checkout -b feat/my-feature
   ```

2. **Make your changes** with clear, focused commits.

3. **Test thoroughly:**
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```

4. **Push** your branch and open a Pull Request against `main`.

5. **Describe your changes** in the PR using the provided template.

6. **Respond to review feedback** promptly.

## Commit Message Format

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<optional scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type       | Description                          |
| ---------- | ------------------------------------ |
| `feat`     | New feature                          |
| `fix`      | Bug fix                              |
| `docs`     | Documentation changes                |
| `style`    | Formatting, no code change           |
| `refactor` | Code restructuring, no behavior change |
| `test`     | Adding or updating tests             |
| `ci`       | CI/CD changes                        |
| `chore`    | Maintenance, dependencies            |
| `perf`     | Performance improvements             |

### Examples

```
feat(tunnel): add tunnel restore on reconnect
fix(transfer): handle Windows backslash paths in SCP
docs: update command reference for !map find
```

### Important

- Do **not** include AI co-author signatures in commits.
- Keep commits focused on a single logical change.

## Questions?

If you have questions about contributing, feel free to open a discussion or issue on GitHub.
