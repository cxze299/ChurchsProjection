use crate::{app::App, media::extension, output::Content};
use cp_core::ResourceDirectory;
use egui::{vec2, Button, ComboBox, Key, ScrollArea, TextEdit, Ui};

impl App {
    pub fn library_tab(&mut self, ui: &mut Ui) {
        ui.heading("诗歌与文稿");
        let old = (
            self.settings.preferred_presentation_format.clone(),
            self.settings.preferred_hymn_book.clone(),
        );
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
            ui.menu_button("选择资源 ▾", |ui| {
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
            });
        });
        if old.0 != self.settings.preferred_presentation_format
            || old.1 != self.settings.preferred_hymn_book
        {
            self.sync_filters();
        }
        let search = ui.add(
            TextEdit::singleline(&mut self.query)
                .desired_width(f32::INFINITY)
                .hint_text("输入诗歌编号、标题或拼音首字母"),
        );
        if self.focus_search {
            search.request_focus();
            self.focus_search = false;
        }
        if search.changed() {
            self.search();
        }
        let enter = search.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if enter {
            if let Some(path) = self.results.first().cloned() {
                self.open(ui.ctx(), &path, true);
            }
            search.surrender_focus();
        }
        ui.separator();
        if ui.available_width() < 540. {
            let h = (ui.available_height() * 0.35).clamp(70., 180.);
            self.result_list(ui, h);
            ui.separator();
            self.slide_preview(ui);
        } else {
            let width = (ui.available_width() * 0.36).min(400.);
            let height = ui.available_height();
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    vec2(width, height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(width);
                        self.result_list(ui, height);
                    },
                );
                ui.separator();
                ui.vertical(|ui| self.slide_preview(ui));
            });
        }
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
        }
    }
    fn slide_preview(&mut self, ui: &mut Ui) {
        if self.opening.is_some() {
            ui.spinner();
            ui.label("正在打开资源…");
            return;
        }
        let Some(page) = self.staged.clone() else {
            ui.strong("资源预览");
            ui.label("选择诗歌或文稿以预览页面。");
            return;
        };
        ui.strong(page.document.title());
        let width = ui
            .available_width()
            .min((ui.available_height() * 0.48).max(50.) * 16. / 9.);
        self.renderer.canvas(
            ui,
            vec2(width, width * 9. / 16.),
            &mut crate::output::Output {
                content: Content::Media(page.clone()),
                ..Default::default()
            },
            &self.settings,
            &self.fonts,
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(page.index > 0, Button::new("上一页"))
                .clicked()
            {
                self.step_page(-1);
            }
            ui.label(format!("{}/{}", page.index + 1, page.document.len()));
            if ui
                .add_enabled(page.index + 1 < page.document.len(), Button::new("下一页"))
                .clicked()
            {
                self.step_page(1);
            }
            if ui.button("投屏").clicked() {
                self.go_media();
            }
            if ui.button("加入程序单").clicked() {
                self.add_schedule();
            }
        });
        ui.separator();
        ScrollArea::vertical()
            .id_salt(("slide_list", page.document.path()))
            .auto_shrink([false, false])
            .show_rows(ui, 28., page.document.len(), |ui, rows| {
                for i in rows {
                    let label = format!("第 {} 页   {}", i + 1, page.document.page_label(i));
                    if ui
                        .add_sized(
                            [ui.available_width(), 28.],
                            egui::SelectableLabel::new(i == page.index, label),
                        )
                        .clicked()
                    {
                        self.step_page(i as isize - page.index as isize);
                    }
                }
            });
    }
}
