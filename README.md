<img src="commentor.svg" alt="Commentor logo" width="120">

# commentor

A two-pane viewer for comments that live outside the source file.

`foo.rs` is paired with `.comments.d/foo.rs.cmt` in the same directory. The wider left pane shows the source, the right pane shows the comment file, and a narrow link strip draws curves between the comment blocks and the source lines they describe.

## Marker syntax

A comment file starts each block with a marker on its own line:

```
@42
Comment for line 42.

@88-91
Comment for lines 88 through 91.
```

`@N` points at one line, `@N-M` at an inclusive range. Line numbers are 1-based. Every line up to the next marker belongs to that block. Lines that do not start with `@` are free text.

Markers stay in the `.cmt` file, but the right pane hides the `@` lines while keeping their line positions. The example adds blank lines between sections; those separator lines stay untinted, so each comment is visually separated while the gutter still counts every physical line. Edits are mapped back into the file with the markers preserved.

## Use

- Open a source file with **File → Open Source...** or pass it as the first argument. The last opened file is saved in `~/.commentor/config.toml` and reopened on the next launch; a command-line path wins.
- Save the comment pane with **File → Save Comments** or `Ctrl+S`.
- Use the usual editing commands in either pane: `Ctrl+C` / `Ctrl+X` / `Ctrl+V` / `Ctrl+A`, plus `Ctrl+Z` / `Ctrl+Y`. The **Edit** menu and the right-click context menu offer the same actions. Copy and Select All also work in the read-only source pane; the other Edit actions are disabled there.
- Click a highlighted line or a curve to select a link; both panes scroll together through the mapped blocks. **View → Next/Previous Link** walks the bindings.
- Each code↔comment binding has its own tint in both panes and **two** cubic bezier curves in the strip. Each curve ends in a small filled triangle that points into the pane like a comic speech balloon tail, so the top and bottom of a block are visibly separate. The selected binding is brighter and thicker. Turn off **Settings → Show link block colors** to remove only the filled backgrounds; the pane outlines, curves and tails still show as a single border.
- The source pane has syntax highlighting for Rust, Python, JavaScript, TypeScript, C, C++, Go, Java, JSON, TOML, shell, HTML, CSS and SQL. The language is detected from the file extension; **Settings → Language** can override it or turn highlighting off.
- **Settings → Theme** switches between dark and light egui themes, and **Settings → Colors** picks blue, green, purple, warm, pink or neutral accents. **Settings → Highlight** picks the default, vivid or soft syntax palette.
- **Settings** also controls line numbers, last-file memory, comment wrapping, link block colors, the number of blank lines between comment sections (`Space lines`, default **2**), and separate source/comment font sizes. Every change is written back to the TOML config immediately. Turn on **Wrap comment text** to wrap long comments in the right pane; the source pane always stays unwrapped.
- Edit the right pane and use **File → Save Comments** or `Ctrl+S` to write the `.cmt` file back.

## Example

A small standalone example lives in `example/`:

```text
example/
├── hello.rs
└── .comments.d/
    └── hello.rs.cmt
```

Open `example/hello.rs`; the paired `.cmt` file explains each block of the source. It has long comments, short ones and empty ones, with blank lines between sections so each block is easy to spot in the right pane. This is the file used by the **Run example** task.

```bash
cargo run -- example/hello.rs
```

## Build and run

`./ask.sh` opens the task menu. Its tasks live in `ask.d/`; each is also runnable directly, for example `sh ask.d/run_example.sh`.

```bash
cargo build --release
cargo run
```

The user builds and runs; the agent only edits files.

## Release workflows

GitHub Actions mirror couchy's release pipelines in `.github/workflows/`:

- `linux_windows_release.yml` — Linux (`x86_64-unknown-linux-gnu`) and Windows (`x86_64-pc-windows-gnu`) binaries
- `macos_release.yml` — Intel (`x86_64-apple-darwin`, `macos-13`) and Apple Silicon (`aarch64-apple-darwin`, `macos-14`) binaries
- `freebsd_release.yml` — FreeBSD (`x86_64-unknown-freebsd`) binary, built in a `vmactions/freebsd-vm@v1` VM

Each workflow runs on pushes to `main`, on any tag, and via `workflow_dispatch`. The control workflow resolves the release tag from the `tag` dispatch input, then the pushed tag, then `v0.1.0`, and passes it to each leaf workflow; leaf workflows skip uploads when no tag is available. All build jobs are gated to the `myridia` account and upload renamed binaries with `svenstaro/upload-release-action@v2`. Couchy's stale `hello_rust_*` upload steps are not copied.
