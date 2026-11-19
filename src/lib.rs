use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub src_start: usize,
    pub src_end: usize,
    pub cmt_start: usize,
    pub cmt_end: usize,
}

pub fn pair_path(path: &Path) -> PathBuf {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    dir.join(".comments.d").join(format!("{name}.cmt"))
}

pub fn parse_marker(line: &str) -> Option<(usize, usize)> {
    let rest = line.strip_prefix('@')?;
    let mut parts = rest.splitn(2, '-');
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

pub fn parse_blocks(text: &str, source_lines: usize) -> Vec<Block> {
    let lines: Vec<&str> = text.lines().collect();
    let mut markers: Vec<(usize, usize, usize)> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if let Some((start, end)) = parse_marker(line) {
            let src_start = start.min(source_lines);
            let src_end = end.min(source_lines).max(src_start);
            markers.push((index, src_start, src_end));
        }
    }
    let mut blocks = Vec::new();
    for (position, (cmt_start, src_start, src_end)) in markers.iter().enumerate() {
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
        });
    }
    blocks
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
