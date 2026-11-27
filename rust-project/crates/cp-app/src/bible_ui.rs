use crate::{app::App, appearance};
use cp_bible::{initials::candidates, CHINESE_BOOK_NAMES};
use cp_core::BibleTranslationSettings;
use egui::{
    vec2, Button, Color32, ComboBox, Context, DragValue, Key, Layout, ScrollArea, TextEdit, Ui,
};
use std::time::{Duration, Instant};

impl App {
    pub fn bible_tab(&mut self, ui: &mut Ui) {
        let mut stack = self.bible.stack.clone();
        ui.horizontal_wrapped(|ui| {
            ui.heading("圣经速投");
            ui.separator();
            ui.label("主译本");
            let label = self
                .bible
                .primary()
                .map(|b| b.abbreviation())
                .unwrap_or("请选择");
            ComboBox::from_id_salt("primary_bible")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    for (i, b) in self.bible.modules.iter().enumerate() {
                        if ui
                            .selectable_label(stack.first() == Some(&i), b.title())
                            .clicked()
                        {
                            if let Some(old) = stack.first_mut() {
                                *old = i;
                            } else {
                                stack.push(i);
                            }
                            let mut seen = std::collections::HashSet::new();
                            stack.retain(|v| seen.insert(*v));
                        }
                    }
                });
            ui.menu_button("对照译本 ▾", |ui| {
                for (i, b) in self.bible.modules.iter().enumerate() {
                    if stack.first() == Some(&i) {
                        continue;
                    }
                    let mut on = stack.contains(&i);
                    if ui.checkbox(&mut on, b.title()).changed() {
                        if on {
                            stack.push(i);
                        } else {
                            stack.retain(|&v| v != i);
                        }
                    }
                }
            });
            for &i in stack.iter().skip(1) {
                ui.small(self.bible.modules[i].abbreviation());
            }
        });
        if stack != self.bible.stack {
            self.bible.stack = stack;
            self.sync_stack();
            self.bible.set_location(self.bible.book, self.bible.chapter);
            self.bible.forget_command();
        }
        ui.add_space(4.);
        let input = ui.add(
            TextEdit::singleline(&mut self.quick)
                .id_salt("quick_bible")
                .desired_width(f32::INFINITY)
                .hint_text("输入书卷拼音首字母、章、起止节，例如 csj 1 12 16"),
        );
        if self.focus_search {
            input.request_focus();
            self.focus_search = false;
        }
        let enter = input.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        let delay = Duration::from_millis(300);
        if input.changed() {
            self.deadline = Some(Instant::now() + delay);
            if self.quick.is_empty() {
                self.bible.forget_command();
            }
            let matches = self.candidate_books();
            if matches.len() == 1 && self.quick.trim().chars().all(|c| c.is_ascii_alphabetic()) {
                self.bible.set_location(matches[0], 1);
            }
        }
        if enter {
            self.apply_quick(true);
            input.request_focus();
        } else if let Some(t) = self.deadline {
            if Instant::now() >= t {
                self.apply_quick(false);
            } else {
                ui.ctx()
                    .request_repaint_after(t.saturating_duration_since(Instant::now()));
            }
        }
        if let Some(label) = cp_bible::quick_passage_label(&self.quick) {
            ui.label(egui::RichText::new(label).color(ui.visuals().hyperlink_color));
        } else {
            let matches = self.candidate_books();
            if !matches.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    for id in matches {
                        if ui
                            .small_button(CHINESE_BOOK_NAMES[id as usize - 1])
                            .clicked()
                        {
                            self.bible.set_location(id, 1);
                            let trimmed = self.quick.trim_start();
                            let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
                            self.quick = format!(
                                "{} {}",
                                cp_bible::initials::CHINESE_BOOK_INITIALS[id as usize - 1],
                                trimmed[end..].trim_start()
                            );
                            input.request_focus();
                            self.deadline = Some(Instant::now() + delay);
                        }
                    }
                });
            }
        }
        ui.horizontal_wrapped(|ui| {
            if ui.toggle_value(&mut self.continuous, "连续上屏").changed() {
                self.bible.forget_command();
                if self.continuous && !self.quick.trim().is_empty() {
                    self.deadline = Some(Instant::now() + delay);
                    ui.ctx().request_repaint_after(delay);
                }
            }
            if ui.button("清除经文").clicked() {
                self.clear_passages();
            }
            if ui.button("样式设置").clicked() {
                self.style_open = true;
            }
            ui.separator();
            if ui
                .add_enabled(!self.bible.rows.is_empty(), Button::new("立即上屏"))
                .clicked()
            {
                if self.deadline.is_some() && !self.quick.trim().is_empty() {
                    self.apply_quick(true);
                } else {
                    self.go_bible();
                }
            }
            if ui
                .add_enabled(!self.bible.rows.is_empty(), Button::new("加入程序单"))
                .clicked()
            {
                self.add_schedule();
            }
        });
        ui.separator();
        if self.bible.modules.is_empty() {
            ui.label(if self.bible_load.is_some() {
                "正在加载圣经译本…"
            } else {
                "请在设置中添加包含 .spb 圣经文件的目录。"
            });
            return;
        }
        let width = ui.available_width();
        let book_w = (width * 0.21).clamp(65., 185.);
        let ch_w = (width * 0.13).clamp(46., 104.);
        let h = ui.available_height().max(60.);
        let mut book = None;
        let mut chapter = None;
        let mut verse = None;
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(book_w, h), Layout::top_down(egui::Align::Min), |ui| {
                ui.set_width(book_w);
                ui.strong("书卷");
                ui.separator();
                ScrollArea::vertical()
                    .id_salt("books")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for b in self.bible.primary().unwrap().books() {
                            if ui
                                .selectable_label(self.bible.book == b.book_id, &b.name)
                                .clicked()
                            {
                                book = Some(b.book_id);
                            }
                        }
                    });
            });
            ui.separator();
            ui.allocate_ui_with_layout(vec2(ch_w, h), Layout::top_down(egui::Align::Min), |ui| {
                ui.set_width(ch_w);
                ui.strong("章");
                ui.separator();
                ScrollArea::vertical()
                    .id_salt(("chapters", self.bible.book))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let count = self
                            .bible
                            .primary()
                            .and_then(|b| b.book(self.bible.book))
                            .map_or(0, |b| b.chapter_count);
                        for n in 1..=count {
                            if ui
                                .selectable_label(n == self.bible.chapter, format!("第 {n} 章"))
                                .clicked()
                            {
                                chapter = Some(n);
                            }
                        }
                    });
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.strong("经节");
                ui.separator();
                let selected = self.bible.selection.selected_rows();
                ScrollArea::vertical()
                    .id_salt(("verses", self.bible.book, self.bible.chapter))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (i, row) in self.bible.rows.iter().enumerate() {
                            let r = ui.add(
                                egui::Label::new(format!(
                                    "{}   {}",
                                    row.verse_number, row.verse_text
                                ))
                                .wrap()
                                .sense(egui::Sense::click())
                                .selectable(false),
                            );
                            if selected.contains(&i) {
                                ui.painter().rect_filled(
                                    r.rect,
                                    3,
                                    ui.visuals().selection.bg_fill.gamma_multiply(0.36),
                                );
                            }
                            if r.clicked() {
                                verse = Some(i);
                            }
                        }
                    });
            });
        });
        if let Some(b) = book {
            self.bible.set_location(b, 1);
        }
        if let Some(ch) = chapter {
            self.bible.set_location(self.bible.book, ch);
        }
        if let Some(row) = verse {
            let mods = ui.input(|i| i.modifiers);
            if mods.shift {
                self.bible.selection.shift_click_verse(row);
            } else if mods.ctrl || mods.mac_cmd {
                self.bible.selection.ctrl_click_verse(row);
            } else {
                self.bible.selection.click_verse(row);
            }
            self.deadline = None;
            if self.settings.selection_goes_live {
                self.go_bible();
            }
        }
    }

    fn candidate_books(&self) -> Vec<i32> {
        let token = self.quick.split_whitespace().next().unwrap_or_default();
        candidates(token)
            .into_iter()
            .filter(|&id| self.bible.primary().is_some_and(|b| b.book(id).is_some()))
            .collect()
    }

    fn sync_stack(&mut self) {
        let old = self.settings.bible_settings.effective_translations();
        let profile = self.settings.projection_settings.output_profiles.first();
        let selected: Vec<_> = self
            .bible
            .stack
            .iter()
            .map(|&i| {
                let file = self.bible.modules[i].filename();
                old.iter()
                    .find(|s| s.file_name == file)
                    .or_else(|| {
                        profile?
                            .bible_settings
                            .translations
                            .iter()
                            .find(|s| s.file_name == file)
                    })
                    .cloned()
                    .unwrap_or_else(|| BibleTranslationSettings {
                        file_name: file.into(),
                        ..Default::default()
                    })
            })
            .collect();
        let bible = &mut self.settings.bible_settings;
        bible.primary_bible = selected
            .first()
            .map(|s| s.file_name.clone())
            .unwrap_or_default();
        bible.secondary_bible = selected
            .get(1)
            .map(|s| s.file_name.clone())
            .unwrap_or_default();
        bible.translations = selected;
        self.save();
    }

    pub fn style_window(&mut self, ctx: &Context) {
        let mut open = self.style_open;
        let mut changed = false;
        egui::Window::new("经文样式设置")
            .open(&mut open)
            .default_width(450.)
            .vscroll(true)
            .show(ctx, |ui| {
                let mut styles = self.settings.active_bible_styles();
                for &i in &self.bible.stack {
                    let module = &self.bible.modules[i];
                    let mut style = styles
                        .iter()
                        .find(|s| s.file_name == module.filename())
                        .cloned()
                        .unwrap_or_else(|| BibleTranslationSettings {
                            file_name: module.filename().into(),
                            ..Default::default()
                        });
                    ui.push_id(module.filename(), |ui| {
                        ui.strong(module.title());
                        let previous = style.clone();
                        ui.horizontal_wrapped(|ui| {
                            ui.label("正文字体");
                            font_select(ui, "body", &mut style.text_font_type, &self.fonts);
                            ui.checkbox(&mut style.text_bold, "粗体");
                        });
                        ui.horizontal(|ui| {
                            ui.label("经文字号");
                            ui.add(DragValue::new(&mut style.text_font_size).range(12..=160));
                            ui.label("文字颜色");
                            color_select(ui, &mut style.text_color);
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label("出处字体");
                            font_select(ui, "ref", &mut style.reference_font_type, &self.fonts);
                            ui.checkbox(&mut style.reference_bold, "粗体");
                        });
                        ui.horizontal(|ui| {
                            ui.label("出处字号");
                            ui.add(DragValue::new(&mut style.reference_font_size).range(10..=140));
                            ui.label("出处颜色");
                            color_select(ui, &mut style.reference_color);
                        });
                        if style != previous {
                            if !styles.iter().any(|s| s.file_name == style.file_name) {
                                styles.push(style.clone());
                            }
                            if let Some(s) =
                                styles.iter_mut().find(|s| s.file_name == style.file_name)
                            {
                                *s = style.clone();
                            }
                            if let Some(profile) = self
                                .settings
                                .projection_settings
                                .output_profiles
                                .first_mut()
                            {
                                let list = &mut profile.bible_settings.translations;
                                if let Some(s) =
                                    list.iter_mut().find(|s| s.file_name == style.file_name)
                                {
                                    s.text_color = style.text_color.clone();
                                    s.reference_color = style.reference_color.clone();
                                    s.text_font_type = style.text_font_type.clone();
                                    s.reference_font_type = style.reference_font_type.clone();
                                    s.text_font_size = style.text_font_size;
                                    s.reference_font_size = style.reference_font_size;
                                    s.text_bold = style.text_bold;
                                    s.reference_bold = style.reference_bold;
                                } else {
                                    list.push(style.clone());
                                }
                            }
                            changed = true;
                        }
                        ui.separator();
                    });
                }
                if changed {
                    self.settings.bible_settings.translations = styles;
                }
                let b = &mut self.settings.bible_settings;
                let mut line = b.continuous_line_spacing_percent.unwrap_or(135);
                let mut gap = b.continuous_passage_spacing.unwrap_or(24);
                ui.horizontal_wrapped(|ui| {
                    ui.label("连续经文行距");
                    let a = ui
                        .add(DragValue::new(&mut line).range(60..=300).suffix("%"))
                        .changed();
                    ui.label("段落间距");
                    let c = ui.add(DragValue::new(&mut gap).range(0..=160)).changed();
                    if a || c {
                        b.continuous_line_spacing_percent = Some(line);
                        b.continuous_passage_spacing = Some(gap);
                        if let Some(p) = self
                            .settings
                            .projection_settings
                            .output_profiles
                            .first_mut()
                        {
                            p.bible_settings.continuous_line_spacing_percent = Some(line);
                            p.bible_settings.continuous_passage_spacing = Some(gap);
                        }
                        changed = true;
                    }
                });
            });
        self.style_open = open;
        if changed {
            self.save();
            ctx.request_repaint();
        }
    }
}
fn font_select(ui: &mut Ui, id: &str, name: &mut String, fonts: &appearance::Fonts) {
    ComboBox::from_id_salt(id)
        .selected_text(name.as_str())
        .show_ui(ui, |ui| {
            for family in fonts.names() {
                ui.selectable_value(name, family.clone(), family);
            }
        });
}
fn color_select(ui: &mut Ui, value: &mut String) {
    let mut c = appearance::color(value, Color32::WHITE);
    if ui.color_edit_button_srgba(&mut c).changed() {
        *value = format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b());
    }
}
