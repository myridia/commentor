# AGENTS.md — commentor

## What it is
Commentor shows comments that are stored outside source files. A source file `foo.rs` pairs with `.comments.d/foo.rs.cmt` beside it; the GUI links comment blocks to the source lines they describe.

## Stack
- Rust (edition 2024)
- eframe/egui 0.28.1 (wgpu) + rfd for the file dialog
- TOML settings via `homedir` / `serde` / `toml`, stored at `~/.commentor/config.toml` (same pattern as couchy)
- No async runtime, no database

## Structure
- `src/lib.rs` — file pairing, marker parsing, block mapping, scroll mapping, marker hiding/restoring and comment spacing
- `src/main.rs` — eframe app, classic top menu, themes, painting, panes, link strip, open/save
- `src/icon.rs` — hardcoded 32×32 RGBA window icon rasterized from `commentor.svg`, applied through `IconData` + `ViewportBuilder::with_icon` in `main()`
- `src/highlight.rs` — lightweight syntax highlighter for the source pane (language detection + token palettes), used through `TextEdit::layouter`
- `src/config.rs` — `AppConfig` (`last_file`, `remember_last_file`, `show_line_numbers`, `comment_wrap`, `show_link_colors`, `space_lines`, font sizes, theme/colour scheme, highlighting) + TOML load/save
- `example/hello.rs` + `example/.comments.d/hello.rs.cmt` — simple standalone example used for testing
- `src/.comments.d/*.cmt` — comments on commentor's own source; kept as samples, not used by the example task
- `vocabulary.md` — shared names for the GUI parts (panes, gutter, tint band, link strip) and program parts (blocks, markers, bindings, settings); use its terms when discussing changes
- `ask.sh` / `ask.d/` — task menu; each task is a script with `# id:`, `# group:` and `# desc:` headers
- `.github/workflows/` — build pipelines copied from couchy (Linux, Debian, Windows, macOS Intel + Silicon, FreeBSD, MSIX, Pages)
- `commentor.svg` — README logo
- GPL-3.0, repo `github/commentor`

## Pair and marker rules
- Pair path: `<dir>/.comments.d/<full-source-filename>.cmt` (append `.cmt`, keep the original extension).
- Marker: `@N` or `@N-M` on its own line, 1-based and inclusive.
- Text from a marker up to the next marker belongs to that block; other lines are free text.
- Markers are stored in the `.cmt` file but hidden in the right pane; `hide_markers` blanks them without changing line numbers, and `restore_markers` puts them back after edits.
- Blank lines between comment sections stay untinted in the right pane, so sections read clearly while the gutter still numbers every physical line.
- The source pane is read-only in the viewer; the comment pane is editable and saved back to the `.cmt` file.
- The comment pane can wrap long lines when **Wrap comment text** is on. `LineLayout` in `main.rs` turns the wrapped galley into line tops, so gutter numbers, tint bands, click selection and scroll sync still work in logical line numbers; the source pane always stays unwrapped.
- Each binding gets its own shade from `LINK_COLORS`; the same shade tints both pane highlights. The 24px link strip draws **two** cubic bezier curves per binding (first line and last line), each ending in a small filled triangle that points into the pane like a comic speech balloon tail. A faint band between the curves shows the range; selection brightens and thickens everything. `Show link block colors` toggles only the filled backgrounds: the pane outlines, curves and tails stay visible when it is off, with one border line per block.
- The source pane is syntax-highlighted through `TextEdit::layouter`. `highlight.rs` detects a language from the file extension (or the Settings override) and colours keywords, strings, comments, numbers, functions, types and macros. Highlighting can be disabled, and the palette has default/vivid/soft variants.
- Settings: the last successfully opened source path is written to `~/.commentor/config.toml` and reopened on startup unless a path is passed as the first argument; a command-line path wins. All settings (last file, line numbers, comment wrapping, space lines, source/comment font sizes, dark/light theme, accent scheme, highlighting, language and palette) are changed in the Settings menu and saved immediately.
- `space_lines` is the number of visible blank lines between comment sections (default **2**, clamped 1–8). The hidden marker line already provides one, so `normalize_spacing` inserts `space_lines - 1` blank lines before later markers.

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
- 3 Run example — `cargo run -- example/hello.rs`
- 4 Check — `cargo check`
- 5 Clean — `cargo clean`
- 6 Comments — list the `.cmt` files

## Menu
A classic top menu in a `TopBottomPanel`, built with `egui::menu::bar` + `ui.menu_button` (same pattern as ct):

- **File** — Open Source..., Save Comments, Exit
- **Edit** — Undo, Redo, Cut, Copy, Paste, Select All, Delete; the same commands are on each pane's right-click menu. Keyboard shortcuts work directly in the focused `TextEdit`. Copy and Select All also work in the read-only source pane; the other Edit actions are disabled there.
- **View** — Next Link, Previous Link, Reset Scroll
- **Settings** — Remember last file, Show line numbers, Wrap comment text, Show link block colors, Theme (dark/light), Colors (blue/green/purple/warm/pink/neutral), Syntax highlighting, Language (auto plus 14 languages), Highlight palette (default/vivid/soft), Space lines, Source font, Comment font, Open Settings File, Save Settings Now
- **Help** — Project Page, About

`Ctrl+S` still saves the comment file. The status line lives in a bottom `TopBottomPanel`.

## Workflows
- `.github/workflows/control.yml` is the single entry point, copied from couchy. It runs on pushes to `main`, any tag, and `workflow_dispatch`; the `jobs:` block is a one-line-per-build list (comment out a line with `#` to disable it). Its optional `tag` input follows mcclim: use the input, then the pushed tag, then `v0.1.0`.
- Every job is gated by `github.repository_owner == 'myridia'` in `control.yml` only. Leaf workflows are `workflow_call` + `workflow_dispatch` with an optional `tag` input, so they can also be dispatched directly.
- Leaf workflows: `linux.yml`, `debian.yml`, `windows.yml`, `macos.yml`, `macos_intel.yml`, `freebsd.yml`, `msix.yml`, `pages.yml`.
- `FreeBSD` adds the target with `rustup target add x86_64-unknown-freebsd` before building.
- `msix` downloads `commentor.exe` from the `veto8/commentor` `main` release.
- Uploads use `svenstaro/upload-release-action@v2` with `GITHUB_TOKEN`, `tag: ${{ steps.release.outputs.tag }}`, `overwrite: true` and `file_glob: true`. Each leaf workflow resolves that tag from its `tag` input or the pushed ref; if neither exists, it sets `publish=false` and builds without uploading.

## Debian packaging
- `make_deb/src/` — packaging source; `Cargo.toml` and `src` are symlinks to the repo root. `debian/rules` sets `CARGO_TARGET_DIR` to its own `target/`, and `debian/install` copies `target/release/commentor` into `/usr/bin`. The `.deb` lands in `make_deb/` where `debian.yml` expects it. No embedded assets in commentor, so no `pages`/`.cargo` symlinks are needed.

## Project page
- `pages/public/` — GitHub Pages site: `index.html`, `style.css`, `favicon.ico` and `img/` (logo, icon, screenshot). `pages.yml` uploads the site in a `build` job and deploys it in a `deploy` job; `control.yml` calls it.

## Conventions
- No comments in code unless asked.
- No git commands; the user commits.
- The user builds, runs and verifies; the agent edits files only.
- Workspace-wide rules live in `~/webs/Agents/AGENTS.md`.
