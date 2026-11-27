use crate::{app::App, appearance::PALETTES, widgets::*};
use cp_core::{ResourceDirectory as Dir, ShortcutAction};
use egui::{Button, ComboBox, Context, ScrollArea, TextEdit, Ui};
use std::path::Path;

const SECTIONS: [(Dir, &str); 9] = [
    (Dir::Bible, "圣经 · BIBLE"),
    (Dir::Songs, "诗歌 · SONGS"),
    (Dir::Pictures, "图片 · PICTURES"),
    (Dir::LowerThird, "下三分之一字幕 · LOWER_THIRD"),
    (Dir::Presentations, "文稿 · PRESENTATIONS"),
    (Dir::Media, "媒体 · MEDIA"),
    (Dir::Schedule, "程序单 · SCHEDULE"),
    (Dir::Calendar, "日历 · CALENDAR"),
    (Dir::Signage, "电子公告 · SIGNAGE"),
];

impl App {
    pub fn settings_tab(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .id_salt("settings")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let p = self.palette;
                card_frame(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("外观与主题");
                    ui.horizontal_wrapped(|ui| {
                        for scheme in PALETTES {
                            let active = p.name == scheme.name;
                            let response = ui.add(
                                Button::new(
                                    egui::RichText::new(format!("●  {}", scheme.name))
                                        .color(scheme.primary)
                                        .strong(),
                                )
                                .fill(scheme.surface_card)
                                .stroke(egui::Stroke::new(
                                    if active { 2.0_f32 } else { 1.0_f32 },
                                    if active { p.primary } else { p.card_edge() },
                                ))
                                .min_size(egui::vec2(78., 32.))
                                .corner_radius(8),
                            );
                            if response.clicked() {
                                self.set_theme(ui.ctx(), scheme.name);
                            }
                        }
                    });
                    if switch(ui, p, &mut self.settings.selection_goes_live, "选中即上屏") {
                        self.save();
                    }
                    if switch(
                        ui,
                        p,
                        &mut self
                            .settings
                            .keyboard_shortcut_settings
                            .focus_search_on_tab_open,
                        "打开标签聚焦搜索",
                    ) {
                        self.save();
                    }
                });
                card_frame(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("资源目录");
                    let section = &mut self.directory_section;
                    ComboBox::from_id_salt("directory_category")
                        .selected_text(SECTIONS.iter().find(|(s, _)| s == section).unwrap().1)
                        .show_ui(ui, |ui| {
                            for (s, label) in SECTIONS {
                                ui.selectable_value(section, s, label);
                            }
                        });
                    let section = *section;
                    let primary = self.settings.primary_directory(section).to_owned();
                    let directories = self.settings.resource_directories(section);
                    if directories.is_empty() {
                        ui.label("尚未设置目录");
                    }
                    for dir in directories {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(&dir);
                            if dir == primary {
                                egui::Frame::new()
                                    .fill(p.secondary_container)
                                    .corner_radius(8)
                                    .inner_margin(egui::Margin::symmetric(6, 2))
                                    .show(ui, |ui| {
                                        ui.colored_label(p.on_secondary_container, "主目录");
                                    });
                            } else {
                                if ui.small_button("设为主目录").clicked() {
                                    self.directory_change(ui.ctx(), section, dir.clone(), true);
                                }
                                if ui.small_button("移除").clicked() {
                                    self.settings
                                        .remove_additional_resource_directory(section, &dir);
                                    self.after_directory(ui.ctx(), section);
                                }
                            }
                        });
                    }
                    ui.add(
                        TextEdit::singleline(&mut self.directory_text)
                            .hint_text("输入目录路径")
                            .desired_width(f32::INFINITY),
                    );
                    ui.horizontal_wrapped(|ui| {
                        if crate::widgets::primary(ui, p, "浏览目录…").clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.directory_change(
                                    ui.ctx(),
                                    section,
                                    p.to_string_lossy().into_owned(),
                                    false,
                                );
                            }
                        }
                        if ui
                            .add_enabled(
                                !self.directory_text.trim().is_empty(),
                                Button::new("添加目录"),
                            )
                            .clicked()
                        {
                            self.directory_change(
                                ui.ctx(),
                                section,
                                self.directory_text.clone(),
                                false,
                            );
                        }
                    });
                });
                card_frame(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("键盘快捷键");
                    egui::Grid::new("shortcut_table")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            for &action in ShortcutAction::ALL {
                                ui.label(action.display_name().split(" (").next().unwrap());
                                ui.push_id(action.key_name(), |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        let keys = crate::keyboard::chords(
                                            &self.settings.keyboard_shortcut_settings,
                                            action,
                                        );
                                        let label = if self.record == Some(action) {
                                            "请按下新组合键".into()
                                        } else if keys.is_empty() {
                                            "未设置".into()
                                        } else {
                                            keys.iter()
                                                .map(crate::keyboard::label)
                                                .collect::<Vec<_>>()
                                                .join(" / ")
                                        };
                                        if raised_action_button(
                                            ui,
                                            p,
                                            &label,
                                            p.tertiary,
                                            p.on_tertiary,
                                            egui::vec2(46., 26.),
                                            "录制并替换快捷键",
                                        )
                                        .clicked()
                                        {
                                            self.record = Some(action);
                                        }
                                        if self.record == Some(action)
                                            && ui.small_button("取消录制").clicked()
                                        {
                                            self.record = None;
                                        }
                                        if ui.small_button("清除").clicked() {
                                            self.record = None;
                                            self.settings
                                                .keyboard_shortcut_settings
                                                .clear_action(action);
                                            self.save();
                                        }
                                        if ui.small_button("恢复默认").clicked() {
                                            self.record = None;
                                            self.settings
                                                .keyboard_shortcut_settings
                                                .reset_action(action);
                                            self.save();
                                        }
                                    });
                                });
                                ui.end_row();
                            }
                        });
                });
                card_frame(p).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let id = egui::Id::new("about_icon");
                        let texture = ui.ctx().data_mut(|d| d.get_temp::<egui::TextureHandle>(id));
                        let texture = texture.or_else(|| {
                            let im =
                                image::load_from_memory(include_bytes!("../assets/app-icon.png"))
                                    .ok()?
                                    .into_rgba8();
                            let tx = ui.ctx().load_texture(
                                "Church Presenter",
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [im.width() as usize, im.height() as usize],
                                    &im,
                                ),
                                egui::TextureOptions::LINEAR,
                            );
                            ui.ctx().data_mut(|d| d.insert_temp(id, tx.clone()));
                            Some(tx)
                        });
                        if let Some(t) = texture {
                            ui.image((t.id(), egui::vec2(42., 42.)));
                        }
                        ui.vertical(|ui| {
                            section_header_label(ui, p, "关于");
                            ui.label("Church Presenter 1.0.6 (Rust Native Engine)");
                        });
                    });
                });
            });
    }
    pub fn directory_change(&mut self, ctx: &Context, section: Dir, path: String, primary: bool) {
        let path = path.trim();
        if !Path::new(path).is_dir() {
            self.message = format!("目录不存在 · {path}");
            return;
        }
        if primary || self.settings.primary_directory(section).trim().is_empty() {
            self.settings
                .set_primary_resource_directory(section, path.into());
        } else {
            self.settings
                .add_additional_resource_directory(section, path.into());
        }
        self.directory_text.clear();
        self.after_directory(ctx, section);
    }
    fn after_directory(&mut self, ctx: &Context, section: Dir) {
        if section == Dir::Presentations {
            self.settings.preferred_hymn_book.clear();
            self.sync_filters();
        } else {
            self.save();
        }
        self.refresh(ctx, section);
    }
}
