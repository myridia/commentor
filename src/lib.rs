use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentKind {
    Drifted,
    Ambiguous,
    Missing,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub src_start: usize,
    pub src_end: usize,
    pub cmt_start: usize,
    pub cmt_end: usize,
    pub anchor: Option<String>,
    pub end_anchor: Option<String>,
    pub is_range: bool,
}

#[derive(Clone, Debug)]
pub struct AlignmentIssue {
    pub block: usize,
    pub kind: AlignmentKind,
    pub message: String,
    pub current: (usize, usize),
    pub candidates: Vec<(usize, usize)>,
}

pub struct AlignmentReview {
    pub blocks: Vec<Block>,
    pub issues: Vec<AlignmentIssue>,
}

pub fn pair_path(path: &Path) -> PathBuf {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    dir.join(".comments.d").join(format!("{name}.cmt"))
}

fn collect_project_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();
            if file_type.is_dir() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if matches!(
                    name.as_ref(),
                    "target" | "make_deb" | ".comments.d" | ".git" | "node_modules"
                ) {
                    continue;
                }
                dirs.push(path);
            } else if file_type.is_file() {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn extension_of(path: &Path) -> &str {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or("(none)")
}

pub fn scan_extensions(root: &Path) -> std::io::Result<Vec<(String, usize)>> {
    let mut counts = std::collections::BTreeMap::new();
    for path in collect_project_files(root)? {
        *counts
            .entry(extension_of(&path).to_string())
            .or_insert(0) += 1;
    }
    Ok(counts.into_iter().collect())
}

pub fn create_comment_files(root: &Path, extensions: &[String]) -> std::io::Result<usize> {
    let selected: Vec<&str> = extensions.iter().map(|value| value.as_str()).collect();
    let mut created = 0;
    for path in collect_project_files(root)? {
        let extension = extension_of(&path);
        if !selected.iter().any(|value| *value == extension) {
            continue;
        }
        let pair = pair_path(&path);
        if pair.exists() {
            continue;
        }
        if let Some(parent) = pair.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let relative = path.strip_prefix(root).unwrap_or(&path);
        std::fs::write(
            pair,
            format!(
                "# Commentor comments for {}\n# This file pairs with the source file under .comments.d/<source>.cmt.\n# Marker: @N or @N-M, optionally followed by [a=start anchor,b=end anchor].\n# Get Commentor: https://commentor.myridia.com/\n\n@1\nAdd a comment for {}.\n",
                relative.display(),
                relative.display()
            ),
        )?;
        created += 1;
    }
    Ok(created)
}

fn escape_anchor(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            ',' => escaped.push_str("\\c"),
            '[' => escaped.push_str("\\l"),
            ']' => escaped.push_str("\\r"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn unescape_anchor(value: &str) -> String {
    let mut plain = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            plain.push(ch);
            continue;
        }
        match chars.next() {
            Some('c') => plain.push(','),
            Some('l') => plain.push('['),
            Some('r') => plain.push(']'),
            Some('\\') => plain.push('\\'),
            Some(other) => {
                plain.push('\\');
                plain.push(other);
            }
            None => plain.push('\\'),
        }
    }
    plain
}

fn split_escaped(value: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            current.push('\\');
            if let Some(next) = chars.next() {
                current.push(next);
            }
        } else if ch == ',' {
            parts.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    parts.push(current);
    parts
}

fn parse_range(text: &str) -> Option<(usize, usize)> {
    let mut parts = text.splitn(2, '-');
    let first: usize = parts.next()?.parse().ok()?;
    let second: usize = match parts.next() {
        Some(value) => value.parse().ok()?,
        None => first,
    };
    if first == 0 || second < first {
        return None;
    }
    Some((first - 1, second))
}

fn parse_marker_anchors(line: &str) -> Option<(Option<String>, Option<String>)> {
    let rest = line.trim().strip_prefix('@')?;
    let split = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let metadata = rest[split..].trim();
    if metadata.is_empty() {
        return Some((None, None));
    }
    if !(metadata.starts_with('[') && metadata.ends_with(']')) {
        return None;
    }
    let inner = &metadata[1..metadata.len() - 1];
    let mut start = None;
    let mut end = None;
    for part in split_escaped(inner) {
        if let Some(value) = part.strip_prefix("a=") {
            start = Some(unescape_anchor(value));
        } else if let Some(value) = part.strip_prefix("b=") {
            end = Some(unescape_anchor(value));
        } else {
            return None;
        }
    }
    Some((start, end))
}

pub fn parse_marker(line: &str) -> Option<(usize, usize)> {
    let rest = line.trim().strip_prefix('@')?;
    let split = rest.find(char::is_whitespace).unwrap_or(rest.len());
    parse_range(&rest[..split])
}

fn source_lines(source: &str) -> Vec<&str> {
    source.lines().collect()
}

fn clean_anchor(value: Option<String>) -> Option<String> {
    value
        .map(|anchor| anchor.trim().to_string())
        .filter(|anchor| !anchor.is_empty())
}

fn anchor_text(line: Option<&str>) -> Option<String> {
    clean_anchor(line.map(|value| value.to_string()))
}

fn display_range(range: (usize, usize)) -> (usize, usize) {
    (range.0 + 1, range.1.max(range.0 + 1))
}

pub fn parse_blocks(text: &str, source: &str) -> Vec<Block> {
    let lines: Vec<&str> = text.lines().collect();
    let rows = source_lines(source);
    let mut markers: Vec<(usize, usize, usize, bool, Option<String>, Option<String>)> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if let Some((start, end)) = parse_marker(line) {
            let src_start = start.min(rows.len());
            let src_end = end.min(rows.len()).max(src_start);
            let is_range = end > start + 1;
            let (mut anchor, mut end_anchor) =
                parse_marker_anchors(line).unwrap_or((None, None));
            anchor = clean_anchor(anchor).or_else(|| anchor_text(rows.get(src_start)));
            end_anchor = clean_anchor(end_anchor).or_else(|| {
                if is_range {
                    anchor_text(rows.get(src_end.saturating_sub(1)))
                } else {
                    None
                }
            });
            markers.push((index, src_start, src_end, is_range, anchor, end_anchor));
        }
    }
    let mut blocks = Vec::new();
    for (position, (cmt_start, src_start, src_end, is_range, anchor, end_anchor)) in
        markers.iter().enumerate()
    {
        let cmt_end = if position + 1 < markers.len() {
            markers[position + 1].0
        } else {
            lines.len()
        };
        blocks.push(Block {
            src_start: *src_start,
            src_end: *src_end,
            cmt_start: *cmt_start,
            cmt_end,
            anchor: anchor.clone(),
            end_anchor: end_anchor.clone(),
            is_range: *is_range,
        });
    }
    blocks
}

