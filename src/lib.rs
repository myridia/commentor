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
        if line >= from_start && line <= from_end {
            let from_len = (from_end - from_start).max(1.0);
            let to_len = (to_end - to_start).max(1.0);
            return to_start + ((line - from_start) / from_len) * to_len;
        }
    }
    line
}

pub fn map_scroll(
    offset: f32,
    from_source: bool,
    blocks: &[Block],
    row_h: f32,
    view_h: f32,
) -> f32 {
    let center_line = offset / row_h + (view_h / row_h) * 0.5;
    let target = map_line(center_line, from_source, blocks);
    (target * row_h - view_h * 0.5).max(0.0)
}
