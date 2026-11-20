mod config;
mod highlight;
mod icon;

use commentor::{
    hide_markers, map_line, normalize_spacing, pair_path, parse_blocks, restore_markers, Block,
};
use config::AppConfig;
use eframe::egui;
use egui::epaint::CubicBezierShape;
use egui::{Align2, Color32, FontId, IconData, Pos2, Rect, Stroke, Vec2};
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

const LINK_COLORS: [(u8, u8, u8); 10] = [
    (92, 150, 214),
    (98, 166, 213),
    (108, 182, 208),
    (122, 196, 198),
    (139, 190, 208),
    (154, 176, 222),
    (171, 162, 224),
    (190, 158, 216),
    (205, 163, 201),
    (216, 174, 185),
];

fn link_rgb(index: usize) -> (u8, u8, u8) {
    LINK_COLORS[index % LINK_COLORS.len()]
}

fn link_stroke_color(index: usize, selected: bool) -> Color32 {
    let (r, g, b) = link_rgb(index);
    if selected {
        Color32::from_rgb(
            r.saturating_add(40),
            g.saturating_add(40),
            b.saturating_add(40),
        )
    } else {
        Color32::from_rgb(r, g, b)
    }
}

fn link_tint_color(index: usize, selected: bool) -> Color32 {
    let (r, g, b) = link_rgb(index);
    Color32::from_rgba_unmultiplied(r, g, b, if selected { 90 } else { 48 })
}

fn source_editor_id() -> egui::Id {
    egui::Id::new("commentor_source_editor")
}

fn comment_editor_id() -> egui::Id {
    egui::Id::new("commentor_comment_editor")
}

fn editor_target(focused: Option<egui::Id>) -> egui::Id {
    focused.unwrap_or_else(comment_editor_id)
}

fn key_event(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn run_editor_action(ctx: &egui::Context, focused: Option<egui::Id>, event: egui::Event) {
    ctx.memory_mut(|memory| memory.request_focus(editor_target(focused)));
    ctx.input_mut(|input| input.events.push(event));
    ctx.request_repaint();
}

fn editor_menu_item(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    focused: Option<egui::Id>,
    label: &str,
    shortcut: egui::KeyboardShortcut,
    event: egui::Event,
) {
    let button = egui::Button::new(label).shortcut_text(ctx.format_shortcut(&shortcut));
    if ui.add_enabled(focused != Some(source_editor_id()) || matches!(label, "Copy" | "Select All"), button).clicked() {
        run_editor_action(ctx, focused, event);
        ui.close_menu();
    }
}

fn editor_context_menu(
    response: &egui::Response,
    ctx: &egui::Context,
    focused: Option<egui::Id>,
    clipboard: &str,
    editable: bool,
) {
    response.context_menu(|ui| {
        let command = egui::Modifiers::COMMAND;
        if editable {
            editor_menu_item(
                ui,
                ctx,
                focused,
                "Undo",
                egui::KeyboardShortcut::new(command, egui::Key::Z),
                key_event(egui::Key::Z, command),
            );
            editor_menu_item(
                ui,
                ctx,
                focused,
                "Redo",
                egui::KeyboardShortcut::new(command | egui::Modifiers::SHIFT, egui::Key::Z),
                key_event(egui::Key::Z, command | egui::Modifiers::SHIFT),
            );
            ui.separator();
            editor_menu_item(
                ui,
                ctx,
                focused,
                "Cut",
                egui::KeyboardShortcut::new(command, egui::Key::X),
                egui::Event::Cut,
            );
        }
        editor_menu_item(
            ui,
            ctx,
            focused,
            "Copy",
            egui::KeyboardShortcut::new(command, egui::Key::C),
            egui::Event::Copy,
        );
        if editable {
            editor_menu_item(
                ui,
                ctx,
                focused,
                "Paste",
                egui::KeyboardShortcut::new(command, egui::Key::V),
                egui::Event::Paste(clipboard.to_owned()),
            );
        }
        ui.separator();
        editor_menu_item(
            ui,
            ctx,
            focused,
            "Select All",
            egui::KeyboardShortcut::new(command, egui::Key::A),
            key_event(egui::Key::A, command),
        );
        if editable {
            editor_menu_item(
                ui,
                ctx,
                focused,
                "Delete",
                egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete),
                key_event(egui::Key::Delete, egui::Modifiers::NONE),
            );
        }
    });
}

const THEMES: [(&str, &str); 2] = [("dark", "Dark"), ("light", "Light")];

