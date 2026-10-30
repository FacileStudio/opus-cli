# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

The entry below was reconstructed from git history on 2026-08-24, so it records
what shipped rather than what was written down at the time.

## [Unreleased]

## [0.7.1] - 2026-10-30

### Changed

- Split the task write code into smaller modules (`task_patch`, `task_args`, `task_values`) to
  meet the filet size limits. No change in behaviour.

## [0.7.0] - 2026-10-30

### Added

- `opus task update <id>` edits a task. Flags: `--title`, `--description`, `--status`,
  `--priority`, `--project` (moves the task), `--start`, `--due`, `--assignee`, `--add-label`
  and `--remove-label`. Fields not passed stay untouched. `none` clears a date or the
  assignee, and an empty `--description` clears the description.
- `opus task add` takes `--description`, `--status`, `--priority`, `--project`, `--start`,
  `--due`, `--assignee` and a repeatable `--label`. Flags override the quick-add text.

### Changed

- `opus task show --json` prints every field the API returns, including `actorId`,
  `actorName`, `actorImage`, `workspaceId`, `subtaskTotal`, `subtaskDone`, and null `startDate`
  and `dueDate` instead of omitting them.
- `opus task show` prints the assignee name.

## [0.6.1] - 2026-10-27

### Added

- `OPUS_SERVER_URL` environment variable support alongside `OPUS_API_URL` for suite harmonization.

## [0.6.0] - 2026-10-19

### Fixed

- The TUI magic edit modal (`e`) no longer writes the whole inline string into the title. It
  parses the string, so labels, assignees, priority and dates land in their own fields and the
  title stops collecting a copy of the syntax on every save.
- Editing a task no longer attaches labels it already carries, and multi-word labels,
  assignees and project names are bracketed so a save round trip is stable.

### Changed

- `d` moves the selected task to the next column of its project and writes the new
  `status` and `columnId` to the backend, so the web UI moves the card too. It no longer toggles
  a local done flag, which only existed in the TUI.
- `d` follows a configured `quick_actions` binding for that key, and falls back to the column
  cycle when no action claims it.

## [0.5.0] - 2026-10-19

### Added

- `s` works as an alias for `S` when adding a subtask to the selected task.

### Removed

- The TUI filter picker and its `f` binding. Saved filters still load from the
  API and still drive `default_filter`.

## [0.4.0] - 2026-10-19

### Added

- The config file is also read from `$XDG_CONFIG_HOME/opus/config.yml`, usually
  `~/.config/opus/config.yml`, when `~/.opus.yml` is absent. Writes go back to
  whichever file was loaded.
- The TUI caches the last session in `$XDG_CACHE_HOME/opus` and renders it before
  the background refresh finishes, so startup no longer waits on the network. A
  failed load keeps the cached view and says so instead of blanking the list.
- The quick-actions modal always has an action. With none configured, `d` cycles
  the selected task through its project's columns, wrapping at the end.

### Changed

- `w` works as an alias for the workspace picker, alongside `W`.
- `q` and `Q` both quit immediately. The double-`q` shortcut and the quit
  confirmation dialog are gone, so leaving does not need a second keypress.

### Removed

- The `s` star keybind. The backend never implemented starring, so the key did
  nothing and only appeared in the help and usage tables.

## [0.3.0] - 2026-10-13

### Added

- `opus token list`, `opus token create` and `opus token revoke` manage personal
  API tokens against the Go backend's `/auth/api-tokens` routes. A token
  authenticates on every route a login does, which is what a scripted job needs;
  `create` prints the plaintext once and the server only ever stores a hash.
- The credential and the settings resolve from the environment first: `OPUS_TOKEN`
  (falling back to `OPUS_API_KEY`, then `OPUS_KEY`), `OPUS_API_URL`,
  `OPUS_WORKSPACE_ID` and `OPUS_DEFAULT_PROJECT`. Precedence is flag > environment
  > config file > default, so exporting `OPUS_TOKEN` now works without `--dev-env`.

### Changed

- `opus keys` is gone. The workspace API-key endpoints it called were retired with
  the TypeScript API, and the Go backend authenticates personal tokens instead.
- The TUI ships only finished surfaces. The advanced-features modal, the attachment
  viewer, the relations modal, the file picker and the URL modal are removed rather
  than left half-wired, with their `x` (debug pane) and `o` (open URLs) bindings and
  the attachments client methods. Comments and subtasks stay.
- The comments modal is a list and a detail view; its attachment preview and
  `A` binding are gone with the attachment surface.
- The task table is one windowed table with a header row, replacing the duplicated
  per-layout rendering, and dates format the same way in every column.

### Fixed

- `opus token list` shows real created and last-used dates. The API answers those
  fields in camelCase and the model read snake_case, so every token read "never".

## [0.2.1] - 2026-10-13

### Fixed

- The client reads the int64 ids the Opus API emits. Every id and `*_id` field
  now deserializes from a JSON number as well as a string, so workspaces,
  projects, tasks, columns, labels, comments, relations and members parse again
  after the backend moved off CUID2 ids. Comment authors answer without an id or
  email and no longer break a task's comment list.
- Task assignment targets `PUT /task/actor/{id}` instead of the removed
  `/task/assignee/{id}`, so assigning or unassigning a task works.
- Setting a task status no longer sends a `columnId` the endpoint rejects, and
  a general task update now re-reads the task and sends the full payload the
  API expects instead of a partial body, so an update stops failing with a 400
  and stops resetting a task's position.

## [0.2.0] - 2026-09-01

### Added

- API key management command group: `opus keys list`, `opus keys create`, and `opus keys revoke`.
- Support for filtering keys by application, configuring allowed origins, and setting daily request quotas.
- JSON output mode across all `opus keys` subcommands.

## [0.1.0] — 2026-08-10

### Added

- First release. A ratatui TUI for Opus project management, plus a scriptable
  CLI mode for task management that needs no terminal.
- Workspace switching from both the TUI and the CLI, with the workspace list
  discovered from the API.
- `install.sh`, a `self-update` command and prebuilt binaries published on tag.
- AI agent skill registration.
- Documentation harmonized against the suite standard.

### Changed

- Command output goes through the shared `ui` helpers instead of printing
  directly.
- `install.sh` delegates to the `facile` CLI.
- Selection style and list colors are unified across every component, and the
  palette is aligned with the vero theme.

### Fixed

- `~/.opus.yml` is written 0600, and the `OPUS_*` environment variables are
  real overrides rather than defaults.
- Task creation always sends the fields the API requires.
- Truncating a title no longer panics on a multi-byte character.
- Task deserialization matches what the Opus API actually returns.

### Removed

- Device auth. An API key is the only credential.

[Unreleased]: https://github.com/FacileStudio/opus-cli/compare/v0.7.1...HEAD
[0.7.1]: https://github.com/FacileStudio/opus-cli/compare/v0.7.0...v0.7.1
[0.6.1]: https://github.com/FacileStudio/opus-cli/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/FacileStudio/opus-cli/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/FacileStudio/opus-cli/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/FacileStudio/opus-cli/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/FacileStudio/opus-cli/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/FacileStudio/opus-cli/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/FacileStudio/opus-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/FacileStudio/opus-cli/releases/tag/v0.1.0
