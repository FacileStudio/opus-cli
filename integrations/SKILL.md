---
name: opus
description: >
  Facile project management CLI. Use when the user asks to create, edit,
  list, or manage tasks, projects, labels, assignments, or API tokens.
---

# opus: Facile project management

Binary: `opus`
Config: `~/.opus.yml` or `~/.config/opus/config.yml`

## When to apply

Use when the user mentions tasks, projects, labels, assignments, priorities, due dates, project management, or API tokens.
Triggers: "create a task", "add a task", "list tasks", "project", "assign", "priority", "label", "due date", "api key", "api token"

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

opus task add <text>           Create task with inline syntax, flags override it
  --description --status --priority --project --start --due --assignee
  --label <name> (repeatable)
  --json / -q

opus task update <id>          Change only the fields passed
  --title --description --status --priority --project --start --due --assignee
  --add-label <name> --remove-label <name> (repeatable)
  --start/--due/--assignee accept "none" to clear; --description "" clears
  --json / -q

opus task delete <id>...       Delete tasks

opus token list                List personal API tokens
  --json                      JSON output

opus token create -n <name>    Create a personal API token, printed once
  --json                      JSON output

opus token revoke <id>         Revoke a personal API token
  --yes                       Confirm without prompt
  --json                      JSON output
```

### Inline syntax
- `+project`: assign to project
- `*label`: add label (multiple OK, quote multi-word: `*"high priority"`)
- `@user`: assign user (multiple OK)
- `!priority`: `!n` none, `!l` low, `!m` medium, `!h` high, `!u` urgent (also `!1`–`!4`).
  Letters or digits, never words: `!high` stays in the title
- `due DATE`: natural language: `due tomorrow`, `next week`, `Feb 17th at 5pm`. A date is also
  recognized without the marker
- `start:DATE` or `start DATE`: start date. `start:` accepts `eow` and `eom`
- `every N UNIT`: repeat (e.g., `every 2 weeks`)

## Rules
- Prefer `--quick` for non-interactive task creation from agents
- Use `--json` when parsing output programmatically
- Run `opus -h` for exact syntax when unsure
- The credential comes from `OPUS_TOKEN` (or `api_key` in `~/.opus.yml` or `~/.config/opus/config.yml`); create one with
  `opus token create -n <name>`
