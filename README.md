<img src="commentor.svg" alt="Commentor logo" width="120">

# commentor

A two-pane viewer for comments that live outside the source file.

`foo.rs` is paired with `.comments.d/foo.rs.cmt` in the same directory. The left pane shows the source, the right pane shows the comment file, and a link strip draws curves between the comment blocks and the source lines they describe.

## Marker syntax

A comment file starts each block with a marker on its own line:

```
@42
Comment for line 42.

@88-91
Comment for lines 88 through 91.
```

`@N` points at one line, `@N-M` at an inclusive range. Line numbers are 1-based. Every line up to the next marker belongs to that block. Lines that do not start with `@` are free text.

## Use

- Open a source file with **File → Open Source...** or pass it as the first argument. The last opened file is saved in `~/.commentor/config.toml` and reopened on the next launch; a command-line path wins.
- Save the comment pane with **File → Save Comments** or `Ctrl+S`.
- Click a highlighted line or a curve to select a link; both panes scroll together through the mapped blocks. **View → Next/Previous Link** walks the bindings.
- Each code↔comment binding has its own tint in both panes and a matching curve in the strip; the selected binding is brighter.
- **Settings** toggles line numbers and last-file memory, and writes them back to the TOML config immediately.
- Edit the right pane and use **File → Save Comments** or `Ctrl+S` to write the `.cmt` file back.

## Example

Commentor's own source is the example. Open `src/main.rs`, `src/lib.rs` or `src/config.rs`; their comment files live in `src/.comments.d/` and describe what each block of code does.

```bash
cargo run -- src/main.rs
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

Each workflow runs on pushes to `main`, on any tag, and via `workflow_dispatch`. All build jobs are gated to the `myridia` account and upload renamed binaries with `svenstaro/upload-release-action@v2`. Couchy's stale `hello_rust_*` upload steps are not copied.
