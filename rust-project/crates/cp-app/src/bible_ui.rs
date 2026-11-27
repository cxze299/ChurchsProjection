use crate::{app::App, appearance, widgets::*};
use cp_bible::{initials::candidates, CHINESE_BOOK_NAMES};
use cp_core::BibleTranslationSettings;
use egui::{vec2, Color32, ComboBox, Context, DragValue, Key, Layout, ScrollArea, Ui};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SearchMode {
    #[default]
    Auto,
    Reference,
    Text,
}
impl SearchMode {
    fn label(self) -> &'static str {
        match self {
            Self::Auto => "自动",
            Self::Reference => "引用",
            Self::Text => "全文",
        }
    }
}
#[derive(Clone, Copy)]
enum VerseAction {
    Select,
    Live,
    Schedule,
    Copy,
}
impl App {
    pub fn submit_bible(&mut self, explicit: bool) {
        self.deadline = None;
        if self.quick.trim().is_empty() {
            self.hits = None;
            return;
        }
        let Some(bible) = self.bible.primary() else {
            return;
        };
        if self.search_mode == SearchMode::Text
            || (self.search_mode == SearchMode::Auto
                && bible.quick_passage_references(&self.quick).is_none()
                && self.candidate_books().is_empty())
        {
            self.hits = Some(bible.search_keywords(
                &self.quick,
                false,
                if self.scoped {
                    cp_bible::SearchScope::Book(self.bible.book)
                } else {
                    cp_bible::SearchScope::All
                },
            ));
        } else {
            self.hits = None;
            self.apply_quick(explicit);
        }
    }
    fn bible_search(&mut self, ui: &mut Ui) {
        let p = self.palette;
        card_frame(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let input = ui
                .horizontal_wrapped(|ui| {
                    let w = (ui.available_width() - 194.).max(85.);
                    let input = well_edit(
                        ui,
                        p,
                        "quick_bible",
                        &mut self.quick,
                        "⌕  搜索经文或输入引用，如 csj 1 1 3",
                        w,
                    );
                    if filter_chip(ui, p, true, self.search_mode.label())
                        .on_hover_text("切换检索模式")
                        .clicked()
                    {
                        self.search_mode = match self.search_mode {
                            SearchMode::Auto => SearchMode::Reference,
                            SearchMode::Reference => SearchMode::Text,
                            SearchMode::Text => SearchMode::Auto,
                        };
                        self.submit_bible(false);
                    }
                    let old = self.scoped;
                    ComboBox::from_id_salt("bible_scope")
                        .width(76.)
                        .selected_text(if self.scoped {
                            "当前书卷"
                        } else {
                            "全本圣经"
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.scoped, false, "全本圣经");
                            ui.selectable_value(&mut self.scoped, true, "当前书卷");
                        });
                    if old != self.scoped {
                        self.submit_bible(false);
                    }
                    let submit = raised_action_button(
                        ui,
                        p,
                        "⌕",
                        p.primary,
                        p.on_primary,
                        vec2(42., 36.),
                        "搜索",
                    )
                    .clicked();
                    (input, submit)
                })
                .inner;
            let (input, submit) = input;
            if self.focus_search {
                input.request_focus();
                self.focus_search = false;
            }
            let delay = Duration::from_millis(300);
            if input.changed() {
                self.deadline = Some(Instant::now() + delay);
                self.hits = None;
                if self.quick.is_empty() {
                    self.bible.forget_command();
                }
                if self.search_mode != SearchMode::Text
                    && self.quick.trim().chars().all(|c| c.is_ascii_alphabetic())
                {
                    let matches = self.candidate_books();
                    if matches.len() == 1 {
                        self.bible.set_location(matches[0], 1);
                    }
                }
            }
            if submit || (input.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))) {
                self.submit_bible(true);
                input.request_focus();
            } else if let Some(t) = self.deadline {
                if Instant::now() >= t {
                    self.submit_bible(false);
                } else {
                    ui.ctx()
                        .request_repaint_after(t.saturating_duration_since(Instant::now()));
                }
            }
            if self.search_mode != SearchMode::Text && !self.quick.trim().is_empty() {
                if let Some(label) = cp_bible::quick_passage_label(&self.quick) {
                    ui.colored_label(p.primary, label);
                } else if self.quick.trim().chars().all(|c| c.is_ascii_alphabetic()) {
                    ui.horizontal_wrapped(|ui| {
                        for id in self.candidate_books() {
                            if filter_chip(ui, p, false, CHINESE_BOOK_NAMES[id as usize - 1])
                                .clicked()
                            {
                                self.bible.set_location(id, 1);
                                self.quick = format!(
                                    "{} ",
                                    cp_bible::initials::CHINESE_BOOK_INITIALS[id as usize - 1]
                                );
                                input.request_focus();
                            }
                        }
                    });
                }
            }
            let mut pick = None;
            if let Some(hits) = &self.hits {
                ui.horizontal(|ui| {
                    ui.small(format!("{} 条结果", hits.len()));
                });
                ScrollArea::vertical()
                    .id_salt("scripture_hits")
                    .max_height(112.)
                    .show_rows(ui, 26., hits.len(), |ui, rows| {
                        for i in rows {
                            if ui
                                .add(
                                    egui::Label::new(&hits[i].verse_text)
                                        .truncate()
                                        .sense(egui::Sense::click()),
                                )
                                .on_hover_text(&hits[i].verse_text)
                                .clicked()
                            {
                                pick = Some(hits[i].clone());
                            }
                        }
                    });
            }
            if let Some(hit) = pick {
                if let Some(id) = self
                    .bible
                    .primary()
                    .and_then(|b| b.books().iter().find(|b| b.name == hit.book))
                    .map(|b| b.book_id)
                {
                    self.bible
                        .select_numbers(id, hit.chapter, hit.verse, hit.verse);
                    if self.settings.selection_goes_live {
                        self.go_bible();
                    }
                }
                self.hits = None;
            }
        });
    }
    fn translations(&mut self, ui: &mut Ui) {
        let mut stack = self.bible.stack.clone();
        {
            let label = self
                .bible
                .primary()
                .map(|b| b.abbreviation())
                .unwrap_or("请选择");
            ComboBox::from_id_salt("primary_bible")
                .width(52.)
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
            ui.menu_button(
                format!("对照 {} ▾", stack.len().saturating_sub(1)),
                |ui| {
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
                },
            );
            for &i in stack.iter().skip(1) {
                ui.small(self.bible.modules[i].abbreviation());
            }
        }
        if stack != self.bible.stack {
            self.bible.stack = stack;
            self.sync_stack();
            self.hits = None;
            self.bible.set_location(self.bible.book, self.bible.chapter);
            self.bible.forget_command();
        }
    }
    pub fn bible_tab(&mut self, ui: &mut Ui) {
        self.bible_search(ui);
        let p = self.palette;
        let w = ui.available_width();
        let h = ui.available_height().max(120.);
        let book_w = (self.bible_widths[0] * w).clamp(70., (w * 0.42).max(70.));
        let ch_w = (self.bible_widths[1] * w).clamp(46., (w * 0.25).max(46.));
        let verse_w = (w - book_w - ch_w - 16.).max(75.);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.;
            for (col, width) in [book_w, ch_w].into_iter().enumerate() {
                ui.allocate_ui_with_layout(
                    vec2(width, h),
                    Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(width);
                        card_frame(p)
                            .inner_margin(egui::Margin::symmetric(5, 10))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.set_min_height((h - 22.).max(20.));
                                section_header_label(ui, p, if col == 0 { "书卷" } else { "章" });
                                let mut picked = None;
                                ScrollArea::vertical()
                                    .id_salt((
                                        "bible_list",
                                        col,
                                        if col == 1 { self.bible.book } else { 0 },
                                    ))
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.spacing_mut().item_spacing.y = 1.;
                                        let Some(bible) = self.bible.primary() else {
                                            return;
                                        };
                                        if col == 0 {
                                            for b in bible.books() {
                                                if list_row(
                                                    ui,
                                                    p,
                                                    b.book_id == self.bible.book,
                                                    &b.name,
                                                    false,
                                                )
                                                .clicked()
                                                {
                                                    picked = Some(b.book_id);
                                                }
                                            }
                                        } else if let Some(book) = bible.book(self.bible.book) {
                                            for n in 1..=book.chapter_count {
                                                if list_row(
                                                    ui,
                                                    p,
                                                    n == self.bible.chapter,
                                                    &n.to_string(),
                                                    true,
                                                )
                                                .clicked()
                                                {
                                                    picked = Some(n);
                                                }
                                            }
                                        }
                                    });
                                if let Some(n) = picked {
                                    self.deadline = None;
                                    self.bible.forget_command();
                                    self.hits = None;
                                    if col == 0 {
                                        self.bible.set_location(n, 1);
                                    } else {
                                        self.bible.set_location(self.bible.book, n);
                                    }
                                }
                            });
                    },
                );
                let (rect, r) = ui.allocate_exact_size(vec2(8., h), egui::Sense::drag());
                r.clone()
                    .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(rect.center(), vec2(2., 32.)),
                    1,
                    p.muted.gamma_multiply(0.22),
                );
                if r.dragged() {
                    let max = if col == 0 {
                        (w * 0.42).max(70.)
                    } else {
                        (w * 0.25).max(46.)
                    };
                    self.bible_widths[col] =
                        (width + r.drag_delta().x).clamp(if col == 0 { 70. } else { 46. }, max) / w.max(1.);
                }
            }
            ui.allocate_ui_with_layout(
                vec2(verse_w, h),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(verse_w);
                    ui.spacing_mut().item_spacing.x = 5.;
                    let history_h = if self.history_open {
                        (h * 0.18).clamp(58., 120.)
                    } else {
                        35.
                    };
                    let list_h = (h - history_h - 6.).max(70.);
                    card_frame(p).inner_margin(8).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let top = ui.cursor().top();
                        self.verse_header(ui);
                        let rows_h = (list_h - (ui.cursor().top() - top) - 18.).max(26.);
                        if self.bible.modules.is_empty() {
                            ui.weak(if self.bible_load.is_some() {
                                "正在加载圣经译本…"
                            } else {
                                "请在设置中添加包含 .spb 圣经文件的目录。"
                            });
                        }
                        self.verse_rows(ui, rows_h);
                    });
                    self.bible_history(ui, history_h);
                },
            );
        });
    }
    fn verse_header(&mut self, ui: &mut Ui) {
        let p = self.palette;
        let compact = ui.available_width() < 650.;
        ui.horizontal_wrapped(|ui| {
            section_header_label(ui, p, "经节");
            self.translations(ui);
            if filter_chip(ui, p, self.continuous, "连续上屏").clicked() {
                self.continuous = !self.continuous;
                self.bible.forget_command();
                if self.continuous && !self.quick.is_empty() {
                    self.deadline = Some(Instant::now());
                }
            }
            if filter_chip(ui, p, false, if compact { "样式" } else { "样式设置" })
                .on_hover_text("样式设置")
                .clicked()
            {
                self.style_open = true;
            }
            if filter_chip(ui, p, false, if compact { "清除" } else { "清除经文" })
                .on_hover_text("清除经文")
                .clicked()
            {
                self.clear_passages();
                self.hits = None;
            }
            ui.add_enabled_ui(!self.bible.rows.is_empty(), |ui| {
                if raised_action_button(
                    ui,
                    p,
                    if compact { "＋" } else { "加入程序单" },
                    p.secondary,
                    p.on_secondary,
                    vec2(34., 34.),
                    "加入程序单",
                )
                .clicked()
                {
                    self.add_schedule();
                }
                if raised_action_button(
                    ui,
                    p,
                    if compact { "▶" } else { "立即上屏" },
                    p.primary,
                    p.on_primary,
                    vec2(34., 34.),
                    "立即上屏",
                )
                .clicked()
                {
                    if self.deadline.is_some() {
                        self.submit_bible(true);
                    } else {
                        self.go_bible();
                    }
                }
            });
        });
    }
    fn verse_rows(&mut self, ui: &mut Ui, height: f32) {
        let p = self.palette;
        let selected = self.bible.selection.selected_rows();
        let mut action = None;
        ScrollArea::vertical()
            .max_height(height)
            .auto_shrink([false, false])
            .id_salt(("verses", self.bible.book, self.bible.chapter))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 1.;
                for (i, v) in self.bible.rows.iter().enumerate() {
                    let on = selected.contains(&i);
                    let r = row(ui, p, on, false, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.label(
                                bold(v.verse_number.to_string())
                                    .strong()
                                    .color(p.primary),
                            );
                            ui.add(
                                egui::Label::new(egui::RichText::new(&v.verse_text).color(if on {
                                    p.selected_ink()
                                } else {
                                    p.ink
                                }))
                                .selectable(false)
                                .wrap(),
                            );
                        });
                    });
                    if r.clicked() || r.double_clicked() {
                        action = Some((
                            i,
                            if r.double_clicked() {
                                VerseAction::Live
                            } else {
                                VerseAction::Select
                            },
                            false,
                        ));
                    }
                    r.context_menu(|ui| {
                        for (label, a) in [
                            ("立即上屏", VerseAction::Live),
                            ("加入程序单", VerseAction::Schedule),
                            ("复制经文", VerseAction::Copy),
                        ] {
                            if ui.button(label).clicked() {
                                action = Some((i, a, true));
                                ui.close_menu();
                            }
                        }
                    });
                }
            });
        if let Some((i, action, menu)) = action {
            self.deadline = None;
            if menu {
                if !selected.contains(&i) {
                    let n = self.bible.rows[i].verse_number;
                    self.bible
                        .select_numbers(self.bible.book, self.bible.chapter, n, n);
                }
            } else if !matches!(action, VerseAction::Live) {
                let mods = ui.input(|i| i.modifiers);
                if mods.shift {
                    self.bible.selection.shift_click_verse(i);
                } else if mods.ctrl || mods.mac_cmd {
                    self.bible.selection.ctrl_click_verse(i);
                } else {
                    self.bible.selection.click_verse(i);
                }
            }
            match action {
                VerseAction::Select if self.settings.selection_goes_live => self.go_bible(),
                VerseAction::Live => self.go_bible(),
                VerseAction::Schedule => self.add_schedule(),
                VerseAction::Copy => ui.ctx().copy_text(
                    self.bible
                        .selected()
                        .iter()
                        .map(|v| format!("{} {}", v.display_reference(), v.verse_text))
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
                _ => {}
            }
        }
    }
    fn bible_history(&mut self, ui: &mut Ui, height: f32) {
        let p = self.palette;
        card_frame(p).inner_margin(7).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(if self.history_open {
                            "▾ 历史记录"
                        } else {
                            "▸ 历史记录"
                        })
                        .frame(false),
                    )
                    .clicked()
                {
                    self.history_open = !self.history_open;
                }
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("×").on_hover_text("清空历史记录").clicked() {
                        self.history.clear();
                    }
                });
            });
            if self.history_open {
                let mut live = None;
                let mut select = None;
                ScrollArea::vertical()
                    .id_salt("bible_history_rows")
                    .auto_shrink([false, false])
                    .max_height((height - 43.).max(10.))
                    .show(ui, |ui| {
                        for (i, group) in self.history.iter().enumerate() {
                            if let Some(v) = group.first() {
                                let r = ui.add(
                                    egui::Label::new(format!(
                                        "{}   {}",
                                        v.display_reference(),
                                        v.verse_text
                                    ))
                                    .truncate()
                                    .selectable(false)
                                    .sense(egui::Sense::click()),
                                );
                                if r.clicked() {
                                    select = Some(i);
                                }
                                if r.double_clicked() {
                                    live = Some(i);
                                }
                            }
                        }
                    });
                if let Some(i) = select.or(live) {
                    self.bible.restore(&self.history[i]);
                    self.deadline = None;
                }
                if let Some(i) = live {
                    self.cancel_pending_live();
                    let verses = self.history[i].clone();
                    self.output.publish(crate::output::Content::Scripture {
                        continuous: !verses[0].continuous_passages.is_empty(),
                        verses,
                    });
                }
            }
        });
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
