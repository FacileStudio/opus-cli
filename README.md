# opus

Keyboard-driven TUI for Opus project management.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/opus-cli/main/install.sh | bash
```

Update: `opus upgrade`

## Setup

Run `opus` to launch the setup wizard, or create `~/.opus.yml`:

```yaml
api_url: "https://opus.example.com"
api_key: "your-api-key"
workspace_id: "your-workspace-id"
```

## Quick Add

Press `a` — write tasks with inline syntax:

```
Buy milk +Groceries *shopping !high due:tomorrow @alice
```

`+project` `*label` `@user` `!priority` `due:date` `start:date`

## Keys

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `j`/`k` | Navigate | `a` | Quick add |
| `g`/`G` | Top / bottom | `e`/`E` | Edit / form edit |
| `h`/`l` | Switch layout | `d`/`D` | Done / delete |
| `p` | Project picker | `s`/`S` | Star / subtask |
| `f` | Filter picker | `Space` | Quick actions |
| `/` | Search | `.` | Comments, attachments |
| `r` | Refresh | `?` | Help |
| `i` | Detail pane | `q`/`Q` | Quit / force quit |

## AI agent integration

`install.sh` auto-registers opus as an AI agent skill for Claude Code and Codex.
After installation, AI coding assistants can use opus commands directly when you ask about tasks, projects, or assignments.

## License

MIT
