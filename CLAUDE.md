# opus-cli

Terminal client for [Opus](https://github.com/FacileStudio/opus), the Facile self-hosted
project management app. Provides a full-screen interactive TUI (ratatui) and scriptable
CLI subcommands for tasks, workspaces, and API keys.

## Tech stack

- Language: Rust (edition 2021), async on Tokio
- HTTP: reqwest (JSON, multipart, blocking)
- TUI: ratatui, crossterm, fuzzy-matcher
- CLI parsing: clap (builder pattern)
- Config: YAML (`~/.opus.yml`) via serde_yaml
- Date parsing: chrono, chrono-english, regex, aho-corasick

## Commands

```sh
cargo build              # debug build
cargo build --release    # optimized (LTO + strip)
cargo test               # run all unit tests
cargo run                # launch interactive TUI
cargo run -- task list   # list tasks
cargo run -- keys list   # list API keys
```

## Project structure

```
src/
  main.rs         CLI entry point, clap definition, subcommand handlers, TUI launcher
  config.rs       Loads ~/.opus.yml (server URL, API key, workspace ID)
  opus/
    models.rs     Domain models (Task, Workspace, Project, Key, User, Label)
    client.rs     Legacy helper client
  opus_client.rs  REST client root and base URL normalization
  opus_client/    Client methods by domain (tasks, projects, keys, labels, users, filters)
  cli/            CLI command modules (task, workspace, keys, output formatting)
  tui/            TUI state machine, events, modals, pickers, ratatui rendering
  opus_parser.rs  Magic inline task syntax parser
  ui.rs           Terminal output styling (ANSI colors, status markers)
integrations/
  SKILL.md        AI agent skill definition
```

## Key features

- Interactive TUI for task and project management
- Quick task creation via inline syntax (`--quick` or `task add`)
- Workspace management and switching
- API key management (`keys list`, `keys create`, `keys revoke`)
- JSON output support across scriptable commands

## Conventions

- No inline comments in code
- Remove dead code rather than allowing it
- Plain language documentation without em dashes
