# lazydb

A simple terminal UI for database management, inspired by [lazygit](https://github.com/jesseduffield/lazygit) and [lazydocker](https://github.com/jesseduffield/lazydocker).

> ⚠️ **Work in Progress** - This project is in early development.

## Overview

lazydb is a TUI (Terminal User Interface) tool that allows developers to interact with databases
directly from the terminal. It aims to provide a lightweight alternative to GUI database clients
like DBeaver, focusing on keyboard-driven workflows.

## Features (Planned)

- 🔌 **Connection Manager** - Add, edit, and delete database connections interactively
- 📊 **Table Browser** - Browse database schemas, tables, and views
- 🔍 **Data Viewer** - View and filter table data with pagination
- ✏️ **Data Editor** - Edit records directly in the terminal
- 💻 **Query Executor** - Write and execute SQL queries
- 📝 **Query History** - Access previously executed queries
- ⌨️ **Vim-style Keybindings** - Efficient keyboard navigation

## Supported Databases

| Database   | Status      |
| ---------- | ----------- |
| PostgreSQL | 🚧 Planned  |
| MySQL      | 📋 Roadmap  |
| SQLite     | 📋 Roadmap  |

## Installation

### From source

```bash
git clone https://github.com/June3141/lazydb.git
cd lazydb
cargo build --release
```

### Using Cargo

```bash
cargo install lazydb  # Coming soon
```

## Usage

```bash
# Launch lazydb
lazydb

# Or connect directly to a database
lazydb "postgres://user:password@localhost:5432/mydb"
```

### Connection Management

lazydb provides an interactive connection manager where you can:

- **Add** new database connections with a simple form
- **Edit** existing connection settings
- **Delete** connections you no longer need
- **Test** connections before saving

## Configuration

### Config Directory Location

Configuration files are stored in the OS-specific config directory:

| OS      | Path                                      |
| ------- | ----------------------------------------- |
| Linux   | `~/.config/lazydb/`                       |
| macOS   | `~/.config/lazydb/`                       |
| Windows | `C:\Users\<User>\AppData\Roaming\lazydb\` |

### Directory Structure

```text
<config_dir>/
├── config.yaml              # Main configuration file
└── projects/
    ├── my-project.yaml      # Project configuration files
    └── another-project.yaml
```

### Main Configuration (`config.yaml`)

```yaml
settings:
  default_project: my-project
  theme: dark
  show_row_count: true

projects:
  # Relative paths (from config directory)
  - projects/my-project.yaml
  - projects/another-project.yaml

  # Absolute paths
  - /shared/team/shared-project.yaml

  # Home directory expansion
  - ~/work/client-a/.lazydb-project.yaml
```

### Project Configuration (`projects/*.yaml`)

```yaml
project:
  name: My Project
  description: Project description

connections:
  - name: Production
    host: prod.example.com
    port: 5432
    database: app_production
    username: dbuser
    password_env: LAZYDB_PROD_PASSWORD  # Read from environment variable

  - name: Development
    host: localhost
    port: 5432
    database: app_dev
    username: dev
    password: dev123  # Direct password (for local development only)
```

### Password Management

Passwords can be configured in two ways:

- `password`: Direct password string (not recommended for production)
- `password_env`: Environment variable name containing the password (recommended)

## Keybindings

| Key          | Action                    |
| ------------ | ------------------------- |
| `↑/k`        | Navigate up               |
| `↓/j`        | Navigate down             |
| `←/h`        | Navigate left             |
| `→/l`        | Navigate right            |
| `Enter`      | Select / Confirm          |
| `q`          | Quit / Back               |
| `a`          | Add new item              |
| `d`          | Delete item               |
| `e`          | Edit item                 |
| `/`          | Search / Filter           |
| `:`          | Command mode              |
| `?`          | Show help                 |

## Documentation

- [Architecture](docs/architecture.md) - TEA パターンとディレクトリ構成

## Requirements

- Rust 1.75+ (for building from source)

### Development Tools

The following tools are required for development:

- [Task](https://taskfile.dev/) - Task runner
- [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2) - Markdown linter / formatter

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [lazygit](https://github.com/jesseduffield/lazygit) - Inspiration for the UI/UX
- [lazydocker](https://github.com/jesseduffield/lazydocker) - Inspiration for the UI/UX
- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework for Rust
- [awesome-claude-code-subagents](https://github.com/VoltAgent/awesome-claude-code-subagents) -
  Claude Code subagents (MIT)
