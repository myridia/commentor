use commentor::{map_scroll, pair_path, parse_blocks, Block};
use eframe::egui;
use egui::epaint::CubicBezierShape;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

#[derive(Default)]
struct App {
    source: String,
    comments: String,
    source_path: Option<PathBuf>,
    comment_path: Option<PathBuf>,
    status: String,
    blocks: Vec<Block>,
    selected: Option<usize>,
    left_offset: f32,
    right_offset: f32,
    prev_left: f32,
    prev_right: f32,
    scroll_from: u8,
}

fn paint_pane(
    ui: &egui::Ui,
    pane: Rect,
    text_area: Rect,
    lines: &[String],
    blocks: &[Block],
    selected: Option<usize>,
    offset: f32,
    row_h: f32,
    font_id: &FontId,
    source_side: bool,
) {
    let painter = ui.painter();
    painter.set_clip_rect(pane);
    painter.rect_filled(pane, 2.0, ui.visuals().extreme_bg_color);

    for (index, block) in blocks.iter().enumerate() {
        let (start, end) = if source_side {
            (block.src_start, block.src_end)
        } else {
            (block.cmt_start, block.cmt_end)
        };
        let y0 = pane.min.y + start as f32 * row_h - offset;
        let y1 = pane.min.y + end as f32 * row_h - offset;
        let color = if selected == Some(index) {
            Color32::from_rgba_premultiplied(80, 145, 225, 72)
        } else {
            Color32::from_rgba_premultiplied(70, 110, 170, 40)
        };
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(text_area.min.x, y0),
                Pos2::new(text_area.max.x, y1),
            ),
            0.0,
            color,
        );
    }

    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(pane.min.x, pane.min.y),
            Pos2::new(text_area.min.x, pane.max.y),
        ),
        0.0,
        ui.visuals().faint_bg_color,
    );

    let first = (offset / row_h).floor().max(0.0) as usize;
    let count = (pane.height() / row_h).ceil() as usize + 2;
    for line in first..(first + count).min(lines.len()) {
        let y = pane.min.y + line as f32 * row_h - offset;
        painter.text(
            Pos2::new(pane.min.x + 4.0, y),
            Align2::LEFT_TOP,
            (line + 1).to_string(),
            font_id.clone(),
            Color32::GRAY,
        );
    }
    painter.set_clip_rect(ui.ctx().screen_rect());
}

fn paint_links(
    ui: &egui::Ui,
    strip: Rect,
    left: Rect,
    right: Rect,
    blocks: &[Block],
    selected: Option<usize>,
    left_offset: f32,
    right_offset: f32,
    row_h: f32,
) {
    let painter = ui.painter();
    painter.set_clip_rect(strip);
    painter.rect_filled(strip, 2.0, ui.visuals().faint_bg_color);

    let x0 = left.max.x;
    let x1 = right.min.x;
    let middle = (x0 + x1) * 0.5;

    for (index, block) in blocks.iter().enumerate() {
        let left_y0 = left.min.y + block.src_start as f32 * row_h - left_offset;
        let left_y1 = left.min.y + block.src_end as f32 * row_h - left_offset;
        let right_y0 = right.min.y + block.cmt_start as f32 * row_h - right_offset;
        let right_y1 = right.min.y + block.cmt_end as f32 * row_h - right_offset;
        let color = if selected == Some(index) {
            Color32::from_rgb(130, 210, 255)
        } else {
            Color32::from_rgb(90, 145, 195)
        };
        for &(from_y, to_y) in &[(left_y0, right_y0), (left_y1, right_y1)] {
            painter.add(CubicBezierShape {
                points: [
                    Pos2::new(x0, from_y),
                    Pos2::new(middle, from_y),
                    Pos2::new(middle, to_y),
                    Pos2::new(x1, to_y),
                ],
                closed: false,
                fill: Color32::TRANSPARENT,
                stroke: Stroke::new(1.5, color),
            });
        }
    }
    painter.set_clip_rect(ui.ctx().screen_rect());
}

