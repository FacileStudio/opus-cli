# opus-cli — CLI Mode Roadmap

Keyboard-driven TUI is great for humans. CLI mode makes opus scriptable for agents, pipes, cron, and shell workflows.

## Design Principles

- `opus` with no subcommand = TUI (backwards compatible)
- Every command supports `--json` (structured output) and `-q` (IDs only)
- Exit codes: 0 success, 1 error, 2 not found
- No interactive prompts in CLI mode — fully scriptable
- Magic syntax (`+project *label @user !priority due:date`) works in `task add`

---

## Phase 1 — Core Task CRUD *(current)*

> Covers 80% of use cases. Makes opus immediately useful from scripts and agents.

| Command | Description |
|---|---|
| `opus task list` | List tasks with filters |
| `opus task show <id>` | Show task details + comments |
| `opus task add "text"` | Create task with magic syntax |

### Filters for `task list`

```
--project <name>       filter by project name
--label <name>         filter by label name
--priority <level>     low, medium, high, urgent
--status <slug>        filter by status slug
--overdue              only overdue tasks
--done                 only completed tasks
--limit <n>            max results (default 50)
```

### Output modes

```sh
opus task list --project Backend --priority high
# ID       Title              Priority  Due         Status
# abc123   Fix auth flow      high      2026-05-30  in-progress

opus task list --json
# [{"id":"abc123","title":"Fix auth flow",...}]

opus task list -q
# abc123
# def456

# piping
opus task list --overdue -q | xargs -I{} opus task done {}
```

---

## Phase 2 — Full Task Operations

| Command | Description |
|---|---|
| `opus task edit <id> --title "..." --priority high` | Update task fields |
| `opus task done <id>` | Toggle task completion |
| `opus task delete <id>` | Delete task |
| `opus task comment <id> "message"` | Add comment |
| `opus task comments <id>` | List comments |
| `opus task search "query"` | Full-text search |

---

## Phase 3 — Reference Data

| Command | Description |
|---|---|
| `opus project list` | List all projects |
| `opus project show <id>` | Project details |
| `opus project create "name"` | Create project |
| `opus label list` | List all labels |
| `opus label create "name" --color "#hex"` | Create label |
| `opus user list` | List workspace members |

---

## Phase 4 — Filters & Discovery

| Command | Description |
|---|---|
| `opus filter list` | List saved filters |
| `opus filter apply <id>` | List tasks matching a saved filter |
| `opus task search "query"` | Search across tasks |

---

## Phase 5 — Agent Ergonomics

| Feature | Description |
|---|---|
| Stdin support | `echo "long description" \| opus task add --stdin` |
| Batch operations | `opus task done --ids id1,id2,id3` |
| Watch mode | `opus task list --watch` (poll and refresh) |
| Templated output | `opus task list --format "{{id}} {{title}}"` |
| Shell completions | `opus completions zsh > _opus` |

---

## Architecture

```
src/
  cli/
    mod.rs            # subcommand definitions + routing
    output.rs         # human / json / quiet formatting
    task.rs           # task subcommand handlers
    project.rs        # project handlers (phase 3)
    label.rs          # label handlers (phase 3)
    user.rs           # user handlers (phase 3)
    filter.rs         # filter handlers (phase 4)
```

Global flags `--json` and `-q` are defined once and propagated to all subcommands via `clap::Arg::global(true)`.

Each handler is a simple async function that takes `&OpusClient` + `&ArgMatches`, does the work, and prints formatted output. No TUI dependencies.
