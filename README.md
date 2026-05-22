# opus

A fast, keyboard-driven TUI for [Opus](https://opus.example.com) project management.

```
┌─ Tasks (Active) ──────────────────────────────────────────┐
│ Title              Project       Due        Priority       │
│ ████████████████████████████████████████████████████████   │ ← selected
│ Fix auth bug       Backend       Today      ⚑ high        │
│ Write docs         Frontend      3d         ⚑ medium      │
│ Deploy v2.1        Infra         Tomorrow   ⚑ urgent      │
└───────────────────────────────────────────────────────────┘
```

## Install

```sh
cargo install --path .
```

## Setup

Run `opus` — the setup wizard walks you through it.

Or create `~/.opus.yml` manually:

```yaml
api_url: "https://opus.example.com"
api_key: "your-api-key"
workspace_id: "your-workspace-id"
default_project: "Inbox"
```

Get your API key from **Settings → Account → Developer** in the Opus dashboard.

## Usage

```sh
opus                                # launch TUI
opus --config /path/to/config.yml   # custom config
opus --quick "Buy milk +Groceries"  # add task without opening TUI
```

## Quick Add

Press `a` to open the quick add modal. Write tasks with inline syntax:

```
Buy milk +Groceries *shopping !high due:tomorrow @alice
```

| Token | Meaning | Example |
|-------|---------|---------|
| `+` | Project | `+Inbox`, `+[Project Name]` |
| `*` | Label | `*bug`, `*[Feature Request]` |
| `@` | Assignee | `@alice`, `@[Alice Smith]` |
| `!` | Priority | `!l` `!m` `!h` `!u` or `!1`–`!4` |
| `due:` | Due date | `due:tomorrow`, `due:friday` |
| `start:` | Start date | `start:monday`, `start:in 3 days` |

Dates understand natural language: `today`, `tomorrow`, `next week`, `friday at 3pm`, `in 2 weeks`, `end of month`.

## Keys

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Move down / up |
| `g` / `G` | Jump to top / bottom |
| `h` / `l` | Previous / next layout |
| `H` / `L` | Cycle task filters |
| `i` | Toggle detail pane |

### Tasks

| Key | Action |
|-----|--------|
| `a` | Quick add |
| `e` | Quick edit |
| `E` | Form editor |
| `d` | Toggle done |
| `D` | Delete |
| `s` | Star / unstar |
| `S` | Add subtask |
| `o` | Open URLs from task |

### Views & Modals

| Key | Action |
|-----|--------|
| `p` | Project picker |
| `f` | Filter picker |
| `/` | Search |
| `Space` | Quick actions |
| `.` | Advanced features (comments, attachments) |
| `?` | Help |
| `r` | Refresh |

### General

| Key | Action |
|-----|--------|
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo |
| `q` | Quit (confirm) |
| `Q` | Quit immediately |

## Config

Full `~/.opus.yml` options:

```yaml
api_url: "https://opus.example.com"
api_key: "your-api-key"
workspace_id: "your-workspace-id"
default_project: "Inbox"
default_filter: "My Tasks"
auto_refresh: true
refresh_interval_seconds: 300

column_layouts:
  - name: "minimal"
    columns:
      - { name: "Title", column_type: Title, enabled: true, min_width: 20 }
      - { name: "Due", column_type: DueDate, enabled: true }
      - { name: "Project", column_type: Project, enabled: true }

quick_actions:
  - { key: "1", action: project, target: "Inbox" }
  - { key: "2", action: label, target: "urgent" }
  - { key: "3", action: priority, target: "3" }
```

Available columns: `Title`, `Project`, `Labels`, `DueDate`, `StartDate`, `Priority`, `Status`, `Assignees`, `Created`, `Updated`.

## License

MIT
