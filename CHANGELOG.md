# Changelog

All notable changes to this project are documented here. The format is
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While on
`0.x`, a breaking change bumps the minor.

The entry below was reconstructed from git history on 2026-08-24, so it records
what shipped rather than what was written down at the time.

## [Unreleased]

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

[Unreleased]: https://github.com/FacileStudio/opus-cli/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/FacileStudio/opus-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/FacileStudio/opus-cli/releases/tag/v0.1.0
