# opus-cli — Usage

The complete reference: global flags, every subcommand, the inline task syntax, and the TUI
keybindings.

## Synopsis

```sh
opus [OPTIONS] [COMMAND]
```

With no command and no `--quick`, `opus` launches the TUI.

### Global options

| Flag | Value | What it does |
|---|---|---|
| `-c`, `--config <FILE>` | path | Read config from `FILE` instead of `~/.opus.yml` |
| `--dev-env` | — | Ignore the config file and read `OPUS_*` environment variables |
| `--quick <TASK_STRING>` | string | Create one task from inline syntax and exit |
| `-w`, `--workspace <ID>` | string | Override the workspace ID for this run only |
| `-V`, `--version` | — | Print the version from `Cargo.toml` |
| `-h`, `--help` | — | Print help |

`--quick` takes precedence over any subcommand. `-w` is applied after the config or
environment is resolved, and is never written back to disk.

## `opus`

Launch the TUI. On startup it tests the connection, then loads workspaces, tasks with their
project map, labels, saved filters and the configured `default_filter`.

```sh
opus
opus --config ~/work/opus.yml
opus -w 01H8XK... 
```

## `opus --quick`

Create a single task from inline syntax, print `Task created: <title> (ID: <id>)`, exit 0. On
failure it prints `Failed to create task: <error>` and exits 1.

```sh
opus --quick "Buy milk +Groceries *shopping !h due tomorrow"
```

This is the path to use from scripts and AI agents — no terminal takeover, no prompts.

## `opus task`

`task` requires a subcommand. `list`, `show` and `add` each accept the output flags:

| Flag | What it does |
|---|---|
| `--json` | Print the raw JSON payload, pretty-printed |
| `-q`, `--quiet` | Print only IDs, one per line |

`--json` wins if both are given. Neither flag exists on `done`, `undone` or `delete`.

### `opus task list`

Fetch every task in the workspace, filter client-side, print a table.

| Flag | Value | Default | What it does |
|---|---|---|---|
| `--project <NAME>` | string | — | Case-insensitive substring match on project name |
| `--label <NAME>` | string | — | Exact label name, case-insensitive |
| `--priority <LEVEL>` | `no-priority`, `low`, `medium`, `high`, `urgent` | — | Exact priority |
| `--status <SLUG>` | string | — | Exact status slug |
| `--overdue` | — | off | Only overdue tasks |
| `--done` | — | off | Only completed tasks |
| `--limit <N>` | integer | `50` | Truncate after N tasks |

```sh
opus task list
opus task list --project "client work" --priority urgent --limit 10
opus task list --overdue --json
opus task list -q | wc -l
```

Human output is a `#`, `TITLE`, `PROJECT`, `PRIORITY`, `STATUS`, `DUE` table sized to the
terminal width, with the title column truncated to fit. Filters are applied in memory after
the full fetch, so they narrow the display, not the request.

### `opus task show`

```sh
opus task show 01H8XK...
opus task show 01H8XK... --json
```

Prints the task header, then Project, Status, Priority, Due, Start, Created and Updated, then
assignees and labels when present, then the description, then every comment with its author
and timestamp. Under `--json` the comments are merged into the task object as a `comments`
key. Under `--quiet` it prints just the ID.

### `opus task add`

```sh
opus task add "Ship the release +Internal !u next friday"
opus task add "Fix login bug *bug @jane" --json
```