const COLOR_SCHEMES: [(&str, &str); 6] = [
    ("blue", "Blue"),
    ("green", "Green"),
    ("purple", "Purple"),
    ("warm", "Warm"),
    ("pink", "Pink"),
    ("neutral", "Neutral"),
];

const LANGUAGES: [(&str, &str); 16] = [
    ("auto", "Auto detect"),
    ("rust", "Rust"),
    ("python", "Python"),
    ("javascript", "JavaScript"),
    ("typescript", "TypeScript"),
    ("c", "C"),
    ("cpp", "C++"),
    ("go", "Go"),
    ("java", "Java"),
    ("json", "JSON"),
    ("toml", "TOML"),
    ("shell", "Shell"),
    ("html", "HTML"),
    ("css", "CSS"),
    ("sql", "SQL"),
    ("plain", "Plain text"),
];

const SYNTAX_THEMES: [(&str, &str); 3] = [
    ("default", "Default"),
    ("vivid", "Vivid"),
    ("soft", "Soft"),
];

fn scheme_accent(name: &str, dark: bool) -> Color32 {
    match name {
        "green" => {
            if dark {
                Color32::from_rgb(94, 200, 150)
            } else {
                Color32::from_rgb(20, 130, 90)
            }
        }
        "purple" => {
            if dark {
                Color32::from_rgb(178, 150, 240)
            } else {
                Color32::from_rgb(110, 70, 190)
            }
        }
        "warm" => {
            if dark {
                Color32::from_rgb(230, 160, 90)
            } else {
                Color32::from_rgb(180, 100, 30)
            }
        }
        "pink" => {
            if dark {
                Color32::from_rgb(232, 140, 190)
            } else {
                Color32::from_rgb(190, 60, 130)
            }
        }
        "neutral" => {
            if dark {
                Color32::from_rgb(160, 168, 178)
            } else {
                Color32::from_rgb(90, 98, 108)
            }
        }
        _ => {
            if dark {
                Color32::from_rgb(92, 150, 214)
            } else {
                Color32::from_rgb(30, 110, 190)
            }
        }
    }
}

fn apply_theme(ctx: &egui::Context, config: &AppConfig) {
    let dark = config.theme != "light";
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let accent = scheme_accent(&config.color_scheme, dark);
    let alpha = |value: u8| Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), value);
    visuals.selection.bg_fill = alpha(if dark { 90 } else { 60 });
    visuals.selection.stroke = Stroke::new(1.0, accent);
    visuals.hyperlink_color = accent;
    visuals.widgets.hovered.bg_fill = alpha(if dark { 60 } else { 36 });
    visuals.widgets.active.bg_fill = alpha(if dark { 90 } else { 54 });
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, alpha(if dark { 70 } else { 50 }));
    ctx.set_visuals(visuals);
}

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
    focused_editor: Option<egui::Id>,
    clipboard: String,
    config: AppConfig,
}

#[derive(Default, Clone)]
struct LineLayout {
    tops: Vec<f32>,
    height: f32,
    row_h: f32,
}

impl LineLayout {
    fn from_galley(galley: &egui::Galley, lines: usize, row_h: f32) -> Self {
        let mut tops = Vec::with_capacity(lines);
        let mut line = 0;
        for row in &galley.rows {
            if line == tops.len() && tops.len() < lines {
                tops.push(row.rect.top());
            }
            if row.ends_with_newline {
                line += 1;
            }
        }
        while tops.len() < lines {
            let last = tops.last().copied().unwrap_or(0.0);
            tops.push(last + row_h);
        }
        Self {
            tops,
            height: galley.rect.height(),
            row_h,
        }
    }

    fn top(&self, line: f32) -> f32 {
        if self.tops.is_empty() {
            return line.max(0.0) * self.row_h;
        }
        let index = (line.max(0.0).floor() as usize).min(self.tops.len() - 1);
        let start = self.tops[index];
        let end = self.tops.get(index + 1).copied().unwrap_or(self.height);
        let frac = (line - index as f32).clamp(0.0, 1.0);
        start + (end - start) * frac
    }

    fn line_at(&self, y: f32) -> usize {
        if self.tops.is_empty() {
            return 0;
        }
        match self
            .tops
            .binary_search_by(|top| top.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal))
        {
            Ok(index) => index,
            Err(0) => 0,
            Err(index) => index - 1,
        }
    }

    fn line_position(&self, y: f32) -> f32 {
        let index = self.line_at(y);
        let start = self.tops.get(index).copied().unwrap_or(0.0);
        let end = self.tops.get(index + 1).copied().unwrap_or(self.height);
        let frac = if end > start {
            ((y - start) / (end - start)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        index as f32 + frac
    }
}

fn paint_pane_background(ui: &mut egui::Ui, pane: Rect, text_area: Rect) {
    let old_clip = ui.clip_rect();
    ui.set_clip_rect(pane);
    let painter = ui.painter();
    painter.rect_filled(pane, 2.0, ui.visuals().extreme_bg_color);
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(pane.min.x, pane.min.y),
            Pos2::new(text_area.min.x, pane.max.y),
        ),
        0.0,
        ui.visuals().faint_bg_color,
    );
    ui.set_clip_rect(old_clip);
}

