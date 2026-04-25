# vikunja-cli

[Vikunja](https://vikunja.io) task management için single-binary Rust CLI. 15 operasyon, nested subcommand yapısı (task list, project create, task comment list, vb.).

## Stack

- **Dil**: Rust 2021
- **Build**: `cargo` (rustc 1.94+)
- **Bağımlılıklar**:
  - `clap` 4.6 (derive) — argparse + nested subcommand
  - `reqwest` 0.13 (rustls + json + query, no native-tls/multipart/cookies/brotli/gzip) — HTTP client
  - `tokio` 1.52 (rt-multi-thread, macros) — async runtime
  - `serde` + `serde_json` — JSON
  - `comfy-table` 7.2 — terminal tables
  - `colored` 3.1 — terminal colors
  - `futures` 0.3 — `try_join_all` for parallel `task get-detail`
  - `anyhow` — error wrapping

## Dil

Türkçe iletişim, İngilizce kod yorumu + commit mesajı.

## Komutlar

```bash
cargo build                                # debug
cargo build --release                      # release (~3.1 MB binary)
cargo run -- task list                     # local çalıştır (env gerekli)
cargo clippy --all-targets -- -D warnings  # lint
cargo fmt --all                            # format
cargo test                                 # test
```

Binary kullanımı:

```bash
export VIKUNJA_API_URL=http://your-vikunja:3456/api/v1
export VIKUNJA_API_TOKEN=tk_xxxxxxxx

vikunja-cli task list --filter "done = false"
vikunja-cli --json task list | jq '.[] | .title'
vikunja-cli task create 6 "Title" --priority 4
vikunja-cli project list
vikunja-cli label list
```

## Proje Yapısı

```
src/
├── main.rs                 clap parser + tokio runtime, dispatch
├── config.rs               env var (VIKUNJA_API_URL/_TOKEN) reader
├── client.rs               reqwest wrapper (Bearer auth, error format)
├── types.rs                serde structs (Task, Project, Label, ProjectNode)
├── output.rs               render() dispatch + table/tree printers
├── util.rs                 truncate, project tree, insert_opt_*, push_opt
└── commands/
    ├── task.rs             list/get/create/update/delete + comment/assign/labels
    ├── project.rs          list/get/create/update
    └── label.rs            list

skills/vikunja-cli/SKILL.md Claude Code skill (workflow wrapper)
.github/workflows/          CI (rustfmt + clippy + test) + Release (multi-target)
```

## Kod Konvansiyonları

- `cargo fmt --all` ile formatla
- `cargo clippy --all-targets -- -D warnings` temiz olmalı
- `anyhow::Result` + `with_context` ile hata zinciri
- HTTP body construction: `util::insert_opt_*` helper'ları kullan, manual `Map.insert` tekrarı yok
- Query string construction: `util::push_opt` helper kullan
- Output dispatch: `output::render(&value, json, |v| print_human(v))` — `if json` tekrarı yok
- Field name string'leri (`"title"`, `"is_archived"` vb.) Vikunja API'nin parçası, transport-layer detay olarak inline kalır

## API Notları

- Vikunja resmi filter syntax: `done = false && priority >= 3 && due_date < now+7d`
- `task list --filter "project_id = X"` — alt-projeleri otomatik OR-expand eder (`util::expand_project_filter`)
- `task get-detail <ids>` — max 25 ID, paralel `try_join_all` ile fetch
- `task labels --ids 1,2,3` — replace semantiği, `--ids ""` ile clear
- Date format: ISO 8601 UTC (`2026-05-15T18:00:00Z`)
- Priority: 1=low, 2=medium, 3=high, 4=urgent, 5=do-now

## Skill

`skills/vikunja-cli/SKILL.md` Claude Code skill'i tetik:
- `/vikunja-cli`, "vikunja task ekle", "açık görevleri getir" vb.
- `--json` ile çağırıp parse eder, kullanıcıya özetler
- AskUserQuestion ile destructive action onayı alır

## Release

Tag push → GitHub Actions multi-target build (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64) + GitHub Release.

```bash
git tag v0.1.0
git push origin v0.1.0
```