Resolves `default_project` to an ID first and fails with `default project '<name>' not found`
if it does not exist. Then parses the text (see [Inline syntax](#inline-syntax)), creates the
task, and attaches labels and assignees in follow-up requests. Human output is
`Created #<number> <title>`.

### `opus task done` / `opus task undone`

Set the status of one or more tasks to `done` or `to-do`. Both take one or more IDs.

```sh
opus task done 01H8XK...
opus task done 01H8XK... 01H8XM... 01H8XN...
opus task undone 01H8XK...
```

Progress lines go to **stderr**, one per ID: `  <title> → done`, or `  <id> failed: <error>`.
A failure on one ID does not stop the rest and does not change the exit code.

### `opus task delete`

```sh
opus task delete 01H8XK... 01H8XM...
```

Deletes each ID with no confirmation prompt. Prints `  <id> deleted` or `  <id> failed:
<error>` to stderr, same non-aborting behavior as above.

## `opus workspace`

`workspace` requires a subcommand.

### `opus workspace list`

```sh
opus workspace list
opus workspace list --json
opus workspace list -q
```

Prints a `NAME` / `ID` table with the current workspace marked by a green `*`. `--json` dumps
the raw workspace array; `--quiet` prints IDs only.

### `opus workspace current`

```sh
opus workspace current
```

Prints `<name> (<id>)`. If the configured ID matches no known workspace it prints the raw ID;
if none is configured it prints `No workspace configured`.

### `opus workspace switch`

```sh
opus workspace switch "Client Work"
opus workspace switch studio
opus workspace switch 01H8XK...
```

Matches the argument against workspace ID, name or slug — case-insensitive for name and slug.
On a match it writes `workspace_id` to `~/.opus.yml` and prints a confirmation. On no match it
lists the available workspaces on stderr and exits 1. Under `--dev-env` there is no config
object to persist to, so the switch prints its confirmation without saving.

## `opus upgrade`

```sh
opus upgrade
```

Clones the repository into a temporary directory and runs `cargo install --path --force`, so
`git` and `cargo` must be on `PATH`. This is handled before clap parses anything, so it takes
no flags, reads no config, and does not appear in `opus --help`.

## Inline syntax

The same parser backs `--quick`, `opus task add`, the TUI quick-add modal (`a`) and the TUI
magic edit modal (`e`).

| Token | Meaning |
|---|---|
| `+project` | Target project. Falls back to `default_project` when the name does not resolve |
| `*label` | Add a label. Repeatable |
| `@user` | Assign a user by name. Repeatable |
| `!priority` | `!n` none, `!l` low, `!m` medium, `!h` high, `!u` urgent — or `!1`–`!4` |
| `due <date>` | Explicit due date |
| `start:<date>` | Explicit start date. Also accepts `start <date>` |
| `every [N] <unit>` | Repeat interval, e.g. `every week`, `every 2 days` |

Multi-word values are quoted or bracketed: `*"high priority"`, `@'john doe'`, `+[Client Work]`.

**Priority takes a letter or a digit, not a word.** `!h` sets high; `!high` matches nothing
and stays in the title. This contradicts older examples that used `!high` — the regex is
`!([1-4]|[nlmhu])`.

Dates are recognized anywhere in the text even without a `due` marker. Supported forms
include `today`, `tomorrow`, `yesterday`, `next/this/last week|month|year`,
`this weekend`, `next weekend`, `end of week`, `end of month`, `end of year`, weekday names
and their three-letter forms, `in 3 days`, ordinals like `15th`, month names like
`Feb 17th`, and an `at 5pm` / `at 10:30am` time suffix. `start:` additionally accepts the
shorthands `eow` and `eom`. Everything the parser consumes is stripped from the title.

```sh
opus --quick 'Review proposal *urgent *"high priority" @jane +"Client Work" next monday at 10am !4 every week'
```

That creates a task titled `Review proposal`, in `Client Work`, labeled `urgent` and
`high priority`, assigned to Jane, priority urgent, due next Monday at 10:00, repeating
weekly.

## TUI keybindings

### Global

| Key | Action |
|---|---|
| `?` | Help and keybinds modal |
| `j` / `k` / `Down` / `Up` | Move the selection |
| `g` / `G` | Jump to top / bottom |
| `i` | Toggle the info (detail) pane |
| `x` | Toggle the debug pane |
| `r` | Refresh tasks, projects and filters |
| `h` / `l` | Previous / next column layout |
| `H` / `L` | Cycle the task filter (active, all, and so on) |
| `Esc` | Close the open modal or dialog |
| `q` | Quit with confirmation, or close the advanced modal |
| `qq` | Quit immediately (two `q` within one second) |
| `Q` | Quit immediately, no confirmation |
| `Ctrl-Z` | Undo the last local task edit |
| `Ctrl-Y` | Redo |

### Task actions

| Key | Action |
|---|---|
| `a` | Quick-add modal (inline syntax) |
| `e` | Edit the selected task in inline syntax |
| `E` | Edit the selected task in a form |
| `d` | Toggle completion, synced to the API |
| `D` | Delete, via the confirmation dialog |
| `s` | Star / unstar |
| `S` | Add a subtask under the selected task |
| `o` | Open URLs found in the selected task |

### Pickers and modals

| Key | Action |
|---|---|
| `p` | Project picker |
| `f` | Filter picker |
| `W` | Workspace picker |
| `Space` | Quick-actions modal, driven by `quick_actions` in the config |
| `.` | Advanced features modal |

From the advanced modal: `a` attachments, `c` comments, `r` task relations (not implemented
yet — it shows a toast). Confirmation dialogs take `Enter` or `y` to confirm, `n` or `Esc` to
cancel.

There is no `/` search binding in the TUI, despite what older documentation claimed.

## AI agent skill

`install.sh` registers `integrations/SKILL.md` with whichever assistants it finds on `PATH`:

- `claude` present — copies the file to `~/.claude/skills/opus/SKILL.md` and injects its
  contents into `~/.claude/CLAUDE.md`
- `codex` present — injects the same contents into `~/.codex/AGENTS.md`

Injection is idempotent: the block is fenced by `<!-- opus:start -->` and `<!-- opus:end -->`
markers, and a rerun strips the old block before appending the new one. Neither file is
created unless the corresponding binary exists. To opt out, install with `cargo install --git`
instead of the script; to remove it later, delete the marked block and the skill directory.
