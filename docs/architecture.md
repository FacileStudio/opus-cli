# opus-cli — Architecture

How the `opus` binary is put together: the paths a run can take, the API surface it consumes,
the inline parser, and the shape of the TUI event loop.

## Topology

```
                   ~/.opus.yml  (or --config FILE, or --dev-env + OPUS_* vars)
                          │
                          ▼
  Terminal ──▶ opus binary ──┬──▶ upgrade        git clone + cargo install
                             ├──▶ --quick STR    parse, create one task, exit
                             ├──▶ task ...       list / show / add / done / undone / delete
                             ├──▶ workspace ...  list / current / switch
                             └──▶ (no args)      ratatui TUI
                          │
                          │  reqwest, Authorization: Bearer <api_key>
                          ▼
                   Opus API  (api_url + /api/...)
                          │
                     PostgreSQL
```

No local database, no cache, no daemon. Every non-TUI invocation builds a client, makes its
requests, prints, and calls `std::process::exit`.

## Dispatch order

`main()` is deliberately not a single clap match. In order:

1. **`upgrade` is intercepted first.** `args[1] == "upgrade"` short-circuits before clap even
   runs, which is why `opus upgrade` does not appear in `opus --help`.
2. `dotenv::dotenv()` loads a `.env` from the working directory if one exists.
3. clap parses global flags and the `task` / `workspace` subcommands.
4. `--quick` wins over any subcommand: if present, the task is created and the process exits.
5. `task` and `workspace` each resolve config, build an `OpusClient`, run on a fresh
   `tokio::runtime::Runtime`, and exit with 0 or 1.
6. Anything left over launches the TUI through `tokio_main`.

Each of those four branches resolves configuration independently, with the same shape: use
`OPUS_*` environment variables when `--dev-env` is set, otherwise load the YAML file and
require a non-empty `api_key`. `-w` / `--workspace` overrides the resolved workspace ID last.

## The HTTP client

`OpusClient::new` normalizes the base URL before storing it:

```rust
url.trim().trim_end_matches('/').trim_end_matches("/api").trim_end_matches('/')
```

So `https://opus.example.com`, `https://opus.example.com/api` and
`https://opus.example.com/api/` all collapse to the same root, and every call then appends
its own `/api/...` path. Three unit tests in `src/opus_client.rs` pin that behavior.

Every request sets `Authorization: Bearer <api_key>` by hand. The client is a default
`reqwest::Client` — no explicit timeout is configured.

## Endpoints used

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/api/workspace` | List workspaces |
| `GET` | `/api/workspace/{workspaceId}/members` | Resolve `@assignee` names to users |
| `GET` | `/api/project?workspaceId={id}` | List projects, and the connection test |
| `POST` | `/api/project` | Create a project |
| `GET` | `/api/project/{projectId}` | Project detail |
| `POST` | `/api/task/{projectId}` | Create a task in a project |
| `GET` | `/api/task/tasks/{projectId}` | List a project's tasks |
| `GET` `PUT` `DELETE` | `/api/task/{taskId}` | Task detail, full update, delete |
| `PUT` | `/api/task/status/{taskId}` | Set or toggle status |
| `PUT` | `/api/task/priority/{taskId}` | Set priority |
| `PUT` | `/api/task/assignee/{taskId}` | Set assignee |
| `PUT` | `/api/task/due-date/{taskId}` | Set due date |
| `PUT` | `/api/task/title/{taskId}` | Rename |
| `GET` | `/api/search/?q={q}&type={type}` | Search |
| `GET` `POST` | `/api/comment/{taskId}` | Read and post comments |
| `GET` | `/api/label/workspace/{workspaceId}` | All labels in the workspace |
| `GET` | `/api/label/task/{taskId}` | Labels on one task |
| `POST` | `/api/label/` | Create a label |
| `POST` `DELETE` | `/api/label/{labelId}/task` | Attach and detach a label |
| `GET` `POST` | `/api/column/{projectId}` | Columns, which back the saved filters |
| `PUT` | `/api/column/{columnId}` | Rename a column |
| `GET` | `/api/task-relation/{taskId}` | Relations for a task |
| `POST` | `/api/task-relation/` | Create a relation, including subtasks |
| `DELETE` | `/api/task-relation/{relationId}` | Remove a relation |

Note the overload on `/api/task/{id}`: `POST` takes a **project** ID and creates a task,
while `GET`, `PUT` and `DELETE` take a **task** ID. Attachments are not fetched separately —
they arrive embedded in the task payload.

Statuses are checked per call; a non-2xx response becomes a
`Failed to ...: <status> - <body>` boxed error.

## The inline parser

`src/opus_parser.rs` turns one free-text string into a `ParsedTask` (`title`, `labels`,
`assignees`, `project`, `priority`, `due_date`, `start_date`, `repeat_interval`) using a set
of compiled regexes plus two Aho-Corasick keyword sets for date words and weekday names.

Order matters: labels, priority, assignees and project are extracted first, then the explicit
`start` marker, then the due date — and if no explicit `due` phrase is present the parser
runs its date extraction over the whole remaining string. Everything it consumed is stripped
out by `clean_title`, and what is left becomes the task title.

`create_task_with_magic` then does the API work: resolve `+project` to an ID (falling back to
the default project when the lookup fails), `POST` the task, `ensure_label_exists` and attach
each `*label`, and resolve each `@assignee` to a user before assigning. Labels and assignees
are therefore separate follow-up requests — a task can be created successfully while a label
attachment fails, and the failure is only visible in the debug log.

## TUI runtime

`tokio_main` builds two `Arc<Mutex<...>>` values — the `App` state and the `OpusClient` — then
front-loads the data: connection test, workspaces, tasks with their project map and colors,
all labels, saved filters, and the configured default filter. `run_ui` takes over from there.

`src/tui/events.rs` runs a dedicated OS thread that polls crossterm and pushes `Key` or `Tick`
events down an `mpsc` channel. The loop in `src/ui_loop.rs` receives them, locks the app, and
dispatches by precedence: active modal or picker first (each has its own handler under
`src/tui/modals/` or `src/tui/pickers/`), then `Ctrl` combinations, then the global key table.
Anything that needs the network is awaited inline while the lock is held.

State lives in `src/tui/app/state.rs` as one wide `App` struct with a `show_*_modal` boolean
per overlay. Local edits are pushed onto an undo stack (`undoable_action.rs`) so `Ctrl-Z` and
`Ctrl-Y` can replay them; destructive actions route through `pending_action.rs` and the
confirmation dialog.

## Suite integration

The CLI is a plain REST consumer of the Opus API. It does not use `pool`, `enveloppe` or
Journal, and it does not speak OIDC — authentication is the dashboard-issued API key only.