fn paint_scrolled_layers(
    ui: &egui::Ui,
    content_top: f32,
    text_left: f32,
    text_right: f32,
    lines: &[String],
    blocks: &[Block],
    selected: Option<usize>,
    layout: &LineLayout,
    source_side: bool,
    show_colors: bool,
) {
    let painter = ui.painter();

    for (index, block) in blocks.iter().enumerate() {
        let (start, end) = if source_side {
            (block.src_start, block.src_end)
        } else {
            let mut end = block.cmt_end;
            while end > block.cmt_start
                && lines
                    .get(end - 1)
                    .is_some_and(|line| line.trim().is_empty())
            {
                end -= 1;
            }
            (block.cmt_start, end)
        };
        let y0 = content_top + layout.top(start as f32);
        let y1 = content_top + layout.top(end as f32);
        let top = y0 - 2.0;
        let bottom = (y1 + 2.0).max(top + 1.0);
        let rect = Rect::from_min_max(
            Pos2::new(text_left, top),
            Pos2::new(text_right - 4.0, bottom),
        );
        let is_selected = selected == Some(index);
        if show_colors {
            painter.rect_filled(rect, 3.0, link_tint_color(index, is_selected));
        } else {
            painter.rect_stroke(
                rect,
                3.0,
                Stroke::new(1.0, link_stroke_color(index, is_selected)),
            );
        }
    }
}

fn paint_line_numbers(
    ui: &mut egui::Ui,
    pane: Rect,
    offset: f32,
    layout: &LineLayout,
    font_id: &FontId,
    line_count: usize,
) {
    let old_clip = ui.clip_rect();
    ui.set_clip_rect(pane);
    let painter = ui.painter();
    let first = layout.line_at(offset);
    let count = (pane.height() / layout.row_h).ceil() as usize + 2;
    for line in first..(first + count).min(line_count) {
        let y = pane.min.y + layout.top(line as f32) - offset;
        if y > pane.max.y {
            break;
        }
        painter.text(
            Pos2::new(pane.min.x + 4.0, y),
            Align2::LEFT_TOP,
            (line + 1).to_string(),
            font_id.clone(),
            Color32::GRAY,
        );
    }
    ui.set_clip_rect(old_clip);
}

fn paint_persisted_selection(
    ui: &egui::Ui,
    id: egui::Id,
    galley: &egui::Galley,
    response: &egui::Response,
) {
    if response.has_focus() {
        return;
    }
    if let Some(state) = egui::TextEdit::load_state(ui.ctx(), id) {
        if let Some(cursor_range) = state.cursor.range(galley) {
            egui::text_selection::visuals::paint_text_selection(
                ui.painter(),
                ui.visuals(),
                response.rect.min,
                galley,
                &cursor_range,
                None,
            );
        }
    }
}

fn paint_links(
    ui: &mut egui::Ui,
    strip: Rect,
    left: Rect,
    right: Rect,
    blocks: &[Block],
    selected: Option<usize>,
    left_offset: f32,
    right_offset: f32,
    left_layout: &LineLayout,
    right_layout: &LineLayout,
    show_colors: bool,
) {
    let x0 = left.max.x;
    let x1 = right.min.x;
    let link_area = Rect::from_min_max(
        Pos2::new(x0 - 10.0, strip.min.y),
        Pos2::new(x1 + 10.0, strip.max.y),
    );
    ui.set_clip_rect(link_area);
    let painter = ui.painter();
    painter.rect_filled(strip, 2.0, ui.visuals().faint_bg_color);

    let middle = (x0 + x1) * 0.5;
    let tail = 7.0;
    let half = 4.0;

    for (index, block) in blocks.iter().enumerate() {
        let left_y0 = left.min.y + left_layout.top(block.src_start as f32) - left_offset;
        let left_y1 = left.min.y + left_layout.top(block.src_end as f32) - left_offset;
        let right_y0 = right.min.y + right_layout.top(block.cmt_start as f32) - right_offset;
        let right_y1 = right.min.y + right_layout.top(block.cmt_end as f32) - right_offset;
        let is_selected = selected == Some(index);
        let color = link_stroke_color(index, is_selected);
        let width = if is_selected { 2.4 } else { 1.5 };

        if show_colors {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(x0 + 2.0, left_y0),
                    Pos2::new(x0 + 2.0, left_y1),
                    Pos2::new(x1 - 2.0, right_y1),
                    Pos2::new(x1 - 2.0, right_y0),
                ],
                link_tint_color(index, is_selected),
                Stroke::NONE,
            ));
        }

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
                stroke: Stroke::new(width, color).into(),
            });

            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(x0 - tail, from_y),
                    Pos2::new(x0, from_y - half),
                    Pos2::new(x0, from_y + half),
                ],
                color,
                Stroke::NONE,
            ));
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(x1 + tail, to_y),
                    Pos2::new(x1, to_y - half),
                    Pos2::new(x1, to_y + half),
                ],
                color,
                Stroke::NONE,
            ));
        }
    }
    let screen_rect = ui.ctx().screen_rect();
    ui.set_clip_rect(screen_rect);
}

