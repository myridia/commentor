# AGENTS.md — commentor

## What it is
Commentor shows comments that are stored outside source files. A source file `foo.rs` pairs with `.comments.d/foo.rs.cmt` beside it; the GUI links comment blocks to the source lines they describe.

## Stack
- Rust (edition 2024)
- eframe/egui 0.28.1 (wgpu) + rfd for the file dialog
- TOML settings via `homedir` / `serde` / `toml`, stored at `~/.commentor/config.toml` (same pattern as couchy)
- No async runtime, no database

## Structure
- `src/lib.rs` — file pairing, marker parsing, block mapping and scroll mapping
- `src/main.rs` — eframe app, classic top menu, painting, panes, link strip, open/save
- `src/config.rs` — `AppConfig` (`last_file`, `remember_last_file`, `show_line_numbers`) + TOML load/save
- `src/.comments.d/main.rs.cmt`, `src/.comments.d/lib.rs.cmt`, `src/.comments.d/config.rs.cmt` — comments on commentor's own source, used as the example
- `ask.sh` / `ask.d/` — task menu; each task is a script with `# id:`, `# group:` and `# desc:` headers
- `.github/workflows/` — release pipelines copied from couchy (Linux/Windows, macOS Intel + Silicon, FreeBSD)
- `commentor.svg` — README logo
- GPL-3.0, repo `github/commentor`

## Pair and marker rules
- Pair path: `<dir>/.comments.d/<full-source-filename>.cmt` (append `.cmt`, keep the original extension).
- Marker: `@N` or `@N-M` on its own line, 1-based and inclusive.
- Text from a marker up to the next marker belongs to that block; other lines are free text.
- The source pane is read-only in the viewer; the comment pane is editable and saved back to the `.cmt` file.
- Each binding (the curve in the link strip between a source block and its comment block) gets its own shade from `LINK_COLORS`; the same shade tints both pane highlights, and selection brightens and thickens it.
- Settings: the last successfully opened source path is written to `~/.commentor/config.toml` and reopened on startup unless a path is passed as the first argument; a command-line path wins. `remember_last_file` and `show_line_numbers` are toggled in the Settings menu and saved immediately.

## Build and run
```bash
cargo build --release
cargo run
cargo run -- path/to/foo.rs
```

## Task Runner
`./ask.sh` builds its menu from `ask.d/*.sh`; the user runs it, the agent only reads it. Task scripts can be run directly: `sh ask.d/<name>.sh`.
- 1 Build — `cargo build`
- 2 Run — `cargo run`
- 3 Run example — `cargo run -- src/main.rs`
- 4 Check — `cargo check`
- 5 Clean — `cargo clean`
- 6 Comments — list the `.cmt` files

## Menu
A classic top menu in a `TopBottomPanel`, built with `egui::menu::bar` + `ui.menu_button` (same pattern as ct):

- **File** — Open Source..., Save Comments, Exit
- **View** — Next Link, Previous Link, Reset Scroll
- **Settings** — Remember last file, Show line numbers, Open Settings File, Save Settings Now
- **Help** — Project Page, About

`Ctrl+S` still saves the comment file. The status line lives in a bottom `TopBottomPanel`.

## CI / releases
Three workflows under `.github/workflows/`, adapted from couchy. All trigger on pushes to `main`, any tag, and `workflow_dispatch`; all build jobs are gated by `if: github.repository_owner == 'myridia'`.

- `linux_windows_release.yml` — builds `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-gnu`, renames to `commentor_linux` / `commentor_windows.exe`.
- `macos_release.yml` — `macos-13` builds `x86_64-apple-darwin` → `commentor_macos_intel`; `macos-14` builds `aarch64-apple-darwin` → `commentor_macos_silicon` (couchy's silicon job is mislabeled; fixed here).
- `freebsd_release.yml` — uses `vmactions/freebsd-vm@v1` on `ubuntu-22.04`, builds `x86_64-unknown-freebsd`, renames to `commentor_freebsd`.

Uploads use `svenstaro/upload-release-action@v2` with `GITHUB_TOKEN`, `tag: ${{ github.ref }}`, `overwrite: true` and `file_glob: true`. Couchy's stale `hello_rust_linux` / `hello_rust_windows.exe` upload steps were intentionally omitted; no workflow is run by the agent.

## Conventions
- No comments in code unless asked.
- No git commands; the user commits.
- The user builds, runs and verifies; the agent edits files only.
- Workspace-wide rules live in `~/webs/Agents/AGENTS.md`.