fn anchor_candidates(rows: &[&str], anchor: &str, preferred: usize) -> Vec<usize> {
    let mut found: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == anchor)
        .map(|(index, _)| index)
        .collect();
    found.sort_by_key(|index| (index.abs_diff(preferred), *index));
    found
}

fn candidate_ranges(rows: &[&str], block: &Block) -> Vec<(usize, usize)> {
    let starts = match &block.anchor {
        Some(anchor) => anchor_candidates(rows, anchor, block.src_start),
        None => Vec::new(),
    };
    if starts.is_empty() {
        return Vec::new();
    }
    let span = block.src_end.saturating_sub(block.src_start).max(1);
    let mut ranges = Vec::new();
    for start in starts.into_iter().take(8) {
        if let Some(end_anchor) = &block.end_anchor {
            let preferred = block.src_end.saturating_sub(1);
            for end in anchor_candidates(rows, end_anchor, preferred)
                .into_iter()
                .take(8)
            {
                if end >= start {
                    ranges.push((start, end + 1));
                }
            }
        } else {
            let end = (start + span).min(rows.len());
            if end > start {
                ranges.push((start, end));
            }
        }
    }
    ranges.sort();
    ranges.dedup();
    ranges
}

pub fn analyze_alignment(source: &str, comments: &str) -> AlignmentReview {
    let blocks = parse_blocks(comments, source);
    let rows = source_lines(source);
    let mut issues = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        let current = display_range((block.src_start, block.src_end));
        let candidates = candidate_ranges(&rows, block);
        if candidates.is_empty() {
            issues.push(AlignmentIssue {
                block: index,
                kind: AlignmentKind::Missing,
                message: "No anchor text matched the source. Choose the correct lines.".to_string(),
                current,
                candidates: Vec::new(),
            });
            continue;
        }
        let score = |range: &(usize, usize)| {
            range.0.abs_diff(block.src_start) + range.1.abs_diff(block.src_end)
        };
        let best_score = candidates.iter().map(score).min().unwrap_or(usize::MAX);
        let best: Vec<(usize, usize)> = candidates
            .into_iter()
            .filter(|range| score(range) == best_score)
            .take(8)
            .collect();
        if best.len() == 1 && best[0] != (block.src_start, block.src_end) {
            issues.push(AlignmentIssue {
                block: index,
                kind: AlignmentKind::Drifted,
                message: format!(
                    "Anchor moved from lines {}-{} to lines {}-{}.",
                    current.0,
                    current.1,
                    display_range(best[0]).0,
                    display_range(best[0]).1
                ),
                current,
                candidates: best.into_iter().map(display_range).collect(),
            });
        } else if best.len() > 1 {
            issues.push(AlignmentIssue {
                block: index,
                kind: AlignmentKind::Ambiguous,
                message: "Anchor matches several source lines. Pick the right range.".to_string(),
                current,
                candidates: best.into_iter().map(display_range).collect(),
            });
        }
    }
    AlignmentReview { blocks, issues }
}

