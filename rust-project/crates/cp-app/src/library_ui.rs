use crate::{
    app::App,
    media::{extension, Page},
    output::{Content, Output},
    widgets::*,
};
use cp_core::ResourceDirectory;
use egui::{vec2, Button, ComboBox, Key, ScrollArea, Ui};
impl App {
    pub fn library_tab(&mut self, ui: &mut Ui) {
        let p = self.palette;
        let old = (
            self.settings.preferred_presentation_format.clone(),
            self.settings.preferred_hymn_book.clone(),
        );
        card_frame(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.scope(|ui| {
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = p.primary;
                    ui.visuals_mut().widgets.hovered.weak_bg_fill = p.primary;
                    ui.visuals_mut().widgets.active.weak_bg_fill = p.primary;
                    ui.spacing_mut().button_padding = vec2(10., 7.);
                    ui.menu_button(
                        egui::RichText::new("▣ 选择资源 ▾")
                            .color(p.on_primary)
                            .strong(),
                        |ui| {
                            if ui.button("添加文件…").clicked() {
                                ui.close_menu();
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("诗歌与文稿", cp_library::discovery::EXTENSIONS)
                                    .pick_file()
                                {
                                    if !self.added.contains(&path) {
                                        self.added.push(path.clone());
                                        self.settings.extra.insert(
                                            "nativePresentationFiles".into(),
                                            serde_json::json!(&self.added),
                                        );
                                        self.save();
                                    }
                                    self.query.clear();
                                    self.settings.preferred_hymn_book.clear();
                                    self.refresh(ui.ctx(), ResourceDirectory::Presentations);
                                    self.open(ui.ctx(), &path, self.settings.selection_goes_live);
                                }
                            }
                            if ui.button("添加目录…").clicked() {
                                ui.close_menu();
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.directory_change(
                                        ui.ctx(),
                                        ResourceDirectory::Presentations,
                                        path.to_string_lossy().into_owned(),
                                        false,
                                    );
                                }
                            }
                        },
                    );
                });
                if let Some(page) = &self.staged {
                    ui.add(egui::Label::new(page.document.title()).truncate());
                }
                ui.add_enabled_ui(self.staged.is_some(), |ui| {
                    if schedule_button(ui, p).clicked() {
                        self.add_schedule();
                    }
                    if primary(ui, p, "▶ 立即上屏").clicked() {
                        self.go_media();
                    }
                });
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("投影");
                self.display_selector(ui, "library_display");
                ui.label("默认格式");
                let format = &mut self.settings.preferred_presentation_format;
                let text = if matches!(format.as_str(), "" | "all") {
                    "全部".into()
                } else {
                    format.to_uppercase()
                };
                ComboBox::from_id_salt("format")
                    .width(72.)
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        for (f, label) in [
                            ("", "全部"),
                            ("pptx", "PPTX"),
                            ("pdf", "PDF"),
                            ("docx", "DOCX"),
                            ("song", "SONG"),
                        ] {
                            ui.selectable_value(format, f.into(), label);
                        }
                    });
                ui.label("默认诗集");
                let book = &mut self.settings.preferred_hymn_book;
                ComboBox::from_id_salt("hymn_book")
                    .selected_text(if book.is_empty() {
                        "所有诗集"
                    } else {
                        book.as_str()
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(book, String::new(), "所有诗集");
                        for name in &self.library.books {
                            ui.selectable_value(book, name.clone(), name);
                        }
                    });
            });
            let (i, n) = self
                .staged
                .as_ref()
                .map_or((0, 0), |p| (p.index + 1, p.document.len()));
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(i > 1, |ui| {
                    if filter_chip(ui, p, false, "◀")
                        .on_hover_text("上一页")
                        .clicked()
                    {
                        self.step_page(-1);
                    }
                });
                ui.add_enabled_ui(n > 0, |ui| {
                    if filter_chip(ui, p, true, "▶")
                        .on_hover_text("播放当前页")
                        .clicked()
                    {
                        self.go_media();
                    }
                });
                ui.add_enabled_ui(i < n, |ui| {
                    if filter_chip(ui, p, false, "▶▏")
                        .on_hover_text("下一页")
                        .clicked()
                    {
                        self.step_page(1);
                    }
                });
                ui.small(format!("第 {i} / {n} 页"));
            });
        });
        if old.0 != self.settings.preferred_presentation_format
            || old.1 != self.settings.preferred_hymn_book
        {
            self.sync_filters();
            self.browse_results = true;
        }
        card_frame(p).inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let w = (ui.available_width() - 78.).max(60.);
                let search = well_edit(
                    ui,
                    p,
                    "presentation_query",
                    &mut self.query,
                    "⌕  输入诗歌编号、标题或拼音首字母",
                    w,
                );
                if self.focus_search {
                    search.request_focus();
                    self.focus_search = false;
                }
                if search.changed() {
                    self.search();
                    self.browse_results = true;
                }
                if search.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    if let Some(path) = self.results.first().cloned() {
                        self.open(ui.ctx(), &path, true);
                        self.browse_results = false;
                    }
                }
                if filter_chip(ui, p, self.browse_results, "浏览 ▾").clicked() {
                    self.browse_results = !self.browse_results;
                }
            });
            if self.browse_results {
                self.result_list(ui, 138.);
            }
        });
        card_frame(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(ui.available_height());
            let Some(page) = self.staged.clone() else {
                section_header_label(ui, p, "文稿预览");
                ui.weak(if self.opening.is_some() {
                    "正在打开文稿…"
                } else {
                    "选择资源或浏览诗集以预览页面。"
                });
                return;
            };
            thumbnail_grid(
                ui,
                egui::Id::new(("slide_grid", page.document.path())),
                page.document.len(),
                |ui, index, w| {
                    let slide = Page {
                        document: page.document.clone(),
                        index,
                    };
                    let canvas = self.renderer.canvas(
                        ui,
                        vec2(w, w * 9. / 16.),
                        &mut Output {
                            content: Content::Media(slide),
                            ..Default::default()
                        },
                        &self.settings,
                        &self.fonts,
                    );
                    let r = thumbnail_caption(
                        ui,
                        p,
                        canvas,
                        &format!("第 {} 页", index + 1),
                        index == page.index,
                    )
                    .on_hover_text(page.document.page_label(index));
                    if r.clicked() || r.double_clicked() {
                        self.step_page(index as isize - page.index as isize);
                        if r.double_clicked() {
                            self.go_media();
                        }
                    }
                },
            );
        });
    }
    fn result_list(&mut self, ui: &mut Ui, height: f32) {
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("资源 · {}", self.results.len()));
            if ui
                .add_enabled(!self.results.is_empty(), Button::new("投屏"))
                .on_hover_text("放映首个搜索结果")
                .clicked()
            {
                if let Some(p) = self.results.first().cloned() {
                    self.open(ui.ctx(), &p, true);
                }
            }
        });
        let mut area = ScrollArea::vertical()
            .id_salt("hymn_results")
            .max_height((height - 36.).max(30.))
            .auto_shrink([false, false]);
        if std::mem::take(&mut self.reset_results) {
            area = area.vertical_scroll_offset(0.);
        }
        let mut pick = None;
        area.show_rows(ui, 30., self.results.len(), |ui, rows| {
            for i in rows {
                let path = &self.results[i];
                let title = format!(
                    "{}   ·   {}",
                    self.library.title(path),
                    extension(path).to_uppercase()
                );
                let active = self
                    .staged
                    .as_ref()
                    .is_some_and(|p| p.document.path() == path);
                let r = ui
                    .add_sized(
                        [ui.available_width(), 30.],
                        egui::SelectableLabel::new(active, title),
                    )
                    .on_hover_text(path.to_string_lossy());
                if r.clicked() {
                    pick = Some(path.clone());
                }
            }
        });
        if let Some(path) = pick {
            self.open(ui.ctx(), &path, self.settings.selection_goes_live);
            self.browse_results = false;
        }
    }
}
