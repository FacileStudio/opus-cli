---
name: opus
description: >
  Facile project management CLI. Use when the user asks to create, edit,
  list, or manage tasks, projects, labels, assignments, or API keys.
---

# opus: Facile project management

Binary: `opus`
Config: `~/.opus.yml`

## When to apply

Use when the user mentions tasks, projects, labels, assignments, priorities, due dates, project management, or API keys.
Triggers: "create a task", "add a task", "list tasks", "project", "assign", "priority", "label", "due date", "api key", "api keys"

## Commands

### TUI mode (default)
```
opus                           Launch interactive TUI
```

### Quick add
```
opus --quick "Buy milk +Groceries *shopping !high due:tomorrow"
```

### CLI mode
```
opus task list                 List tasks
  --project <name>            Filter by project
  --label <name>              Filter by label
  --priority <level>          no-priority | low | medium | high | urgent
  --status <slug>             Filter by status
  --overdue                   Overdue only
  --done                      Completed only
  --limit <n>                 Max results (default 50)
  --json                      JSON output
  -q / --quiet                IDs only

opus task show <id>            Show task + comments
  --json

opus task add <text>           Create task with inline syntax
  --json

opus keys list                 List API keys
  --app <name>                Filter by application name
  --json                      JSON output

opus keys create --app <name>  Create an API key
  --public                    Create a public browser key
  --origins <urls>            Allowed origins (comma-separated)
  --quota <n>                 Daily request quota
  --json                      JSON output

opus keys revoke <id>          Revoke an API key
  --yes                       Confirm without prompt
  --json                      JSON output
```

### Inline syntax
- `+project`: assign to project
- `*label`: add label (multiple OK, quote multi-word: `*"high priority"`)
- `@user`: assign user (multiple OK)
- `!priority`: `!low` `!medium` `!high` `!urgent` (or `!1`–`!4`)
- `due:DATE`: natural language: `tomorrow`, `next week`, `Feb 17th at 5pm`
- `start:DATE`: start date
- `every N UNIT`: repeat (e.g., `every 2 weeks`)

### Self-upgrade
```
opus upgrade
```

## Rules
- Prefer `--quick` for non-interactive task creation from agents
- Use `--json` when parsing output programmatically
- Run `opus -h` for exact syntax when unsure