pub fn set_block_range(source: &str, block: &mut Block, range: (usize, usize)) {
    let rows = source_lines(source);
    let start = range.0.saturating_sub(1).min(rows.len().saturating_sub(1));
    let end = range.1.max(range.0).min(rows.len());
    block.src_start = start;
    block.src_end = end.max(start + 1).min(rows.len());
    block.anchor = anchor_text(rows.get(start));
    block.end_anchor = if block.is_range && block.src_end > block.src_start + 1 {
        anchor_text(rows.get(block.src_end - 1))
    } else {
        None
    };
}

fn format_marker(block: &Block) -> String {
    let start = block.src_start + 1;
    let end = block.src_end.max(block.src_start + 1);
    let range = if block.is_range && end > block.src_start + 1 {
        format!("{start}-{end}")
    } else {
        start.to_string()
    };
    let mut parts = Vec::new();
    if let Some(anchor) = &block.anchor {
        parts.push(format!("a={}", escape_anchor(anchor)));
    }
    if let Some(anchor) = &block.end_anchor {
        parts.push(format!("b={}", escape_anchor(anchor)));
    }
    if parts.is_empty() {
        format!("@{range}")
    } else {
        format!("@{range} [{}]", parts.join(","))
    }
}

pub fn rewrite_markers(text: &str, blocks: &[Block]) -> String {
    let mut index = 0;
    text.split('\n')
        .map(|line| {
            if parse_marker(line).is_some() && index < blocks.len() {
                let marker = format_marker(&blocks[index]);
                index += 1;
                marker
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn map_line(line: f32, from_source: bool, blocks: &[Block]) -> f32 {
    for block in blocks {
        let (from_start, from_end, to_start, to_end) = if from_source {
            (
                block.src_start as f32,
                block.src_end as f32,
                block.cmt_start as f32,
                block.cmt_end as f32,
            )
        } else {
            (
                block.cmt_start as f32,
                block.cmt_end as f32,
                block.src_start as f32,
                block.src_end as f32,
            )
        };
        let from_len = (from_end - from_start).max(1.0);
        let to_len = (to_end - to_start).max(1.0);
        if line >= from_start && line < from_end {
            let fraction = ((line - from_start) / from_len).clamp(0.0, 1.0);
            return to_start + fraction * to_len;
        }
    }
    line
}

pub fn map_scroll(
    offset: f32,
    from_source: bool,
    blocks: &[Block],
    from_row_h: f32,
    to_row_h: f32,
    view_h: f32,
) -> f32 {
    let center_line = offset / from_row_h + (view_h / from_row_h) * 0.5;
    let target = map_line(center_line, from_source, blocks);
    (target * to_row_h - view_h * 0.5).max(0.0)
}

pub fn is_marker_line(line: &str) -> bool {
    parse_marker(line.trim()).is_some()
}

pub fn hide_markers(text: &str) -> String {
    text.split('\n')
        .map(|line| if is_marker_line(line) { "" } else { line })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn restore_markers(canonical: &str, edited: &str) -> String {
    let original: Vec<String> = canonical.split('\n').map(str::to_string).collect();
    let display: Vec<String> = original
        .iter()
        .map(|line| {
            if is_marker_line(line) {
                String::new()
            } else {
                line.clone()
            }
        })
        .collect();
    let edited_lines: Vec<String> = edited.split('\n').map(str::to_string).collect();

    let mut prefix = 0;
    while prefix < display.len()
        && prefix < edited_lines.len()
        && display[prefix] == edited_lines[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < display.len() - prefix
        && suffix < edited_lines.len() - prefix
        && display[display.len() - 1 - suffix] == edited_lines[edited_lines.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let mut result: Vec<String> = Vec::new();
    result.extend_from_slice(&original[..prefix]);

    let middle_end = display.len() - suffix;
    for line in &original[prefix..middle_end] {
        if is_marker_line(line) {
            result.push(line.clone());
        }
    }

    result.extend_from_slice(&edited_lines[prefix..edited_lines.len() - suffix]);
    result.extend_from_slice(&original[middle_end..]);
    result.join("\n")
}

pub fn normalize_spacing(text: &str, space_lines: usize) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut result: Vec<String> = Vec::new();
    let mut seen_marker = false;
    for line in lines {
        if is_marker_line(line) {
            if seen_marker {
                while result.last().is_some_and(|last| last.trim().is_empty()) {
                    result.pop();
                }
                for _ in 0..space_lines.saturating_sub(1) {
                    result.push(String::new());
                }
            }
            seen_marker = true;
        }
        result.push(line.to_string());
    }
    result.join("\n")
}
