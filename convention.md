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

The GUI has the same bulk action under **Tools → Create Comment Files...**. It opens a folder dialog, scans the project for file extensions, then shows an extension picker with file counts. `rs` starts selected; the other extensions start unchecked. After **Create**, `create_comment_files()` in `src/lib.rs` creates starter files only for the selected types. The scan skips `target/`, `make_deb/`, `.comments.d/`, `.git/` and `node_modules/`, and the status bar reports how many files were created.

## Block markers

A comment block starts with a marker on its own line:

```text
@42
Comment for line 42.

@88-91
Comment for lines 88 through 91.
```

`@N` means one line and `@N-M` means an inclusive range. Line numbers are 1-based. Text from a marker up to the next marker belongs to that block; other lines are free text.

## Anchors and alignment

A marker may add source anchors after the line numbers:

```text
@42 [a=fn answer() -> i32 {]
@88-91 [a=fn pair(value: i32) -> i32 {,b=pair + 1]
```

`a` is the trimmed first source line of the block. `b` is the trimmed last source line of a range. Commas, brackets and backslashes inside an anchor are escaped as `\,`, `\[`, `\]` and `\\`. The anchors live on the hidden marker line, so they do not add visible comment lines.

`analyze_alignment()` in `src/lib.rs` parses the anchors and compares their current positions with the marker numbers when a file is opened or reloaded. Missing anchors are derived from the source at the recorded position and written back on save, so older `.cmt` files gain anchors without a separate migration. A block whose anchor moved, has no match, or matches several lines opens **Comment alignment review**. The dialog offers the suggested range, candidate ranges or manual line fields. Accepted fixes are applied to the in-memory blocks and written as new marker numbers and anchors by `rewrite_markers()`. Saving is blocked until the review is applied or skipped.

Anchors work best when the recorded source lines are unique and non-blank. Commentor prefers the match nearest the recorded number; equally near matches are shown as candidates instead of being chosen silently. `View → Reload Source` reruns the same check after the source changes on disk.

Markers stay in the `.cmt` file, but the right pane hides the marker lines without changing their line positions. `hide_markers()` blanks them for display and `restore_markers()` puts them back when the comment text is saved.

Generated starter files include a short header before the first marker. It names the source file, explains the `.comments.d/<source>.cmt` pairing, states the marker and anchor syntax, and links to https://commentor.myridia.com/. Header lines are ordinary free text, so they remain visible in the comment pane.
