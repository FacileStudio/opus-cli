# opus-cli — Configuration

Every key the config file accepts and every environment variable the code actually reads.

## The config file

`~/.opus.yml` by default — `dirs::home_dir()` joined with `.opus.yml`. Override the path with
`--config FILE` / `-c FILE` on any invocation. There is no `XDG_CONFIG_HOME` support.

The CLI never creates this file on its own. `opus workspace switch` is the only command that
writes to it, and it always writes to `~/.opus.yml` regardless of `--config`.

```yaml
api_url: "https://opus.facile.studio"
api_key: "your-api-key"
workspace_id: "your-workspace-id"
default_project: "Inbox"
default_filter: "Active"
```

| Key | Required | Default | What it does |
|---|---|---|---|
| `api_url` | yes | `http://localhost:1337` | Instance root. A trailing `/api` and slashes are stripped |
| `api_key` | yes | — | Bearer token. An empty or missing value exits 1 with a setup hint |
| `workspace_id` | no | `""` | Sent as `workspaceId` on project, label and member queries |
| `default_project` | no | `Inbox` | Project used when the task text has no `+project` |
| `default_filter` | no | — | Name of a saved filter applied on TUI startup |
| `quick_actions` | no | — | Key-to-action bindings for the TUI quick-actions modal |
| `table_columns` | no | built-in set | Columns for the task table |
| `column_layouts` | no | four built-ins | Named column layouts cycled with `h` and `l` |
| `active_layout` | no | `default` | Which layout starts selected |
| `auto_refresh` | no | `true` | Parsed and defaulted, but nothing currently reads it |
| `refresh_interval_seconds` | no | `300` | Parsed and defaulted, but nothing currently reads it |

`auto_refresh` and `refresh_interval_seconds` are accepted by the deserializer and have
accessors on `OpusConfig`, but no call site reads them — the TUI refreshes on `r`, not on a
timer. They are listed here because `config.example.yaml` ships them; do not expect them to
do anything.

### Quick actions

Each entry binds one key to one mutation, applied to the selected task from the quick-actions
modal (`Space`) or directly:

```yaml
quick_actions:
  - key: "u"
    action: "priority"
    target: "urgent"
  - key: "b"
    action: "label"
    target: "bug"
```

`action` is one of `project`, `priority`, `label`, `status` or `workspace`; `target` is the
name to apply. `label` actions go through the label attachment endpoint, everything else
through a full task update.

### Column layouts

`column_layouts` is a list of `{ name, description, columns }`. Each column is
`{ name, column_type, enabled, min_width, max_width, wrap_text, width_percentage, sort }`, and
`column_type` is one of `Title`, `Project`, `Labels`, `DueDate`, `StartDate`, `Priority`,
`Status`, `Assignees`, `Created`, `Updated`. With none configured, four layouts ship built in:
`default`, `minimal`, `project-focused` and `time-management`.

## Environment variables

Environment variables are read **only when `--dev-env` is passed**. Without that flag the
config file is the sole source, and exporting `OPUS_API_URL` changes nothing. `dotenv` loads a
`.env` from the working directory before clap runs, so a `.env` plus `--dev-env` works.

| Variable | Default under `--dev-env` | What it does |
|---|---|---|
| `OPUS_API_URL` | `http://localhost:1337` | Instance root |
| `OPUS_API_KEY` | `demo-token` | Bearer token |
| `OPUS_WORKSPACE_ID` | `""` | Workspace scope |
| `OPUS_DEFAULT_PROJECT` | `Inbox` | Project for tasks without `+project` |
| `OPUS_DEBUG` | unset | If set to anything, enables the debug log |

`OPUS_DEBUG` is the exception: it is checked directly in `src/debug.rs` on every log call, so
it works with or without `--dev-env`. When set, the binary truncates and then appends to
`opus_debug.log` **in the current working directory**. Unset, `debug_log` returns immediately
and nothing is written.

## Getting an API key

Generate it in the Opus dashboard under **Settings > Account > Developer**, then write it into
`~/.opus.yml`. There is no `opus login`, no device flow and no OIDC path in the CLI.

## Token storage

The key is stored in plaintext YAML. The CLI does not use the OS keychain, does not encrypt
the file, and does not restrict its mode. On a shared machine, do it yourself:

```sh
chmod 600 ~/.opus.yml
```

The TUI help modal (`?`) shows the key obfuscated as first four and last four characters.

## Precedence

For a single run, values resolve as:

1. `--dev-env` set — `OPUS_*` variables with the defaults above, config file ignored entirely.
2. Otherwise — `--config FILE` if given, else `~/.opus.yml`. A missing file or an empty
   `api_key` exits 1.
3. `-w` / `--workspace` overrides the workspace ID last, whichever source it came from, and
   is not persisted.

## Error messages you will actually see

| Symptom | Cause |
|---|---|
| `No config found. Run \`opus\` to start setup, or create ~/.opus.yml` | No `~/.opus.yml`. Despite the wording, running `opus` does not open a wizard — write the file yourself |
| `Config file not found at: <path>` | `--config` pointed at a missing file |
| `No API key configured. Generate one from your Opus dashboard: Settings > Account > Developer` | `api_key` missing or blank |
| `Failed to fetch workspaces: 401 - ...` | Wrong or revoked key |
| `default project '<name>' not found` | `default_project` does not exist in the workspace |

A `first_run_wizard()` exists in `src/first_run.rs` but nothing calls it, which is why the
"run `opus` to start setup" hint does not lead anywhere.
