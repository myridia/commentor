# Comment file convention

Commentor keeps comments outside the source file in a sidecar directory named `.comments.d`.

## Pair path

For a source file at `<dir>/<full-source-filename>` the comment file is:

```text
<dir>/.comments.d/<full-source-filename>.cmt
```

The directory is `.comments.d`, not `.comments`. The filename keeps the source extension, then appends `.cmt`.

| Source file | Comment file |
|-------------|--------------|
| `foo.rs` | `.comments.d/foo.rs.cmt` |
| `example/hello.rs` | `example/.comments.d/hello.rs.cmt` |
| `src/lib.rs` | `src/.comments.d/lib.rs.cmt` |

`pair_path()` in `src/lib.rs` is the single implementation of this rule. Opening a source file computes that path and reads it when present. Saving writes to the same path and creates `.comments.d` if needed; a missing comment file simply starts empty.

`ask.d/make_comments.sh` applies the same rule in bulk. It walks the Rust source files under the project root, skips `target/` and `make_deb/target/`, and creates a starter `.cmt` for each source file that does not already have one. Existing comment files are never overwritten.

The GUI has the same bulk action under **Tools → Create Comment Files...**. It opens a folder dialog, uses `create_comment_files()` in `src/lib.rs` from the chosen directory, skips `target/`, `make_deb/`, `.comments.d/` and `.git/`, and reports how many files it created in the status bar.

## Block markers

A comment block starts with a marker on its own line:

```text
@42
Comment for line 42.

@88-91
Comment for lines 88 through 91.
```

`@N` means one line and `@N-M` means an inclusive range. Line numbers are 1-based. Text from a marker up to the next marker belongs to that block; other lines are free text.

Markers stay in the `.cmt` file, but the right pane hides the marker lines without changing their line positions. `hide_markers()` blanks them for display and `restore_markers()` puts them back when the comment text is saved.
