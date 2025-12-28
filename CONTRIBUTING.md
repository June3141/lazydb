# Contributing to lazydb

Thank you for your interest in contributing to lazydb! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We welcome contributors of all experience levels.

## Getting Started

### Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **Docker and Docker Compose** - For running the development database
- **Task** - Task runner ([installation guide](https://taskfile.dev/installation/))
- **markdownlint-cli2** - Markdown linter ([GitHub](https://github.com/DavidAnson/markdownlint-cli2))

### Setting Up the Development Environment

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/<your-username>/lazydb.git
   cd lazydb
   ```

2. **Set up environment variables**

   ```bash
   cp .env.sample .env
   ```

3. **Start the development database**

   ```bash
   task docker:up
   ```

4. **Build the project**

   ```bash
   cargo build
   ```

5. **Run the application**

   ```bash
   cargo run
   ```

## Development Workflow

### Available Tasks

Use the Task runner for common development tasks:

```bash
# Show all available tasks
task

# Format all code (Rust + Markdown)
task format

# Lint all code
task lint

# Run all checks
task check

# Fix auto-fixable issues
task fix
```

### Rust-specific Tasks

```bash
task rust:build         # Build the project
task rust:build-release # Build in release mode
task rust:test          # Run tests
task rust:format        # Format Rust code
task rust:lint          # Run Clippy
task rust:fix           # Auto-fix issues
```

### Docker Tasks

```bash
task docker:up    # Start PostgreSQL
task docker:down  # Stop PostgreSQL
task docker:logs  # View logs
task docker:psql  # Connect with psql
task docker:reset # Reset database
```

## Making Changes

### Branch Naming

Create a feature branch from `main`:

```bash
git checkout -b feature/amazing-feature
git checkout -b fix/bug-description
git checkout -b docs/documentation-update
```

### Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification with [gitmoji](https://gitmoji.dev/):

```text
<type>: <emoji> <description>

[optional body]

[optional footer(s)]
```

**Types:**

| Type       | Emoji | Description                    |
| ---------- | ----- | ------------------------------ |
| `feat`     | ✨     | New feature                    |
| `fix`      | 🐛     | Bug fix                        |
| `docs`     | 📝     | Documentation only changes     |
| `style`    | 💄     | Code style (formatting, etc.)  |
| `refactor` | ♻️     | Code refactoring               |
| `test`     | ✅     | Adding or updating tests       |
| `chore`    | 🔧     | Maintenance tasks              |
| `ci`       | 👷     | CI/CD changes                  |
| `perf`     | ⚡     | Performance improvements       |

**Examples:**

```text
feat: ✨ add database trigger listing functionality
fix: 🐛 constrain data table navigation within page boundaries
docs: 📝 update README to reflect v0.1.0 MVP features
refactor: ♻️ split large files into smaller focused modules
```

### Code Style

- **Rust**: Run `cargo fmt` before committing. We use default rustfmt settings.
- **Clippy**: Ensure `cargo clippy -- -D warnings` passes without warnings.
- **Markdown**: Run `task markdown:format` to format markdown files.

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Pull Request Process

1. **Ensure your code passes all checks**

   ```bash
   task check
   cargo test
   ```

2. **Update documentation** if you're adding new features

3. **Create a Pull Request** with a clear title and description

4. **Link related issues** using keywords like `Closes #123` or `Fixes #456`

### PR Title Format

Follow the same format as commit messages:

```text
feat: ✨ add new feature description
fix: 🐛 fix bug description
```

## Project Structure

```text
src/
├── main.rs          # Entry point
├── app/             # Application state and logic (TEA pattern)
│   ├── mod.rs
│   ├── state.rs     # App state
│   ├── handlers/    # Message handlers
│   └── modals/      # Modal dialog logic
├── config/          # Configuration management
├── db/              # Database abstraction layer
│   ├── postgres/    # PostgreSQL implementation
│   └── worker/      # Async DB worker
├── event/           # Keyboard event handling
├── export/          # Data export (CSV, JSON)
├── model/           # Data models
│   └── schema/      # Database schema models
└── ui/              # UI components
    ├── modal/       # Modal dialogs
    ├── panel/       # Main panel tabs
    └── sidebar.rs   # Sidebar component
```

## Architecture

lazydb follows **The Elm Architecture (TEA)** pattern:

1. **Model** - Application state (`App` struct in `src/app/state.rs`)
2. **View** - UI rendering (`src/ui/` module)
3. **Update** - State updates via messages (`src/message.rs` and handlers)

For more details, see [docs/architecture.md](docs/architecture.md).

## Reporting Issues

When reporting issues, please include:

- **lazydb version** (or commit hash)
- **Operating system** and version
- **Database type** and version (if applicable)
- **Steps to reproduce** the issue
- **Expected behavior** vs. **actual behavior**
- **Error messages** or logs (if any)

## Feature Requests

Feature requests are welcome! Please:

- Check existing issues to avoid duplicates
- Describe the use case and motivation
- Explain how the feature should work

## Questions?

Feel free to open an issue for questions about contributing.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
