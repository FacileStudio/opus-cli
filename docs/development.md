# opus-cli — Development

Building the binary, running it against an Opus instance, where the tests are, and how the
source tree is laid out.

## Prerequisites

- Rust with edition 2021 support and `cargo` on `PATH` (install via [rustup](https://rustup.rs))
- A reachable Opus instance and an API key
- `git`, for `install.sh` and `opus upgrade`

There is no `mise.toml`, no `Makefile`, no `scripts/check.sh`, no `.githooks/` and no CI
workflow. Cargo is the entire toolchain.

## Setup

```sh
git clone https://github.com/FacileStudio/opus-cli.git
cd opus-cli
cargo build
```

Then either write `~/.opus.yml` (see [configuration.md](configuration.md)) or work against
environment variables:

```sh
cp .env.example .env
cargo run -- --dev-env
```

`dotenv` loads `.env` from the working directory, but the `OPUS_*` values are only consulted
when `--dev-env` is passed. Without that flag the config file wins and `.env` is inert.

## Running

```sh
cargo run                                  # TUI against ~/.opus.yml
cargo run -- --dev-env                     # TUI against .env / OPUS_*
cargo run -- --config ./local.yml
cargo run -- task list --json
cargo run -- --quick "Test task +Inbox !h"
cargo run -- --help
```

## Debugging

Set `OPUS_DEBUG` to anything. The binary truncates `opus_debug.log` in the working directory
at startup and appends timestamped lines throughout — config resolution, every request URL,
parser output, API errors. Without the variable, `debug_log` is a no-op and no file is
created.

```sh
OPUS_DEBUG=1 cargo run
tail -f opus_debug.log
```

Inside the TUI, `x` toggles a debug pane fed by `add_debug_message`, which is where most API
failures surface — several code paths log the error there and show a toast rather than
returning it.

## Tests

```sh
cargo test
cargo clippy
cargo fmt --check
```

There is no `tests/` directory; everything is inline `#[cfg(test)]` modules. The meaningful
coverage is:

| File | What it pins |
|---|---|
| `src/opus_parser.rs` | Priority letters and digits, quoted labels, repeat intervals, date forms, `start:` handling, title cleaning |
| `src/opus_client.rs` | `normalize_base_url` accepting a root, an `/api` suffix, and trailing slashes |
| `src/url_utils.rs` | URL extraction from task text |
| `src/tui/utils.rs` | String normalization and comparison |
| `src/terminal_capabilities.rs` | Terminal feature detection |
| `src/tui/app/form_edit_state.rs` | Form edit state transitions |

`proptest` and `lazy_static` are dev-dependencies. The parser is the natural place for
property tests — it is pure, it has the most edge cases, and it is where a regression is
hardest to notice by eye.

## Where things live

| Path | What it holds |
|---|---|
| `src/main.rs` | Upgrade shortcut, clap tree, four config-resolution branches, TUI bootstrap |
| `src/lib.rs` | Re-exports, so the crate is usable as `opus_cli` as well as a binary |
| `src/config.rs` | `OpusConfig`, quick actions, table columns, column layouts |
| `src/first_run.rs` | A setup wizard that nothing currently calls |
| `src/opus_parser.rs` | The inline magic-syntax parser and its tests |
| `src/opus_client.rs` | `OpusClient`, `normalize_base_url`, workspaces, connection test |
| `src/opus_client/` | One module per resource: tasks, projects, labels, users, filters, relations, attachments |
| `src/opus/models.rs` | `Task`, `Project`, `Label`, `User`, `Comment`, `Workspace`, `Priority` |
| `src/cli/` | `task` and `workspace` subcommands plus the human / JSON / quiet printers |
| `src/ui_loop.rs` | The TUI event loop and the global key table |
| `src/tui/app/` | `App` state, split by concern: tasks, projects, labels, filters, undo, workspaces |
| `src/tui/modals/`, `src/tui/pickers/` | Per-overlay input handlers |
| `src/tui/ui/` | ratatui rendering |
| `integrations/SKILL.md` | The AI agent skill the installer registers |

## Adding a command

1. Add the subcommand in `src/cli/task.rs` or `src/cli/workspace.rs`, attaching
   `output::output_args()` if it should support `--json` and `--quiet`.
2. Add its arm to that module's `handle`.
3. Add the endpoint to the right module under `src/opus_client/` if it does not exist.
4. Document it in [usage.md](usage.md) and, if an assistant should know about it, in
   `integrations/SKILL.md`.

New top-level subcommands also need registering in `main.rs`, which currently repeats the
config-resolution block once per branch — reuse an existing block rather than inventing a
fifth variant.

## Known debt

- `src/main.rs` opens with `#![allow(dead_code, unused_variables, unreachable_patterns,
  unused_assignments)]`, so the compiler will not tell you when something stops being used.
- `src/ui_loop.rs` holds the event loop, the key table and most state transitions in one
  file. Changes there are the easiest way to break something unrelated.
- `src/tui/shortcuts.rs` is empty and `src/opus/client.rs` targets a different, unprefixed
  endpoint shape than `src/opus_client/` — neither is on a live path.
- `first_run_wizard()` is written but unreachable, while several error messages still tell
  users to run `opus` for setup.

## Conventions

- No inline comments. Names and structure carry the meaning.
- Remove dead code as you touch it, rather than widening the `allow` list.
- Commit messages are plain imperative sentence case.