impl App {
    fn open(&mut self, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(text) => {
                self.source = text;
                self.source_path = Some(path.clone());
                let pair = pair_path(&path);
                self.comments = fs::read_to_string(&pair).unwrap_or_default();
                self.comment_path = Some(pair);
                self.status = format!("opened {}", path.display());
                self.refresh();
            }
            Err(error) => {
                self.status = format!("open failed: {error}");
            }
        }
    }

    fn save(&mut self) {
        let Some(path) = self.comment_path.clone() else {
            self.status = "no file open".to_string();
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                self.status = format!("save failed: {error}");
                return;
            }
        }
        match fs::write(&path, &self.comments) {
            Ok(()) => self.status = format!("saved {}", path.display()),
            Err(error) => self.status = format!("save failed: {error}"),
        }
    }

    fn refresh(&mut self) {
        let count = self.source.lines().count();
        self.blocks = parse_blocks(&self.comments, count);
        if let Some(index) = self.selected {
            if index >= self.blocks.len() {
                self.selected = None;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S)) {
            self.save();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        self.open(path);
                    }
                }
                if ui.button("Save").clicked() {
                    self.save();
                }
                ui.label(&self.status);
            });
            ui.separator();

            let area = ui.available_rect_before_wrap();
            let font_id = FontId::monospace(13.0);
            let row_h = ui.fonts(|fonts| fonts.row_height(&font_id));
            let gutter_w = 46.0;
            let gap = 6.0;
            let strip_w = 26.0;
            let pane_w = ((area.width() - strip_w - gap * 2.0) / 2.0).max(160.0);
            let pane_h = area.height().max(60.0);

            let left_rect = Rect::from_min_size(area.min, Vec2::new(pane_w, pane_h));
            let strip_rect = Rect::from_min_size(
                Pos2::new(left_rect.right() + gap, area.min.y),
                Vec2::new(strip_w, pane_h),
            );
            let right_rect = Rect::from_min_size(
                Pos2::new(strip_rect.right() + gap, area.min.y),
                Vec2::new(pane_w, pane_h),
            );
            let left_text = Rect::from_min_size(
                Pos2::new(left_rect.min.x + gutter_w, left_rect.min.y),
                Vec2::new((pane_w - gutter_w).max(20.0), pane_h),
            );
            let right_text = Rect::from_min_size(
                Pos2::new(right_rect.min.x + gutter_w, right_rect.min.y),
                Vec2::new((pane_w - gutter_w).max(20.0), pane_h),
            );

            let mut target_left = self.left_offset;
            let mut target_right = self.right_offset;
            let mut applied_sync = false;
            if self.scroll_from == 1 {
                target_right = map_scroll(self.left_offset, true, &self.blocks, row_h, pane_h);
                self.scroll_from = 0;
                applied_sync = true;
            } else if self.scroll_from == 2 {
                target_left = map_scroll(self.right_offset, false, &self.blocks, row_h, pane_h);
                self.scroll_from = 0;
                applied_sync = true;
            }

            let blocks = self.blocks.clone();
            let selected = self.selected;
            let source_lines: Vec<String> = self.source.lines().map(str::to_string).collect();
            let comment_lines: Vec<String> = self.comments.lines().map(str::to_string).collect();
            let mut source_edit = self.source.clone();
            let mut comments_edit = self.comments.clone();
            let strip_response = ui.allocate_rect(strip_rect, egui::Sense::click());

            paint_pane(
                ui,
                left_rect,
                left_text,
                &source_lines,
                &blocks,
                selected,
                target_left,
                row_h,
                &font_id,
                true,
            );
            paint_pane(
                ui,
                right_rect,
                right_text,
                &comment_lines,
                &blocks,
                selected,
                target_right,
                row_h,
                &font_id,
                false,
            );
            paint_links(
                ui,
                strip_rect,
                left_text,
                right_text,
                &blocks,
                selected,
                target_left,
                target_right,
                row_h,
            );

            let mut left_response = None;
            let mut left_child =
                ui.child_ui(left_text, egui::Layout::top_down(egui::Align::Min), None);
            let left_output = egui::ScrollArea::vertical()
                .scroll_offset(Vec2::new(0.0, target_left))
                .show(&mut left_child, |ui| {
                    let editor = egui::TextEdit::multiline(&mut source_edit)
                        .desired_width(f32::INFINITY)
                        .interactive(false)
                        .frame(false)
                        .margin(Vec2::ZERO)
                        .font(font_id.clone());
                    left_response = Some(ui.add_sized(
                        Vec2::new(left_text.width(), left_text.height()),
                        editor,
                    ));
                });

            let mut right_response = None;
            let mut right_child =
                ui.child_ui(right_text, egui::Layout::top_down(egui::Align::Min), None);
            let right_output = egui::ScrollArea::vertical()
                .scroll_offset(Vec2::new(0.0, target_right))
                .show(&mut right_child, |ui| {
                    let editor = egui::TextEdit::multiline(&mut comments_edit)
                        .desired_width(f32::INFINITY)
                        .frame(false)
                        .margin(Vec2::ZERO)
                        .font(font_id.clone());
                    right_response = Some(ui.add_sized(
                        Vec2::new(right_text.width(), right_text.height()),
                        editor,
                    ));
                });

            let new_left = left_output.state.offset.y;
            let new_right = right_output.state.offset.y;

            if let Some(response) = left_response {
                if response.clicked() {
                    if let Some(point) = response.interact_pointer_pos() {
                        let line = ((point.y - left_text.min.y + new_left) / row_h) as usize;
                        self.selected = blocks.iter().position(|block| {
                            line >= block.src_start && line <= block.src_end
                        });
                    }
                }
            }
            if let Some(response) = right_response {
                if response.clicked() {
                    if let Some(point) = response.interact_pointer_pos() {
                        let line = ((point.y - right_text.min.y + new_right) / row_h) as usize;
                        self.selected = blocks.iter().position(|block| {
                            line >= block.cmt_start && line <= block.cmt_end
                        });
                    }
                }
            }
            if strip_response.clicked() {
                if let Some(point) = strip_response.interact_pointer_pos() {
                    let mut best: Option<usize> = None;
                    let mut best_distance = f32::MAX;
                    for (index, block) in blocks.iter().enumerate() {
                        let left_y = left_text.min.y
                            + (block.src_start + block.src_end) as f32 * 0.5 * row_h
                            - new_left;
                        let right_y = right_text.min.y
                            + (block.cmt_start + block.cmt_end) as f32 * 0.5 * row_h
                            - new_right;
                        let distance = (point.y - (left_y + right_y) * 0.5).abs();
                        if distance < best_distance {
                            best_distance = distance;
                            best = Some(index);
                        }
                    }
                    self.selected = best;
                }
            }

            if !applied_sync {
                if (new_left - self.prev_left).abs() > 0.5 {
                    self.scroll_from = 1;
                } else if (new_right - self.prev_right).abs() > 0.5 {
                    self.scroll_from = 2;
                }
            }
            self.prev_left = new_left;
            self.prev_right = new_right;
            self.left_offset = new_left;
            self.right_offset = new_right;

            if comments_edit != self.comments {
                self.comments = comments_edit;
                self.refresh();
            }
            if source_edit != self.source {
                self.source = source_edit;
                self.refresh();
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(1100.0, 760.0)),
        ..Default::default()
    };
    eframe::run_native(
        "commentor",
        options,
        Box::new(|_cc| {
            let mut app = App::default();
            if let Some(argument) = std::env::args().nth(1) {
                app.open(PathBuf::from(argument));
            }
            Ok(Box::new(app))
        }),
    )
}
