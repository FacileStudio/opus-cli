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
| `-c`, `--config <FILE>` | path | Read config from `FILE` instead of `~/.opus.yml` or `~/.config/opus/config.yml` |
| `--dev-env` | — | Run without requiring a config file, on localhost defaults |
| `--quick <TASK_STRING>` | string | Create one task from inline syntax and exit |
| `-w`, `--workspace <ID>` | string | Override the workspace ID for this run only |
| `--no-color` | — | Disable colored output |
| `-V`, `--version` | — | Print the version from `Cargo.toml` |
| `-h`, `--help` | — | Print help |

`--quick` takes precedence over any subcommand. `-w` is applied after the config or
environment is resolved, and is never written back to disk.

Values resolve as flag > environment variable > config file > built-in default, so `OPUS_TOKEN`
and `OPUS_API_URL` point the CLI at an instance without any config file. `--dev-env` only
relaxes the requirement for one: it lets a run start with no config at all and supplies the
throwaway localhost defaults.

## `opus`

Launch the TUI. When a cached session exists for the API URL and workspace, it renders that
immediately, then refreshes workspaces, tasks with their project map, labels, saved filters and
the configured `default_filter` in the background. Without a cache it shows `Refreshing...` until
the first load finishes.

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
the assignee and labels when present, then the description, then every comment with its author
and timestamp. Under `--json` it prints every field the API returns (description, dates, status,
priority, project, workspace, column, assignee, labels, subtask counts) plus the comments as a
`comments` key. Under `--quiet` it prints just the ID.

### `opus task add`

```sh
opus task add "Ship the release +Internal !u next friday"
opus task add "Fix login bug" --project Opus --description "Repro in #12" \
  --status to-do --priority high --start 2026-11-20 --due 2026-12-01 \
  --assignee Yann --label bug --label auth --json
```

Parses the text (see [Inline syntax](#inline-syntax)), creates the task, then applies any
flags on top. A flag always wins over the same field in the text. `--project` picks the
project by name or ID and fails if it is unknown; without it the task goes to the project in
`+project` or to `default_project`.

| Flag | Value |
| --- | --- |
| `--description` | Free text |
| `--status` | Column slug of the project, for example `to-do`, `done` |
| `--priority` | `no-priority`, `low`, `medium`, `high`, `urgent` |
| `--project` | Project name (case-insensitive) or ID |
| `--start`, `--due` | `YYYY-MM-DD` (UTC midnight), RFC 3339, or natural text like `next friday` |
| `--assignee` | User ID or workspace member name |
| `--label` | Label name, repeatable; created on the task when missing |

Human output is `Created #<number> <title>`. `--json` prints the full task and `-q` prints
the ID.

### `opus task update`

```sh
opus task update 42 --title "New title" --priority urgent
opus task update 42 --due none --assignee none
opus task update 42 --project Mycelium --status done
opus task update 42 --add-label bug --remove-label wip --description ""
```

Changes only the fields you pass. The API replaces the whole task on update, so the CLI
fetches the task, merges your flags into it and sends the result. It needs at least one
field flag.

| Flag | Effect |
| --- | --- |
| `--title` | Set the title |
| `--description` | Set the description; an empty string clears it |
| `--status` | Set the status slug; with `--project` it is the status in the new project |
| `--priority` | `no-priority`, `low`, `medium`, `high`, `urgent` |
| `--project` | Move the task to another project (name or ID) |
| `--start`, `--due` | Set a date; `none` clears it |
| `--assignee` | User ID or member name; `none` unassigns |
| `--add-label`, `--remove-label` | Label name, repeatable |

Human output is `Updated #<number> <title>`. `--json` prints the full updated task and `-q`
prints the ID.

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

Matches the argument against workspace ID, name or slug, case-insensitive for name and slug.
On a match it writes `workspace_id` to the config file it was loaded from (usually `~/.opus.yml`)
and prints a confirmation. On no match it lists the available workspaces on stderr and exits 1.
Under `--dev-env` there is no config object to persist to, so the switch prints its confirmation
without saving.

## `opus token`

Manage personal API tokens. A token authenticates as you on every route a browser session
does, and unlike a login it never idles out, which is what a cron job or an agent needs.

`token` requires a subcommand, and every subcommand authenticates with the credential the run
resolved — so `opus token list` under one token shows that account's tokens.

### `opus token list`

```sh
opus token list
opus token list --json
```

Prints an `ID`, `NAME`, `CREATED`, `LAST USED` table, newest first. Only named tokens are
listed: a browser login is a session in the same table but never appears here, and
`LAST USED` is the day porte last saw the token.

| Flag | Value | What it does |
|---|---|---|
| `--json` | — | Output the raw token array, camelCase fields included |

### `opus token create`

```sh
opus token create --name laptop
opus token create -n ci --json
```

Creates a personal token and prints it **once**, with a reminder that the server keeps only a
hash. The plaintext sits on the line under the confirmation, so `--json` is the better shape
for a script.

| Flag | Value | What it does |
|---|---|---|
| `-n`, `--name <NAME>` | string | Required. The label the token is listed under |
| `--json` | — | Output the created token object, secret included |

### `opus token revoke`

```sh
opus token revoke 42
opus token revoke 42 --yes
```

Revokes one token by ID, which ends it immediately: the next request carrying it answers 401.
Prompts for confirmation when stdin is a terminal and `--yes` is absent.

| Flag | Value | What it does |
|---|---|---|
| `-y`, `--yes` | — | Confirm without prompting |
| `--json` | — | Output `{"id": ..., "revoked": true}` |

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
| `r` | Refresh tasks, projects and filters |
| `h` / `l` | Previous / next column layout |
| `H` / `L` | Cycle the task filter (active, all, and so on) |
| `Esc` | Close the open modal or dialog |
| `q` / `Q` | Quit immediately |
| `Ctrl-Z` | Undo the last local task edit |
| `Ctrl-Y` | Redo |

### Task actions

| Key | Action |
|---|---|
| `a` | Quick-add modal (inline syntax) |
| `e` | Edit the selected task in inline syntax, parsed on save |
| `E` | Edit the selected task in a form |
| `d` | Move the task to the next project column, synced to the API |
| `D` | Delete, via the confirmation dialog |
| `S` / `s` | Add a subtask under the selected task |
| `c` | Comments on the selected task |

### Pickers and modals

| Key | Action |
|---|---|
| `p` | Project picker |
| `w` / `W` | Workspace picker |
| `Space` | Quick-actions modal, driven by `quick_actions` in the config |

With no `quick_actions` configured, the modal contains one default action: `d` cycles the
selected task through the project's columns, wrapping from the last column to the first.

Confirmation dialogs take `Enter` or `y` to confirm, `n` or `Esc` to cancel. The comments modal
switches between the list and the selected comment with `Tab`, moves with `Up` / `Down`, and
submits with `Enter`.

Attachments, task relations, a file picker and a URL viewer were once reachable from an
advanced-features modal. They were never finished, so they are gone: the modal, the keys, and
the client calls behind them.

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
