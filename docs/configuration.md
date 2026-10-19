# opus-cli — Configuration

Every key the config file accepts and every environment variable the code actually reads.

## The config file

Two locations are read, in order:

1. `~/.opus.yml`, the original path (`dirs::home_dir()` joined with `.opus.yml`).
2. `$XDG_CONFIG_HOME/opus/config.yml`, usually `~/.config/opus/config.yml`.
   `XDG_CONFIG_HOME` is honoured, so a relocated config root works too.

When both files exist, `~/.opus.yml` wins, so an existing setup keeps working after an upgrade.
Override both with `--config FILE` / `-c FILE` on any invocation; an explicit path is used even
when it does not exist, so the error names the file you asked for.

Only a workspace switch writes the file: `opus workspace switch` and the TUI workspace picker
save back to whichever file was loaded, or to `~/.opus.yml` when the session had no config file
behind it.

```yaml
api_url: "https://opus.facile.studio"
api_key: "your-api-token"
workspace_id: "your-workspace-id"
default_project: "Inbox"
default_filter: "Active"
```

| Key | Required | Default | What it does |
|---|---|---|---|
| `api_url` | no | `http://localhost:1337` | Instance root. A trailing `/api` and slashes are stripped |
| `api_key` | no | — | Bearer token. Required unless `OPUS_TOKEN` is set, or the file is missing and `--dev-env` is passed |
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

`action` is one of `project`, `priority`, `label`, `status`, `workspace` or `cycle`; `target` is
the name to apply and is ignored by `cycle`. `label` actions go through the label attachment
endpoint, `cycle` moves the task to the next column in its project (wrapping from the last to the
first), and everything else through a full task update.

When `quick_actions` is missing or empty, the modal offers one built-in action: `d` cycles the
selected task through the project's columns. Outside the modal, `d` runs the action bound to that
key, and falls back to the column cycle when no action claims it.

### Column layouts

`column_layouts` is a list of `{ name, description, columns }`. Each column is
`{ name, column_type, enabled, min_width, max_width, wrap_text, width_percentage, sort }`, and
`column_type` is one of `Title`, `Project`, `Labels`, `DueDate`, `StartDate`, `Priority`,
`Status`, `Assignees`, `Created`, `Updated`. With none configured, four layouts ship built in:
`default`, `minimal`, `project-focused` and `time-management`.

## Session cache

The TUI saves the data it loaded to `$XDG_CACHE_HOME/opus/<api>-<workspace>.json`, usually
`~/.cache/opus/<api>-<workspace>.json`. The file name combines a slug of `api_url` and the
workspace ID, so two instances or two workspaces never share an entry. It holds workspaces,
tasks, project names and colors, labels and saved filters. It never holds the API token.

On startup the TUI renders that snapshot immediately and refreshes from the API in the
background. A manual refresh (`r`) and a workspace switch rewrite the matching entry. If the API
cannot be reached, the cached view stays on screen instead of being replaced by an empty one. A
cache file that cannot be parsed, or that was written by an older cache version, is ignored.
Delete the directory to force a cold start.

## Environment variables

Every variable below overrides the config file, so a shell profile, a CI secret or a `.env`
loaded by `dotenv` from the working directory can carry the whole setup. A `--dev-env` run is
simply one that is allowed to have no config file at all.

| Variable | Replaces | Default with no config file |
|---|---|---|
| `OPUS_TOKEN` | `api_key` | `demo-token` under `--dev-env`, otherwise unset |
| `OPUS_API_KEY` | `api_key`, if `OPUS_TOKEN` is unset | — |
| `OPUS_KEY` | `api_key`, if both above are unset | — |
| `OPUS_API_URL` | `api_url` | `http://localhost:1337` |
| `OPUS_WORKSPACE_ID` | `workspace_id` | `""` |
| `OPUS_DEFAULT_PROJECT` | `default_project` | `Inbox` |
| `OPUS_DEBUG` | — | unset |

The three credential names are checked in order and the first non-empty one wins, because a
credential channel that answers to one spelling only fails in a way that reads as a bad token.
`OPUS_TOKEN` is the documented name and the one the Developer settings page hands out.

`OPUS_DEBUG` is separate from precedence: it is checked in `src/debug.rs` on every log call.
When set, the binary truncates and then appends to `opus_debug.log` **in the current working
directory**. Unset, `debug_log` returns immediately and nothing is written.

## Getting an API token

Create one with the CLI:

```sh
opus token create --name laptop
```

Or in the Opus dashboard under **Settings > Account > Developer**, which lists the same tokens
and prints the value once on creation. Either way the token is what `api_key` or `OPUS_TOKEN`
should hold. There is no `opus login`, no device flow and no OIDC path in the CLI.

## Token storage

The token is stored in plaintext YAML. The CLI does not use the OS keychain, does not encrypt
the file, and does not restrict its mode. On a shared machine, do it yourself:

```sh
chmod 600 ~/.opus.yml
chmod 600 ~/.config/opus/config.yml
```

Better still, keep it out of the file: `export OPUS_TOKEN=...` from a secret store.

The TUI help modal (`?`) shows the token obfuscated as first four and last four characters.

## Precedence

For a single run, values resolve as:

1. An explicit flag, where one exists — `--config`, `-w` / `--workspace`.
2. The environment variable, `OPUS_TOKEN` / `OPUS_API_URL` / `OPUS_WORKSPACE_ID` /
   `OPUS_DEFAULT_PROJECT`.
3. The config file: `--config FILE` if given, else `~/.opus.yml` when it exists, else
   `$XDG_CONFIG_HOME/opus/config.yml` (usually `~/.config/opus/config.yml`).
4. The built-in default — `http://localhost:1337`, `Inbox`, an empty workspace ID.

A missing config file is an error unless `--dev-env` is passed or a credential came from the
environment. A blank `api_key` with nothing in the environment exits 1 with a setup hint.
`-w` is applied last, whichever source the workspace ID came from, and is never persisted.

## Error messages you will actually see

| Symptom | Cause |
|---|---|
| `no config found — create ~/.opus.yml (or ~/.config/opus/config.yml), or set OPUS_TOKEN` | No config file in either location, no `--dev-env`, no credential in the environment |
| `config file not found at <path>` | `--config` pointed at a missing file |
| `no API token configured — create one with \`opus token create\`, set OPUS_TOKEN, or put api_key in ~/.opus.yml or ~/.config/opus/config.yml` | Nothing supplied a credential |
| `GET /api/auth/api-tokens failed (401 Unauthorized)` | The token is wrong or has been revoked |
| `default project '<name>' not found` | `default_project` does not exist in the workspace |
