# Vocabulary

Shared names for the parts of the commentor GUI and the code behind them. Use these words when describing a change, so "the box", "the band" and "the gutter" mean exactly one thing.

## Layout at a glance

```
┌──────────────────────────── menu bar ────────────────────────────┐
│ File   Edit   View   Settings   Help                             │
├──────────────────┬───────────┬───────────────────────────────────┤
│  source pane     │ link strip│  comment pane                     │
│ ┌──────┬─────────┤           │ ┌──────┬──────────────────────────┐ │
│ │ gutter│ text   │  curves   │ │ gutter│ text                    │ │
│ │  12  │        │  + tails  │ │  12  │                         │ │
│ └──────┴─────────┘           │ └──────┴──────────────────────────┘ │
├──────────────────┴───────────┴───────────────────────────────────┤
│ status bar                                                       │
└──────────────────────────────────────────────────────────────────┘
```

## GUI parts

| Name | What it is | Where it lives |
| --- | --- | --- |
| **Window** | The native viewport opened by eframe, titled `commentor`, default 1100 × 760. | `main`, `main.rs` |
| **Menu bar** | The classic top panel with the `File`, `Edit`, `View`, `Settings` and `Help` menus. | `update`, `TopBottomPanel::top("menu_bar")` |
| **Edit menu** | Undo, Redo, Cut, Copy, Paste, Select All and Delete for the focused pane. The same commands are on each pane's right-click menu, and the usual keyboard shortcuts work directly in the `TextEdit`. Copy and Select All also work in the read-only source pane; the other Edit actions are disabled there. | `editor_menu_item`, `editor_context_menu`, `run_editor_action` |
| **Clipboard** | The last text copied or cut from either pane. Keyboard paste still uses the system clipboard; menu paste reuses this remembered text. | `App::clipboard`, `egui::Event::Paste` |
| **Status bar** | The bottom panel that shows the last action or error, e.g. `opened …`, `saved …`, `settings saved`. | `update`, `TopBottomPanel::bottom("status_bar")` |
| **Workspace** | The central area between the two bars that holds the panes and the link strip. | `update`, `CentralPanel` |
| **Source pane** | The left, read-only pane. It shows the source file, is syntax-highlighted, and receives 58% of the workspace width. | `left_rect`, `left_text`, `"source_pane"` |
| **Comment pane** | The right, editable pane. It shows the marker-free view of the `.cmt` file and can wrap long lines when the wrap setting is on. | `right_rect`, `right_text`, `"comment_pane"` |
| **Pane gutter** | The narrow vertical strip at the left edge of each pane, behind the line numbers. Its background is `faint_bg_color`. | `paint_pane_background`, `gutter_width` |
| **Line-number column** | The digits painted inside the gutter, one per physical line. Can be hidden in Settings. | `paint_line_numbers`, `show_line_numbers` |
| **Text area** | The part of a pane to the right of the gutter where the `TextEdit` lives. | `left_text`, `right_text` |
| **Tint band** | The translucent coloured rectangle behind one block. Internally called the block band. It starts at the text edge, extends 2 px above the first row and 2 px below the last row, ends 4 px before the pane's right edge, and has a 3 px corner radius. | `paint_scrolled_layers`, `link_tint_color` |
| **Selection** | The block chosen by a click or by `View → Next/Previous Link`. Its tint is stronger and its curves are thicker and brighter. | `selected`, `link_stroke_color`, `link_tint_color` |
| **Link strip** | The 24 px column between the two panes that carries the curves, separated from each pane by a 4 px gap. | `strip_rect`, `paint_links` |
| **Link curve** | A cubic bezier joining the first (or last) row of a source range to the first (or last) row of its comment range. Two curves per block. | `paint_links`, `CubicBezierShape` |
| **Balloon tail** | The small filled triangle at each end of a link curve, pointing into the pane like a speech-balloon tail. | `paint_links`, `tail` |
| **Range band** | The faint four-cornered fill between the two link curves, showing the whole binding. | `paint_links`, `convex_polygon` |

## Data and program parts

