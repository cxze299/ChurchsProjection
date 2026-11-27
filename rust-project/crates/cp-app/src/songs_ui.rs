use crate::{
    app::App,
    appearance::GOLD,
    media::{Document, Page},
    output::Content,
    widgets::*,
};
use egui::{vec2, Button, Frame, Layout, RichText, ScrollArea, Sense, Stroke, Ui};
use std::{collections::BTreeSet, sync::Arc};

impl App {
    pub fn filter_songs(&mut self) {
        let text = self.song_query.trim().to_lowercase();
        let mut rows = self
            .library
            .songs
            .iter()
            .filter(|(_, s)| {
                (self.song_book.is_empty() || self.song_book == s.songbook)
                    && (text.is_empty()
                        || s.number.contains(&text)
                        || s.title.to_lowercase().contains(&text)
                        || cp_library::pinyin_match::matches(&s.title, &text))
            })
            .collect::<Vec<_>>();
        rows.sort_by(|(pa, a), (pb, b)| {
            (a.number.parse::<u32>().unwrap_or(u32::MAX), &a.title, pa).cmp(&(
                b.number.parse::<u32>().unwrap_or(u32::MAX),
                &b.title,
                pb,
            ))
        });
        self.song_results = rows.into_iter().map(|(path, _)| path.clone()).collect();
    }
    pub fn songs_tab(&mut self, ui: &mut Ui) {
        let w = ui.available_width();
        let h = ui.available_height();
        if w < 470. {
            self.song_table(ui, (h * 0.46).max(100.));
            self.stanzas(ui, ui.available_height());
        } else {
            let left = (w * 0.54).floor();
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    vec2(left, h),
                    Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(left);
                        self.song_table(ui, h);
                    },
                );
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    self.stanzas(ui, h);
                });
            });
        }
    }
    fn song_table(&mut self, ui: &mut Ui, h: f32) {
        let p = self.palette;
        card_frame(p).inner_margin(9).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let start = ui.cursor().top();
            let edit = well_edit(
                ui,
                p,
                "song_query",
                &mut self.song_query,
                "⌕  搜索歌名、编号或拼音",
                ui.available_width(),
            );
            if self.focus_search {
                edit.request_focus();
                self.focus_search = false;
            }
            let before = self.song_book.clone();
            egui::ComboBox::from_id_salt("songbook")
                .selected_text(if before.is_empty() {
                    "所有歌本"
                } else {
                    &before
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.song_book, String::new(), "所有歌本");
                    for book in self
                        .library
                        .songs
                        .values()
                        .map(|s| &s.songbook)
                        .filter(|s| !s.is_empty())
                        .collect::<BTreeSet<_>>()
                    {
                        ui.selectable_value(&mut self.song_book, book.clone(), book);
                    }
                });
            if edit.changed() || before != self.song_book {
                self.filter_songs();
            }
            let w = ui.available_width() - 8.;
            let num = (w * 0.14).max(28.);
            let book = (w * 0.23).max(32.);
            let title = (w - num - book - 42.).max(20.);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.;
                for (width, text) in [(num, "编号"), (title, "歌名"), (book, "歌本"), (24., "＋")]
                {
                    ui.add_sized(
                        [width, 21.],
                        egui::Label::new(RichText::new(text).small().strong().color(p.muted))
                            .halign(egui::Align::Min)
                            .truncate(),
                    );
                }
            });
            let mut open = None;
            let mut add = None;
            let height = (h - (ui.cursor().top() - start) - 22.).max(36.);
            ScrollArea::vertical()
                .id_salt("song_table")
                .auto_shrink([false, false])
                .max_height(height)
                .show_rows(ui, 28., self.song_results.len(), |ui, range| {
                    for i in range {
                        let path = &self.song_results[i];
                        let song = &self.library.songs[path];
                        let selected = self
                            .staged
                            .as_ref()
                            .is_some_and(|s| s.document.path() == path);
                        let r = Frame::new()
                            .fill(if selected {
                                p.selection()
                            } else {
                                egui::Color32::TRANSPARENT
                            })
                            .corner_radius(7)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 2.;
                                    for (width, label) in [
                                        (num, &song.number),
                                        (title, &song.title),
                                        (book, &song.songbook),
                                    ] {
                                        let r = ui.add_sized(
                                            [width, 28.],
                                            egui::Label::new(RichText::new(label).size(12.).color(
                                                if selected { p.selected_ink() } else { p.ink },
                                            ))
                                            .halign(egui::Align::Min)
                                            .truncate()
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );
                                        if r.clicked() || r.double_clicked() {
                                            open = Some((path.clone(), r.double_clicked()));
                                        }
                                        r.on_hover_text(label);
                                    }
                                    if ui
                                        .add(
                                            Button::new(RichText::new("＋").color(p.secondary))
                                                .frame(false)
                                                .min_size(vec2(22., 24.)),
                                        )
                                        .on_hover_text("加入程序单")
                                        .clicked()
                                    {
                                        add = Some(path.clone());
                                    }
                                });
                            });
                        if r.response.hovered() {
                            ui.painter().rect_stroke(
                                r.response.rect,
                                7,
                                Stroke::new(1.0_f32, p.card_edge()),
                                egui::StrokeKind::Inside,
                            );
                        }
                    }
                });
            if let Some(path) = add {
                let song = self.library.songs[&path].clone();
                let slides = song.lyric_slides();
                if slides.is_empty() {
                    self.message = "诗歌文件中没有歌词".into();
                } else {
                    self.enqueue(Content::Media(Page {
                        document: Arc::new(Document::Song { song, slides }),
                        index: 0,
                    }));
                }
            }
            if let Some((path, live)) = open {
                self.open(ui.ctx(), &path, live || self.settings.selection_goes_live);
            }
        });
    }
    fn stanzas(&mut self, ui: &mut Ui, h: f32) {
        let p = self.palette;
        card_frame(p).inner_margin(9).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height((h - 22.).max(60.));
            ui.add_enabled_ui(self.staged.is_some(), |ui| {
                ui.horizontal_wrapped(|ui| {
                    if schedule_button(ui, p).clicked() {
                        self.add_schedule();
                    }
                    if primary(ui, p, "▶ 立即上屏").clicked() {
                        self.go_media();
                    }
                });
            });
            let Some(page) = self.staged.clone() else {
                ui.weak("选择诗歌以预览歌词。");
                return;
            };
            let Document::Song { song, slides } = &*page.document else {
                return;
            };
            ui.label(RichText::new(song.display_title()).strong());
            ScrollArea::vertical()
                .id_salt(("stanzas", &song.source_file))
                .auto_shrink([false, true])
                .max_height((h - 105.).max(50.))
                .show(ui, |ui| {
                    for (i, slide) in slides.iter().enumerate() {
                        let active = i == page.index;
                        let r = card_frame(p)
                            .fill(if active { p.selection() } else { p.surface })
                            .stroke(Stroke::new(
                                1.0_f32,
                                if active { p.primary } else { p.card_edge() },
                            ))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                let section = if slide.section.is_empty() {
                                    format!("VERSE {}", i + 1)
                                } else {
                                    slide.section.to_uppercase()
                                };
                                Frame::new()
                                    .fill(GOLD.gamma_multiply(0.16))
                                    .corner_radius(8)
                                    .inner_margin(egui::Margin::symmetric(6, 2))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(section).size(10.5).strong().color(
                                            if p.dark {
                                                GOLD
                                            } else {
                                                egui::Color32::from_rgb(115, 78, 16)
                                            },
                                        ));
                                    });
                                ui.add(
                                    egui::Label::new(slide.lines.join("\n"))
                                        .wrap()
                                        .selectable(false),
                                );
                            })
                            .response
                            .interact(Sense::click());
                        if r.clicked() || r.double_clicked() {
                            self.step_page(i as isize - page.index as isize);
                            if r.double_clicked() {
                                self.go_media();
                            }
                        }
                    }
                });
        });
    }
}
