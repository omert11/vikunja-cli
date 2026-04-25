# vikunja-cli

[![CI](https://github.com/omert11/vikunja-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/omert11/vikunja-cli/actions/workflows/ci.yml)
[![Release](https://github.com/omert11/vikunja-cli/actions/workflows/release.yml/badge.svg)](https://github.com/omert11/vikunja-cli/actions/workflows/release.yml)

Single-binary Rust CLI for [Vikunja](https://vikunja.io) — manage tasks, projects, labels, comments, assignees from the terminal.

## Features

- **15 operations** organized as nested subcommands (`task list`, `project create`, `task comment list`, …)
- **Pretty colored tables** by default, `--json` flag for piping
- **Native Vikunja filter syntax** in `task list --filter "..."` — auto-expands `project_id = X` to include all sub-projects
- **Single static binary** (~3 MB, no runtime)
- Same coverage as the Python/FastMCP `vikunja-mcp` server, but standalone

## Install

### Prebuilt binaries (recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/omert11/vikunja-cli/releases/latest):

| Platform | Archive |
|----------|---------|
| Linux x86_64 | `vikunja-cli-x86_64-unknown-linux-gnu.tar.gz` |
| Linux aarch64 | `vikunja-cli-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 (Intel) | `vikunja-cli-x86_64-apple-darwin.tar.gz` |
| macOS aarch64 (Apple Silicon) | `vikunja-cli-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `vikunja-cli-x86_64-pc-windows-msvc.zip` |

Quick install (Linux/macOS):

```bash
TARGET=$(rustc -vV 2>/dev/null | sed -n 's/host: //p')
[ -z "$TARGET" ] && TARGET=$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)
curl -L "https://github.com/omert11/vikunja-cli/releases/latest/download/vikunja-cli-${TARGET}.tar.gz" \
  | tar xz -C /tmp \
  && sudo mv /tmp/vikunja-cli /usr/local/bin/vikunja-cli \
  && vikunja-cli --version
```

### From source

```bash
cargo install --git https://github.com/omert11/vikunja-cli
```

### Build locally

```bash
git clone https://github.com/omert11/vikunja-cli
cd vikunja-cli
cargo build --release
# binary: ./target/release/vikunja-cli
```

## Configuration

Set two environment variables:

```bash
export VIKUNJA_API_URL=http://your-vikunja:3456/api/v1
export VIKUNJA_API_TOKEN=tk_your_personal_token
```

Generate a token from Vikunja → Settings → API tokens.

## Usage

### Tasks

```bash
vikunja-cli task list                                 # all tasks (50 per page)
vikunja-cli task list --filter "done = false"
vikunja-cli task list --filter "priority >= 3 && done = false"
vikunja-cli task list --filter "project_id = 6"       # auto-expands sub-projects
vikunja-cli task list --search "deploy"
vikunja-cli task list --json | jq '.[] | .title'

vikunja-cli task get 42
vikunja-cli task get-detail 1,2,3,4,5

vikunja-cli task create 6 "Refactor auth middleware" \
  --description "Drop the legacy session middleware" \
  --priority 4 \
  --due-date 2026-05-15T18:00:00Z

vikunja-cli task update 42 --done true
vikunja-cli task update 42 --priority 5 --title "URGENT: ..."
vikunja-cli task delete 42

vikunja-cli task comment list 42
vikunja-cli task comment create 42 --comment "Pushed fix in PR #87"

vikunja-cli task assign 42 7
vikunja-cli task unassign 42 7

vikunja-cli task labels 42 --ids 1,3,5
vikunja-cli task labels 42 --ids ""                   # clear all labels
```

### Projects

```bash
vikunja-cli project list                              # hierarchical tree
vikunja-cli project list --search "infra"
vikunja-cli project list --archived true
vikunja-cli project get 6
vikunja-cli project create "Q2 Roadmap" --hex-color 3b82f6 --parent 1
vikunja-cli project update 6 --title "Q2 Roadmap (revised)" --favorite true
```

### Labels

```bash
vikunja-cli label list
vikunja-cli label list --search "bug"
```

## Filter Syntax

`task list --filter` accepts Vikunja's native filter expression:

```
done = false
priority >= 3 && done = false
due_date < now && done = false
project_id = 6 && done = false                # also matches sub-projects
done = false && due_date < now+7d
```

When you use `project_id = X`, sub-projects are auto-expanded into an OR group, so you don't have to enumerate child IDs.

## Output

Default: pretty colored tables (tasks/labels) or trees (projects).

`--json` (global flag): JSON to stdout. Pipe into `jq`:

```bash
vikunja-cli --json task list --filter "done = false" | jq 'map(.title)'
```

## Dependencies

- [`clap`](https://crates.io/crates/clap) — argparse with derive macros
- [`reqwest`](https://crates.io/crates/reqwest) — HTTP client (rustls TLS, no OpenSSL)
- [`tokio`](https://crates.io/crates/tokio) — async runtime
- [`serde` / `serde_json`](https://crates.io/crates/serde) — JSON I/O
- [`anyhow`](https://crates.io/crates/anyhow) — error handling
- [`comfy-table`](https://crates.io/crates/comfy-table) — terminal tables
- [`colored`](https://crates.io/crates/colored) — terminal colors

## License

MIT