| Name | What it is | Where it lives |
| --- | --- | --- |
| **Source file** | The file being annotated, e.g. `example/hello.rs`. Opened read-only and never edited by commentor. | `App::source`, `App::source_path` |
| **Comment file** | The `.cmt` file that stores the comment blocks, e.g. `example/.comments.d/hello.rs.cmt`. Editable in the right pane and written on save. | `App::comments`, `App::comment_path` |
| **Pair path** | The rule that maps `foo.rs` to `.comments.d/foo.rs.cmt`: keep the full filename and append `.cmt`. | `pair_path`, `lib.rs` |
| **Marker line** | A line that is exactly `@N` or `@N-M` (1-based, inclusive). It begins a block and is hidden in the comment pane. | `parse_marker`, `is_marker_line` |
| **Marker** | The range on a marker line, stored internally as 0-based start with an exclusive end. | `parse_marker` |
| **Block** | One comment section: a source range plus the comment-text range that belongs to it. | `Block` in `lib.rs` |
| **Binding / link** | The pairing inside a block between its source lines and its comment lines. Each binding gets its own shade from `LINK_COLORS`; its index is also the thing `Next Link` moves through. | `Block`, `LINK_COLORS` |
| **Comment body** | The free text after a marker, up to the next marker. This is what the user types. | `Block::cmt_start..cmt_end` |
| **Canonical comments** | The real file contents, including marker lines. This is what gets saved. | `App::comments` |
| **Display comments** | The marker-free view shown in the comment pane. Marker lines become empty lines, so physical line numbers stay aligned. | `hide_markers` |
| **Separator lines** | The visible blank lines between comment sections. Controlled by **Space lines** (default 2, clamped 1–8). They stay untinted. | `normalize_spacing`, `space_lines` |
| **Row height** | The pixel height of one text line for a pane's font. Each pane has its own; it converts between line numbers and y coordinates. | `source_row_h`, `comment_row_h` |
| **Line layout** | The per-pane map from logical line numbers to y offsets. Both panes build it from the same galley their `TextEdit` renders, so pixel-rounded row heights cannot drift; tint bands are painted inside each `ScrollArea` using that map. | `LineLayout`, `from_galley`, `top`, `line_at`, `line_position` |
| **Gutter width** | The pane-gutter width in pixels, derived from the number of digits in the largest line number plus 4 px left pad and 6 px breathing room (minimum 18 px). | `gutter_width` |
| **Scroll offset** | The vertical scroll position of one pane, in pixels. | `App::left_offset`, `App::right_offset` |
| **Scroll sync** | Mapping the centre of the pane being scrolled through its bindings into the other pane's line space. `scroll_from` (1 = source, 2 = comments) marks who drove the last move. | `map_line`, `map_scroll`, `App::scroll_from` |
| **Selected block** | The block currently highlighted, stored as an index into `App::blocks`. | `App::selected`, `select_relative` |
| **Settings** | The persistent `AppConfig` values, saved to `~/.commentor/config.toml`. | `config.rs` |
| **Wrap comment text** | The setting that lets the comment pane wrap long lines. The source pane never wraps. Wrapped rows still share one logical line number. | `AppConfig::comment_wrap`, `comment_wrap_w` |
| **Theme** | `Dark` or `Light`, applied to egui visuals. | `AppConfig::theme`, `apply_theme` |
| **Colour scheme / accent** | The accent used for selection, links and hover states: Blue, Green, Purple, Warm, Pink or Neutral. It is not the per-block tint palette. | `AppConfig::color_scheme`, `scheme_accent` |
| **Syntax palette** | The keyword/string/comment/number/function/type/macro colours used by the highlighter: Default, Vivid or Soft. | `highlight::Palette`, `AppConfig::syntax_theme` |
| **Language** | The highlighter grammar selected by extension, or forced in Settings. `Auto detect` falls back to plain text. | `highlight::Language`, `App::active_language` |
| **Highlighter / layouter** | The function that turns one source line into a coloured `LayoutJob`. The comment pane uses a plain no-wrap layouter instead. | `highlight::layout_job`, `TextEdit::layouter` |

## Code map

| File | Responsibility |
| --- | --- |
| `src/main.rs` | eframe app, menu, themes, painting (`paint_pane`, `paint_links`), panes, gutter maths, link selection, open/save. |
| `src/lib.rs` | Pairing, markers, `Block`, line/scroll mapping, marker hiding and restoring, comment spacing. |
| `src/config.rs` | `AppConfig` fields, defaults, TOML load/save at `~/.commentor/config.toml`. |
| `src/highlight.rs` | Language detection, `Palette`, tokeniser that builds the source `LayoutJob`. |
| `example/hello.rs` + `example/.comments.d/hello.rs.cmt` | Standalone 500-line example with 32 comment blocks. |
| `src/.comments.d/*.cmt` | Commentor's own source comments, kept in sync with the code. |

## Phrases to prefer

- **"tint band"** or **"colour box"** — the coloured rectangle behind a block. Say which pane and which block.
- **"gutter"** — the whole narrow column at the left of a pane, including its padding.
- **"text area"** — the region to the right of the gutter, not the whole pane.
- **"marker line"** — the hidden `@N` line; **"marker"** — just the `@N`/`@N-M` token.
- **"binding"** or **"link"** — one source↔comment pair; **"block"** — the comment section that owns it.
- **"selection"** — the active binding, not the text selection inside the editor.
- **"display comments"** vs **"canonical comments"** — what the pane shows versus what is saved.