impl App {
    fn open(&mut self, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(text) => {
                self.source = text;
                self.source_path = Some(path.clone());
                let pair = pair_path(&path);
                let comments = fs::read_to_string(&pair).unwrap_or_default();
                self.comments = normalize_spacing(&comments, self.config.space_lines);
                self.comment_path = Some(pair);
                self.status = format!("opened {}", path.display());
                self.refresh();
                self.remember_last_file();
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

    fn remember_last_file(&mut self) {
        if !self.config.remember_last_file {
            return;
        }
        if let Some(path) = &self.source_path {
            self.config.last_file = Some(path.to_string_lossy().into_owned());
            if let Err(error) = config::save_config(&self.config) {
                self.status.push_str(&format!(" (settings save failed: {error:?})"));
            }
        }
    }

    fn save_settings(&mut self) {
        match config::save_config(&self.config) {
            Ok(()) => self.status = "settings saved".to_string(),
            Err(error) => self.status = format!("settings save failed: {error:?}"),
        }
    }

    fn set_space_lines(&mut self, value: usize) {
        let value = value.clamp(1, 8);
        if self.config.space_lines == value {
            return;
        }
        self.config.space_lines = value;
        self.comments = normalize_spacing(&self.comments, value);
        self.refresh();
        self.save_settings();
    }

    fn set_source_font(&mut self, value: f32) {
        let value = value.clamp(8.0, 32.0);
        if (self.config.source_font_size - value).abs() < f32::EPSILON {
            return;
        }
        self.config.source_font_size = value;
        self.save_settings();
    }

    fn set_comment_font(&mut self, value: f32) {
        let value = value.clamp(8.0, 32.0);
        if (self.config.comment_font_size - value).abs() < f32::EPSILON {
            return;
        }
        self.config.comment_font_size = value;
        self.save_settings();
    }

    fn active_language(&self) -> highlight::Language {
        if self.config.language == "auto" {
            self.source_path
                .as_deref()
                .map(highlight::Language::from_path)
                .unwrap_or(highlight::Language::Plain)
        } else {
            highlight::Language::from_setting(&self.config.language)
        }
    }

    fn select_relative(&mut self, step: isize) {
        if self.blocks.is_empty() {
            self.selected = None;
            return;
        }
        let current = self
            .selected
            .map(|index| index as isize)
            .unwrap_or(if step > 0 { -1 } else { 0 });
        let next = (current + step).rem_euclid(self.blocks.len() as isize) as usize;
        self.selected = Some(next);
    }
}

fn open_target(target: &str) {
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", target])
        .spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(target).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(target).spawn();
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, &self.config);

        if ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S)) {
            self.save();
        }

        let mut menu_open = false;
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                menu_open |= ui.menu_button("File", |ui| {
                    if ui.button("Open Source...").clicked() {
                        if let Some(path) = FileDialog::new().pick_file() {
                            self.open(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save Comments").clicked() {
                        self.save();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
                        ui.close_menu();
                    }
                }).inner.is_some();
                menu_open |= ui.menu_button("Edit", |ui| {
                    let command = egui::Modifiers::COMMAND;
                    let focused = self.focused_editor;
                    editor_menu_item(
                        ui,
                        ctx,
                        focused,
                        "Undo",
                        egui::KeyboardShortcut::new(command, egui::Key::Z),
                        key_event(egui::Key::Z, command),
                    );
                    editor_menu_item(
                        ui,
                        ctx,
                        focused,
                        "Redo",
                        egui::KeyboardShortcut::new(command | egui::Modifiers::SHIFT, egui::Key::Z),
                        key_event(egui::Key::Z, command | egui::Modifiers::SHIFT),
                    );
                    ui.separator();
                    editor_menu_item(
                        ui,
                        ctx,
                        focused,
                        "Cut",
                        egui::KeyboardShortcut::new(command, egui::Key::X),
                        egui::Event::Cut,
                    );
                    editor_menu_item(
                        ui,
                        ctx,
                        focused,
                        "Copy",
                        egui::KeyboardShortcut::new(command, egui::Key::C),
                        egui::Event::Copy,
                    );
                    editor_menu_item(
                        ui,
                        ctx,
                        focused,
                        "Paste",
                        egui::KeyboardShortcut::new(command, egui::Key::V),
                        egui::Event::Paste(self.clipboard.clone()),
                    );
                    ui.separator();
                    editor_menu_item(
                        ui,
                        ctx,
                        focused,
                        "Select All",
                        egui::KeyboardShortcut::new(command, egui::Key::A),
                        key_event(egui::Key::A, command),
                    );
                    editor_menu_item(
                        ui,
                        ctx,
                        focused,
                        "Delete",
                        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete),
                        key_event(egui::Key::Delete, egui::Modifiers::NONE),
                    );
                }).inner.is_some();
                menu_open |= ui.menu_button("View", |ui| {
                    if ui.button("Next Link").clicked() {
                        self.select_relative(1);
                        ui.close_menu();
                    }
                    if ui.button("Previous Link").clicked() {
                        self.select_relative(-1);
                        ui.close_menu();
                    }
                    if ui.button("Reset Scroll").clicked() {
                        self.left_offset = 0.0;
                        self.right_offset = 0.0;
                        self.prev_left = 0.0;
                        self.prev_right = 0.0;
                        self.scroll_from = 0;
                        ui.close_menu();
                    }
                }).inner.is_some();
                menu_open |= ui.menu_button("Settings", |ui| {
                    if ui
                        .checkbox(&mut self.config.remember_last_file, "Remember last file")
                        .changed()
                    {
                        if !self.config.remember_last_file {
                            self.config.last_file = None;
                        }
                        self.save_settings();
                    }
                    if ui
                        .checkbox(&mut self.config.show_line_numbers, "Show line numbers")
                        .changed()
                    {
                        self.save_settings();
                    }
                    if ui
                        .checkbox(&mut self.config.comment_wrap, "Wrap comment text")
                        .changed()
                    {
                        self.save_settings();
                    }
                    if ui
                        .checkbox(
                            &mut self.config.show_link_colors,
                            "Show link block colors",
                        )
                        .changed()
                    {
                        self.save_settings();
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        egui::ComboBox::from_id_source("theme")
                            .selected_text(
                                THEMES
                                    .iter()
                                    .find(|(value, _)| *value == self.config.theme.as_str())
                                    .map(|(_, label)| *label)
                                    .unwrap_or("Dark"),
                            )
                            .show_ui(ui, |ui| {
                                for (value, label) in THEMES {
                                    if ui
                                        .selectable_value(
                                            &mut self.config.theme,
                                            value.to_string(),
                                            label,
                                        )
                                        .changed()
                                    {
                                        self.save_settings();
                                    }
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Colors:");
                        egui::ComboBox::from_id_source("color_scheme")
                            .selected_text(
                                COLOR_SCHEMES
                                    .iter()
                                    .find(|(value, _)| *value == self.config.color_scheme.as_str())
                                    .map(|(_, label)| *label)
                                    .unwrap_or("Blue"),
                            )
                            .show_ui(ui, |ui| {
                                for (value, label) in COLOR_SCHEMES {
                                    if ui
                                        .selectable_value(
                                            &mut self.config.color_scheme,
                                            value.to_string(),
                                            label,
                                        )
                                        .changed()
                                    {
                                        self.save_settings();
                                    }
                                }
                            });
                    });
                    ui.separator();
                    if ui
                        .checkbox(&mut self.config.highlight, "Syntax highlighting")
                        .changed()
                    {
                        self.save_settings();
                    }
                    ui.horizontal(|ui| {
                        ui.label("Language:");
                        egui::ComboBox::from_id_source("language")
                            .selected_text(
                                LANGUAGES
                                    .iter()
                                    .find(|(value, _)| *value == self.config.language.as_str())
                                    .map(|(_, label)| *label)
                                    .unwrap_or("Auto detect"),
                            )
                            .show_ui(ui, |ui| {
                                for (value, label) in LANGUAGES {
                                    if ui
                                        .selectable_value(
                                            &mut self.config.language,
                                            value.to_string(),
                                            label,
                                        )
                                        .changed()
                                    {
                                        self.save_settings();
                                    }
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Highlight:");
                        egui::ComboBox::from_id_source("syntax_theme")
                            .selected_text(
                                SYNTAX_THEMES
                                    .iter()
                                    .find(|(value, _)| *value == self.config.syntax_theme.as_str())
                                    .map(|(_, label)| *label)
                                    .unwrap_or("Default"),
                            )
                            .show_ui(ui, |ui| {
                                for (value, label) in SYNTAX_THEMES {
                                    if ui
                                        .selectable_value(
                                            &mut self.config.syntax_theme,
                                            value.to_string(),
                                            label,
                                        )
                                        .changed()
                                    {
                                        self.save_settings();
                                    }
                                }
                            });
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Space lines:");
                        if ui.small_button("-").clicked() {
                            self.set_space_lines(self.config.space_lines.saturating_sub(1));
                        }
                        ui.label(self.config.space_lines.to_string());
                        if ui.small_button("+").clicked() {
                            self.set_space_lines(self.config.space_lines + 1);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Source font:");
                        if ui.small_button("-").clicked() {
                            self.set_source_font(self.config.source_font_size - 1.0);
                        }
                        ui.label(format!("{:.0}", self.config.source_font_size));
                        if ui.small_button("+").clicked() {
                            self.set_source_font(self.config.source_font_size + 1.0);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Comment font:");
                        if ui.small_button("-").clicked() {
                            self.set_comment_font(self.config.comment_font_size - 1.0);
                        }
                        ui.label(format!("{:.0}", self.config.comment_font_size));
                        if ui.small_button("+").clicked() {
                            self.set_comment_font(self.config.comment_font_size + 1.0);
                        }
                    });
                    ui.separator();
                    if ui.button("Open Settings File").clicked() {
                        if let Ok(path) = config::config_path() {
                            let target = path.to_string_lossy().into_owned();
                            open_target(&target);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save Settings Now").clicked() {
                        self.save_settings();
                        ui.close_menu();
                    }
                }).inner.is_some();
                menu_open |= ui.menu_button("Help", |ui| {
                    if ui.button("Project Page").clicked() {
                        open_target("https://github.com/myridia/commentor");
                        ui.close_menu();
                    }
                    if ui.button("About").clicked() {
                        self.status = "commentor 0.1.0 - comments beside source files".to_string();
                        ui.close_menu();
                    }
                }).inner.is_some();
            });
            if menu_open { if let Some(id) = self.focused_editor { ctx.memory_mut(|memory| memory.request_focus(id)); } }
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let area = ui.available_rect_before_wrap();
            let source_font = FontId::monospace(self.config.source_font_size);
            let comment_font = FontId::monospace(self.config.comment_font_size);
            let source_row_h = ui.fonts(|fonts| fonts.row_height(&source_font));
            let comment_row_h = ui.fonts(|fonts| fonts.row_height(&comment_font));
            let source_gutter = gutter_width(ui, &source_font, self.source.lines().count());
            let comment_gutter = gutter_width(ui, &comment_font, self.comments.lines().count());
            let gap = 4.0;
            let strip_w = 24.0;
            let panes_w = (area.width() - strip_w - gap * 2.0).max(320.0);
            let source_w = panes_w * 0.58;
            let comment_w = panes_w - source_w;
            let pane_h = area.height().max(60.0);

            let left_rect = Rect::from_min_size(area.min, Vec2::new(source_w, pane_h));
            let strip_rect = Rect::from_min_size(
                Pos2::new(left_rect.right() + gap, area.min.y),
                Vec2::new(strip_w, pane_h),
            );
            let right_rect = Rect::from_min_size(
                Pos2::new(strip_rect.right() + gap, area.min.y),
                Vec2::new(comment_w, pane_h),
            );
            let left_text = Rect::from_min_size(
                Pos2::new(left_rect.min.x + source_gutter, left_rect.min.y),
                Vec2::new((source_w - source_gutter).max(20.0), pane_h),
            );
            let right_text = Rect::from_min_size(
                Pos2::new(right_rect.min.x + comment_gutter, right_rect.min.y),
                Vec2::new((comment_w - comment_gutter).max(20.0), pane_h),
            );

            let blocks = self.blocks.clone();
            let selected = self.selected;
            let source_lines: Vec<String> = self.source.lines().map(str::to_string).collect();
            let comment_lines: Vec<String> = self.comments.lines().map(str::to_string).collect();
            let comments_display = hide_markers(&self.comments);
            let source_edit = self.source.clone();
            let mut source_view: &str = &source_edit;
            let mut comments_edit = comments_display.clone();
            let dark = self.config.theme != "light";
            let syntax_palette = highlight::Palette::new(dark, &self.config.syntax_theme);
            let language = self.active_language();
            let plain_color = ui.visuals().widgets.inactive.text_color();
            let show_link_colors = self.config.show_link_colors;
            let (source_galley, source_layout) = {
                let mut job = if self.config.highlight {
                    highlight::layout_job(
                        &source_edit,
                        &source_font,
                        language,
                        &syntax_palette,
                        plain_color,
                    )
                } else {
                    egui::text::LayoutJob::simple(
                        source_edit.clone(),
                        source_font.clone(),
                        plain_color,
                        f32::INFINITY,
                    )
                };
                job.wrap.max_width = f32::INFINITY;
                let galley = ui.fonts(|fonts| fonts.layout_job(job));
                let layout = LineLayout::from_galley(&galley, source_lines.len(), source_row_h);
                (galley, layout)
            };
            let wrap_comments = self.config.comment_wrap;
            let comment_wrap_w =
                (right_text.width() - ui.spacing().scroll.allocated_width()).max(40.0);
            let comment_wrap_width = if wrap_comments {
                comment_wrap_w
            } else {
                f32::INFINITY
            };
            let comment_job = egui::text::LayoutJob::simple(
                comments_display.clone(),
                comment_font.clone(),
                plain_color,
                comment_wrap_width,
            );
            let (comment_galley, comment_layout) = {
                let galley = ui.fonts(|fonts| fonts.layout_job(comment_job));
                let layout = LineLayout::from_galley(&galley, comment_lines.len(), comment_row_h);
                (galley, layout)
            };

            let mut target_left = self.left_offset;
            let mut target_right = self.right_offset;
            let mut applied_sync = false;
            if self.scroll_from == 1 {
                let center = source_layout.line_position(self.left_offset + pane_h * 0.5);
                let target_line = map_line(center, true, &self.blocks);
                target_right = (comment_layout.top(target_line) - pane_h * 0.5).max(0.0);
                self.scroll_from = 0;
                applied_sync = true;
            } else if self.scroll_from == 2 {
                let center = comment_layout.line_position(self.right_offset + pane_h * 0.5);
                let target_line = map_line(center, false, &self.blocks);
                target_left = (source_layout.top(target_line) - pane_h * 0.5).max(0.0);
                self.scroll_from = 0;
                applied_sync = true;
            }

            let strip_response = ui.allocate_rect(strip_rect, egui::Sense::click());

            paint_pane_background(ui, left_rect, left_text);
            paint_pane_background(ui, right_rect, right_text);
            if self.config.show_line_numbers {
                paint_line_numbers(
                    ui,
                    left_rect,
                    target_left,
                    &source_layout,
                    &source_font,
                    source_lines.len(),
                );
                paint_line_numbers(
                    ui,
                    right_rect,
                    target_right,
                    &comment_layout,
                    &comment_font,
                    comment_lines.len(),
                );
            }

            let mut left_response = None;
            let mut left_child = ui.child_ui_with_id_source(
                left_text,
                egui::Layout::top_down(egui::Align::Min),
                "source_pane",
                None,
            );
            left_child.set_clip_rect(left_text);
            let highlight_source = self.config.highlight;
            let mut source_layouter =
                |layouter_ui: &egui::Ui, string: &str, _wrap_width: f32| {
                    let mut job = if highlight_source {
                        highlight::layout_job(
                            string,
                            &source_font,
                            language,
                            &syntax_palette,
                            plain_color,
                        )
                    } else {
                        egui::text::LayoutJob::simple(
                            string.to_owned(),
                            source_font.clone(),
                            plain_color,
                            f32::INFINITY,
                        )
                    };
                    job.wrap.max_width = f32::INFINITY;
                    layouter_ui.fonts(|fonts| fonts.layout_job(job))
                };
            let left_output = egui::ScrollArea::both()
                .vertical_scroll_offset(target_left)
                .show(&mut left_child, |ui| {
                    paint_scrolled_layers(
                        ui,
                        left_text.min.y - target_left,
                        left_text.min.x,
                        left_text.max.x,
                        &source_lines,
                        &blocks,
                        selected,
                        &source_layout,
                        true,
                        show_link_colors,
                    );
                    let editor = egui::TextEdit::multiline(&mut source_view)
                        .id(source_editor_id())
                        .desired_width(f32::INFINITY)
                        .frame(false)
                        .margin(Vec2::ZERO)
                        .font(source_font.clone())
                        .layouter(&mut source_layouter);
                    let response = ui.add(editor);
                    if menu_open {
                        paint_persisted_selection(ui, source_editor_id(), &source_galley, &response);
                    }
                    left_response = Some(response);
                });

            let mut comment_layouter =
                |layouter_ui: &egui::Ui, string: &str, _wrap_width: f32| {
                    let job = egui::text::LayoutJob::simple(
                        string.to_owned(),
                        comment_font.clone(),
                        plain_color,
                        comment_wrap_width,
                    );
                    layouter_ui.fonts(|fonts| fonts.layout_job(job))
                };
            let mut right_response = None;
            let mut right_child = ui.child_ui_with_id_source(
                right_text,
                egui::Layout::top_down(egui::Align::Min),
                "comment_pane",
                None,
            );
            right_child.set_clip_rect(right_text);
            let right_area = if wrap_comments {
                egui::ScrollArea::vertical().scroll_bar_visibility(
                    egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
                )
            } else {
                egui::ScrollArea::both()
            };
            let right_output = right_area
                .vertical_scroll_offset(target_right)
                .show(&mut right_child, |ui| {
                    paint_scrolled_layers(
                        ui,
                        right_text.min.y - target_right,
                        right_text.min.x,
                        right_text.max.x,
                        &comment_lines,
                        &blocks,
                        selected,
                        &comment_layout,
                        false,
                        show_link_colors,
                    );
                    let editor = egui::TextEdit::multiline(&mut comments_edit)
                        .id(comment_editor_id())
                        .desired_width(comment_wrap_width)
                        .frame(false)
                        .margin(Vec2::ZERO)
                        .font(comment_font.clone())
                        .layouter(&mut comment_layouter);
                    let response = ui.add(editor);
                    if menu_open {
                        paint_persisted_selection(ui, comment_editor_id(), &comment_galley, &response);
                    }
                    right_response = Some(response);
                });

            let new_left = left_output.state.offset.y;
            let new_right = right_output.state.offset.y;

            paint_links(
                ui,
                strip_rect,
                left_text,
                right_text,
                &blocks,
                selected,
                new_left,
                new_right,
                &source_layout,
                &comment_layout,
                show_link_colors,
            );

            if let Some(response) = &left_response {
                if response.clicked() {
                    if let Some(point) = response.interact_pointer_pos() {
                        let line =
                            source_layout.line_at(point.y - left_text.min.y + new_left);
                        self.selected = blocks.iter().position(|block| {
                            line >= block.src_start && line <= block.src_end
                        });
                    }
                }
            }
            if let Some(response) = &right_response {
                if response.clicked() {
                    if let Some(point) = response.interact_pointer_pos() {
                        let line =
                            comment_layout.line_at(point.y - right_text.min.y + new_right);
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
                            + source_layout.top((block.src_start + block.src_end) as f32 * 0.5)
                            - new_left;
                        let right_y = right_text.min.y
                            + comment_layout.top((block.cmt_start + block.cmt_end) as f32 * 0.5)
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

            if left_response.as_ref().is_some_and(|response| response.has_focus()) {
                self.focused_editor = Some(source_editor_id());
            } else if right_response.as_ref().is_some_and(|response| response.has_focus()) {
                self.focused_editor = Some(comment_editor_id());
            }
            if let Some(response) = &left_response {
                editor_context_menu(response, ctx, self.focused_editor, &self.clipboard, false);
            }
            if let Some(response) = &right_response {
                editor_context_menu(response, ctx, self.focused_editor, &self.clipboard, true);
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

            if comments_edit != comments_display {
                self.comments = restore_markers(&self.comments, &comments_edit);
                self.refresh();
            }
        });

        let copied = ctx.output(|output| output.copied_text.clone());
        if !copied.is_empty() {
            self.clipboard = copied;
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let icon_data = IconData {
        rgba: icon::get_icon(),
        width: 32,
        height: 32,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(1100.0, 760.0))
            .with_icon(icon_data),
        ..Default::default()
    };
    eframe::run_native(
        "commentor",
        options,
        Box::new(|_cc| {
            let config = config::get_config();
            let mut app = App {
                config,
                ..Default::default()
            };
            let start_path = std::env::args().nth(1).or_else(|| {
                if app.config.remember_last_file {
                    app.config.last_file.clone()
                } else {
                    None
                }
            });
            if let Some(path) = start_path {
                app.open(PathBuf::from(path));
            }
            Ok(Box::new(app))
        }),
    )
}

fn gutter_width(ui: &egui::Ui, font: &FontId, lines: usize) -> f32 {
    let digits = lines.max(1).to_string().len();
    let digit = ui.fonts(|fonts| fonts.glyph_width(font, '0'));
    (4.0 + digits as f32 * digit + 6.0).max(18.0)
}
