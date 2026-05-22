# opus-cli

A fast, keyboard-driven TUI client for Opus project management.

## Features

- Full task management: create, edit, complete, and delete tasks
- Project and workspace navigation
- Quick add with inline syntax for projects, labels, due dates, and priority
- Fuzzy search and filtering
- Customizable column layouts
- API key authentication
- Auto-refresh with configurable interval
- Vim-inspired keybindings

## Installation

Build from source:

```sh
cargo build --release
```

The binary is at `./target/release/opus`.

## Configuration

Generate an API key from your Opus dashboard at Settings > Account > Developer, then add it to `~/.opus.yml`.

Run `opus` with no config file to launch the setup wizard, or create the file manually.
You can also pass a custom path via `--config /path/to/config.yaml` or use environment variables (see `.env.example`).

### Example config.yaml

```yaml
api_url: "https://opus.example.com"
api_key: "your-api-key"
workspace_id: "your-workspace-id"
default_project: "Inbox"
auto_refresh: true
refresh_interval_seconds: 300
```

## Usage

```sh
opus                          # launch the TUI
opus --config /path/to.yaml   # use a custom config
```

### Quick Add

Press `a` to open quick add. Inline syntax:

- `Buy milk +Groceries` -- assign to project
- `Fix bug @alice` -- assign to user
- `Deploy !3` -- set priority
- `Review PR ~friday` -- set due date

### Keyboard Shortcuts

| Key       | Action              |
|-----------|---------------------|
| `j` / `k` | Navigate down / up  |
| `a`       | Quick add task      |
| `e`       | Edit task           |
| `d`       | Delete task         |
| `Enter`   | Toggle complete     |
| `/`       | Search              |
| `p`       | Switch project      |
| `r`       | Refresh             |
| `q`       | Quit                |

## License

MIT
