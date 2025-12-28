# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-12-28

### Added

- **Connection Manager**
  - Add, edit, and delete database connections interactively
  - Connection modal dialog with vim-style keybindings
  - Connection search functionality
  - Test connections before saving

- **Schema Browser**
  - Browse database schemas, tables, views, materialized views, and triggers
  - Display detailed metadata: columns, constraints, indexes, foreign keys
  - View/Materialized View definitions with Definition tab
  - Column visibility toggle for schema sub-tabs

- **Data Viewer**
  - View and filter table data with pagination
  - Scrollable data table with keyboard navigation
  - Correct row number display for paginated data

- **Data Export**
  - Export table data to CSV format
  - Export table data to JSON format

- **Query Executor**
  - Write and execute SQL queries (basic functionality)
  - Query history feature with Ctrl+r keybinding

- **Search**
  - Connection search functionality
  - Project search functionality
  - Table search functionality in connections view
  - Unified search modal

- **Project Management**
  - YAML-based configuration file management
  - Project CRUD operations (add/edit/delete)
  - Auto-create sample project on first startup
  - Mode indicator for Projects/Connections visibility

- **User Interface**
  - TUI prototype with ratatui framework
  - Vim-style keybindings for efficient navigation
  - Directional pane navigation with Shift + movement keys
  - Help bar with context-sensitive keybinding hints
  - Unified modal border color (green)

- **Async Operations**
  - Implement async DB operations to prevent UI blocking
  - Background DB worker thread

- **Development Environment**
  - Docker PostgreSQL development environment
  - Task runner configuration
  - GitHub Actions workflows for CI/CD
  - CodeQL security scanning
  - cargo-deny for dependency auditing

### Fixed

- Constrain data table navigation within page boundaries
- Use page-relative index for scrollbar position
- Correct row number display in info bar for paginated data
- Handle schema-qualified names in incoming_references
- Load table details when selecting table in sidebar
- Change query history keybinding from 'h' to Ctrl+r
- Help bar overlapping panel issue

### Changed

- Split large files into smaller focused modules for maintainability
- Migrate from serde_yml to serde_norway for security
- Make ConstraintType match exhaustive for maintainability
- Use unicode-width crate for accurate character width calculation
- Replace Japanese comments with English in config models

### Security

- Add comprehensive security scanning and dependency management
- Migrate from serde_yml to serde_norway to address vulnerabilities
- Introduce cargo-deny for dependency auditing

## [0.0.1] - Initial Development

- First commit with basic project structure
- Initial TUI prototype implementation
- Basic documentation and development setup

[Unreleased]: https://github.com/June3141/lazydb/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/June3141/lazydb/releases/tag/v0.1.0
[0.0.1]: https://github.com/June3141/lazydb/releases/tag/v0.0.1
