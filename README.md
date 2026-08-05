# opus-cli

Keyboard-driven terminal client for [Opus](https://github.com/FacileStudio/Opus), the
self-hosted project management app. The `opus` binary opens a full-screen TUI by default and
also exposes scriptable `task` and `workspace` subcommands.

Tasks are written in an inline "magic" syntax — `Review proposal +"Client Work" *urgent
@jane next monday !4` — parsed locally before the task is created.

## What it does

- Full-screen task list with layouts, filters, a detail pane and vim-style navigation
- Quick-add and edit tasks in inline syntax: project, labels, assignees, priority, dates, repeats
- Create, list, show, complete and delete tasks from the shell, with `--json` output
- List, inspect and switch workspaces, persisting the choice to the config file
- Project, filter, label and workspace pickers with fuzzy input
- Task comments, attachments and subtask relations from inside the TUI
- Undo and redo local task edits with `Ctrl-Z` and `Ctrl-Y`
- Self-updates from the GitHub repository with `opus upgrade`

## Stack

| Layer | Tech |
|---|---|
| CLI | Rust 2021, clap 4.5, tokio 1.36, anyhow 1 |
| TUI | ratatui 0.26 (all widgets), crossterm 0.27, fuzzy-matcher 0.3 |
| Transport | reqwest 0.12 (JSON, blocking, multipart), bearer token auth |
| Parsing | regex 1.10, chrono 0.4, chrono-english 0.1, aho-corasick 1 |
| Storage | `~/.opus.yml` via serde_yaml 0.9 |

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/opus-cli/main/install.sh | bash
```

The installer needs `git` and `cargo` on `PATH`. It shallow-clones the repo, runs
`cargo install --path`, and registers the AI agent skill described in
[docs/usage.md](docs/usage.md). Without the installer:

```sh
cargo install --git https://github.com/FacileStudio/opus-cli.git --force
```

Update in place with `opus upgrade`.

## Usage

```sh
opus                                   # TUI
opus --quick "Buy milk +Groceries *shopping !h due tomorrow"
opus task list --project Groceries --limit 10
opus task show <task-id> --json
opus task add "Ship the release +Internal !u next friday"
opus workspace list
opus workspace switch studio
```

Full command reference, inline syntax and TUI keybindings: [docs/usage.md](docs/usage.md).

## Configuration

The CLI reads `~/.opus.yml`, or the file given to `--config`. Environment variables are only
consulted when `--dev-env` is passed.

```yaml
api_url: "https://opus.facile.studio"
api_key: "your-api-key"
workspace_id: "your-workspace-id"
default_project: "Inbox"
```

| Key | What it does |
|---|---|
| `api_url` | Instance root. A trailing `/api` is stripped, then re-added per request |
| `api_key` | Sent as `Authorization: Bearer <key>` on every request |
| `workspace_id` | Scopes project and task queries; `-w` overrides it for one run |
| `default_project` | Project for tasks created without `+project`. Falls back to `Inbox` |

Generate the key in the Opus dashboard under Settings > Account > Developer. Full reference:
[docs/configuration.md](docs/configuration.md).

## Structure

```
src/
  main.rs         clap tree, upgrade shortcut, config resolution, TUI bootstrap
  config.rs       ~/.opus.yml model: layouts, columns, quick actions
  opus_parser.rs  the inline magic-syntax parser
  opus_client/    REST client split by resource: tasks, projects, labels, filters
  cli/            non-interactive subcommands and the human/JSON/quiet printers
  tui/            app state, modals, pickers, ratatui rendering
  ui_loop.rs      the TUI event loop and global key dispatch
integrations/     SKILL.md, registered with Claude Code and Codex by install.sh
```

## Documentation

| Doc | What's in it |
|---|---|
| [Architecture](docs/architecture.md) | Topology, the two modes, endpoints, the parser |
| [Configuration](docs/configuration.md) | Every config key, every environment variable |
| [Development](docs/development.md) | Building, tests, layout of the source tree |
| [Usage](docs/usage.md) | Every command, flag, keybinding and syntax token |

---

Part of the [Facile Suite](https://facile.studio) — self-hosted tools for creative studios
and freelancers. One login, zero cloud dependency.
