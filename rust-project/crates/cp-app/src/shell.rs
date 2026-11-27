use crate::{
    app::{App, Tab},
    appearance,
    media::{Document, Page},
    output::worker,
    output::Content,
    widgets::*,
};
use egui::{
    vec2, Align2, Button, CentralPanel, Color32, Context, FontId, Frame, Rect, RichText,
    ScrollArea, Sense, SidePanel, Stroke, TopBottomPanel, Ui, ViewportBuilder, ViewportCommand,
    WindowLevel,
};
use std::time::{Duration, Instant};

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.poll();
        self.keyboard(ctx, false);
        self.workspace(ctx);
        if self.style_open {
            self.style_window(ctx);
        }
        if self.output.visible {
            self.projector(ctx);
        } else {
            self.placed = None;
        }
    }
}
impl App {
    pub fn workspace(&mut self, ctx: &Context) {
        self.side(ctx, true);
        self.side(ctx, false);
        if !self.message.is_empty() {
            self.toast = Some((
                std::mem::take(&mut self.message),
                Instant::now() + Duration::from_secs(4),
            ));
        }
        self.toast.take_if(|(_, until)| Instant::now() >= *until);
        let loading = self.bible_load.is_some()
            || self.library_load.is_some()
            || self.opening.is_some()
            || self.pictures.loading.is_some()
            || !self.schedule_loads.is_empty();
        if loading || self.toast.is_some() {
            TopBottomPanel::bottom("status")
                .frame(Frame::new().fill(self.palette.surface).inner_margin(4))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if loading {
                            ui.spinner();
                            ui.small("正在读取资源…");
                        }
                        if let Some((msg, until)) = &self.toast {
                            ui.small(msg);
                            ctx.request_repaint_after(
                                until.saturating_duration_since(Instant::now()),
                            );
                        }
                    });
                });
        }
        CentralPanel::default()
            .frame(Frame::NONE.fill(self.palette.background))
            .show(ctx, |ui| {
                self.tab_bar(ui);
                Frame::new()
                    .inner_margin(egui::Margin::symmetric(5, 4))
                    .show(ui, |ui| match self.tab {
                        Tab::Bible => self.bible_tab(ui),
                        Tab::Songs => self.songs_tab(ui),
                        Tab::Library => self.library_tab(ui),
                        Tab::Pictures => self.pictures_tab(ui),
                        Tab::Announcements => self.announcements_tab(ui),
                        Tab::Settings => self.settings_tab(ui),
                    });
            });
    }
    fn tab_bar(&mut self, ui: &mut Ui) {
        let p = self.palette;
        let bar = Frame::new()
            .fill(p.surface)
            .corner_radius(egui::CornerRadius {
                sw: 14,
                se: 14,
                ..Default::default()
            })
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(Button::new(RichText::new("⚙").size(19.)).frame(false))
                        .on_hover_text("设置")
                        .clicked()
                    {
                        self.change_tab(Tab::Settings);
                    }
                    if ui
                        .add(
                            Button::new(RichText::new(if p.dark { "☀" } else { "☾" }).size(19.))
                                .frame(false),
                        )
                        .on_hover_text("切换深色或浅色主题")
                        .clicked()
                    {
                        self.set_theme(ui.ctx(), if p.dark { "Light" } else { "Dark" });
                    }
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ScrollArea::horizontal()
                            .id_salt("main_tabs")
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                            )
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 2.;
                                    for (tab, name) in [
                                        (Tab::Bible, "圣经"),
                                        (Tab::Songs, "诗歌"),
                                        (Tab::Library, "文稿"),
                                        (Tab::Pictures, "图片"),
                                        (Tab::Announcements, "布告"),
                                        (Tab::Settings, "设置"),
                                    ] {
                                        let active = self.tab == tab;
                                        let text = if active {bold(name)} else {RichText::new(name)}.size(14.).color(if active {
                                            p.ink
                                        } else {
                                            p.ink.gamma_multiply(0.52)
                                        });
                                        let r = ui.add(
                                            Button::new(text)
                                                .frame(false)
                                                .min_size(vec2(44., 38.)),
                                        );
                                        if active {
                                            let rect = Rect::from_min_max(
                                                r.rect.left_bottom() + vec2(3., -3.),
                                                r.rect.right_bottom() - vec2(3., 0.),
                                            );
                                            ui.painter().rect_filled(rect, 2, p.primary);
                                        }
                                        if r.clicked() {
                                            self.change_tab(tab);
                                        }
                                    }
                                });
                            });
                    });
                });
            });
        ui.painter().line_segment(
            [
                bar.response.rect.left_bottom(),
                bar.response.rect.right_bottom(),
            ],
            Stroke::new(1.0_f32, p.card_edge()),
        );
    }
    fn side(&mut self, ctx: &Context, left: bool) {
        let expanded = if left { self.left } else { self.right };
        let width = if left {
            self.settings.schedule_panel_width_dp
        } else {
            self.settings.preview_panel_width_dp
        };
        let max = ((ctx.screen_rect().width() - 560.) * 0.5 - 36.).clamp(120., 480.);
        let width = (width as f32).clamp(100., max);
        if expanded {
            let panel = if left {
                SidePanel::left("schedule")
            } else {
                SidePanel::right("output")
            };
            panel
                .exact_width(width)
                .resizable(false)
                .frame(
                    Frame::new()
                        .fill(ctx.style().visuals.panel_fill)
                        .inner_margin(8),
                )
                .show(ctx, |ui| {
                    ui.set_width((width - 16.).max(1.));
                    if left {
                        self.schedule_panel(ui);
                    } else {
                        self.live_panel(ui);
                    }
                });
        }
        let strip = if left {
            SidePanel::left("schedule_strip")
        } else {
            SidePanel::right("output_strip")
        };
        strip
            .exact_width(36.)
            .resizable(false)
            .frame(Frame::NONE.fill(ctx.style().visuals.panel_fill))
            .show(ctx, |ui| {
                let strip = ui.max_rect();
                let button = Rect::from_center_size(strip.center(), vec2(32., 40.));
                let drag = ui.interact(strip, ui.id().with("resize"), Sense::drag());
                if expanded {
                    if drag.hovered() || drag.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    }
                    if drag.dragged() {
                        let sign = if left { 1. } else { -1. };
                        let new_width = (width + sign * drag.drag_delta().x)
                            .clamp(100., max)
                            .round() as i32;
                        if left {
                            self.settings.schedule_panel_width_dp = new_width;
                        } else {
                            self.settings.preview_panel_width_dp = new_width;
                        }
                    }
                    if drag.drag_stopped() {
                        self.save();
                    }
                }
                let label = if left {
                    "程序单"
                } else {
                    "投影预览与控制"
                };
                let response = ui
                    .interact(button, ui.id().with("toggle"), Sense::click())
                    .on_hover_text(label);
                let fill = self.palette.secondary_container;
                ui.painter().rect_filled(button, 8, fill);
                let arrow = if left != expanded { "▶" } else { "◀" };
                ui.painter().text(
                    button.center(),
                    Align2::CENTER_CENTER,
                    arrow,
                    FontId::proportional(24.),
                    self.palette.on_secondary_container,
                );
                if response.clicked() {
                    if left {
                        self.left = !self.left;
                    } else {
                        self.right = !self.right;
                    }
                }
            });
    }
    fn pick_schedule_files(&mut self, ctx: &Context) {
        let Some(files) = rfd::FileDialog::new()
            .add_filter("诗歌、文稿与图片", cp_library::discovery::EXTENSIONS)
            .pick_files()
        else {
            return;
        };
        let roots = self.library.roots.clone();
        self.schedule_loads.push_back(worker(ctx, move || {
            files
                .into_iter()
                .map(|path| {
                    if crate::pictures_ui::is_picture(&path) {
                        Ok(Content::Picture(path))
                    } else {
                        Document::open(&path, &cp_library::quick_play::book(&path, &roots))
                            .map(|document| Content::Media(Page { document, index: 0 }))
                            .map_err(|e| format!("{} · {e}", path.display()))
                    }
                })
                .collect()
        }));
    }
    fn schedule_panel(&mut self, ui: &mut Ui) {
        let p = self.palette;
        card_frame(p).inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("程序单").size(17.).strong());
                Frame::new()
                    .fill(p.well_bg)
                    .corner_radius(10)
                    .inner_margin(egui::Margin::symmetric(7, 2))
                    .show(ui, |ui| {
                        ui.small(format!("{} 项", self.schedule.len()));
                    });
                if ui.small_button("＋").on_hover_text("添加文件").clicked() {
                    self.pick_schedule_files(ui.ctx());
                }
                if ui
                    .add_enabled(!self.schedule.is_empty(), Button::new("清空").small())
                    .clicked()
                {
                    self.schedule.clear();
                }
            });
        });
        let mut play = None;
        let mut remove = None;
        let mut swap = None;
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let r = ui.add_sized(
                [ui.available_width(), 34.],
                Button::new("＋ 添加文件…")
                    .fill(p.surface)
                    .stroke(Stroke::NONE)
                    .corner_radius(8),
            );
            let rect = r.rect.shrink(1.);
            let points = [
                rect.left_top(),
                rect.right_top(),
                rect.right_bottom(),
                rect.left_bottom(),
                rect.left_top(),
            ];
            ui.painter().extend(egui::Shape::dashed_line(
                &points,
                Stroke::new(1.0_f32, p.muted.gamma_multiply(0.45)),
                4.,
                4.,
            ));
            if r.clicked() {
                self.pick_schedule_files(ui.ctx());
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ScrollArea::vertical()
                    .id_salt("queue")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (i, item) in self.schedule.iter().enumerate() {
                            ui.push_id(i, |ui| {
                                let (icon, color, subtitle) = match &item.content {
                                    Content::Scripture { verses, .. } => (
                                        "📖",
                                        Color32::from_rgb(57, 164, 125),
                                        verses
                                            .first()
                                            .map(|v| v.verse_text.clone())
                                            .unwrap_or_default(),
                                    ),
                                    Content::Media(page) => match &*page.document {
                                        Document::Song { song, .. } => {
                                            ("♫", p.primary, song.songbook.clone())
                                        }
                                        Document::Deck(_) => (
                                            "▤",
                                            p.tertiary,
                                            format!("文稿 · {} 页", page.document.len()),
                                        ),
                                    },
                                    Content::Picture(path) => {
                                        ("▧", p.error, path.display().to_string())
                                    }
                                    _ => ("", p.primary, String::new()),
                                };
                                let title = match &item.content {
                                    Content::Media(page) => page.document.title(),
                                    Content::Picture(path) => path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .into_owned(),
                                    _ => item
                                        .entry
                                        .as_ref()
                                        .map(|e| {
                                            e.display_label().trim_start_matches("📖 ").to_string()
                                        })
                                        .unwrap_or_default(),
                                };
                                card_frame(p).inner_margin(7).show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        let (tile, _) =
                                            ui.allocate_exact_size(vec2(28., 28.), Sense::hover());
                                        ui.painter().rect_filled(
                                            tile,
                                            7,
                                            color.gamma_multiply(0.17),
                                        );
                                        ui.painter().text(
                                            tile.center(),
                                            Align2::CENTER_CENTER,
                                            icon,
                                            FontId::proportional(17.),
                                            color,
                                        );
                                        ui.vertical(|ui| {
                                            if ui
                                                .add(
                                                    egui::Label::new(
                                                        RichText::new(&title).strong(),
                                                    )
                                                    .truncate()
                                                    .sense(Sense::click()),
                                                )
                                                .on_hover_text(&title)
                                                .clicked()
                                            {
                                                play = Some(i);
                                            }
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(&subtitle).small().color(p.muted),
                                                )
                                                .truncate(),
                                            );
                                        });
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.small_button("▶").on_hover_text("立即上屏").clicked()
                                        {
                                            play = Some(i);
                                        }
                                        if ui
                                            .add_enabled(i > 0, Button::new("↑").small())
                                            .on_hover_text("上移")
                                            .clicked()
                                        {
                                            swap = Some((i, i - 1));
                                        }
                                        if ui
                                            .add_enabled(
                                                i + 1 < self.schedule.len(),
                                                Button::new("↓").small(),
                                            )
                                            .on_hover_text("下移")
                                            .clicked()
                                        {
                                            swap = Some((i, i + 1));
                                        }
                                        if ui.small_button("×").on_hover_text("移除").clicked() {
                                            remove = Some(i);
                                        }
                                    });
                                });
                            });
                        }
                    });
            });
        });
        if let Some(i) = play {
            self.play_schedule(i);
        }
        if let Some(i) = remove {
            self.schedule.remove(i);
        }
        if let Some((a, b)) = swap {
            self.schedule.swap(a, b);
        }
    }
    pub fn display_selector(&mut self, ui: &mut Ui, id: &str) {
        let old = (self.fullscreen, self.target);
        let label = if self.fullscreen {
            format!("全屏显示器 {}", self.target + 1)
        } else {
            "窗口投影".into()
        };
        egui::ComboBox::from_id_salt(id)
            .selected_text(if id == "target_preview" {
                "▣ 屏幕".to_owned()
            } else {
                label
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.fullscreen, false, "窗口投影");
                for (i, screen) in self.screens.iter().enumerate() {
                    if ui
                        .selectable_label(
                            self.fullscreen && i == self.target,
                            format!("全屏显示器 {} · {}", i + 1, screen.name),
                        )
                        .clicked()
                    {
                        self.target = i;
                        self.fullscreen = true;
                    }
                }
            });
        if old != (self.fullscreen, self.target) {
            self.target_changed();
        }
    }
    fn live_panel(&mut self, ui: &mut Ui) {
        let p = self.palette;
        ScrollArea::vertical()
            .id_salt("live_controls")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let cols = if ui.available_width() >= 232. { 2 } else { 1 };
                let width = (ui.available_width()
                    - ui.spacing().item_spacing.x * (cols - 1) as f32)
                    / cols as f32;
                egui::Grid::new("output_actions")
                    .num_columns(cols)
                    .show(ui, |ui| {
                        for i in 0..4 {
                            if i == 0 {
                                ui.spacing_mut().combo_width = width;
                                self.display_selector(ui, "target_preview");
                            } else {
                                let (label, ink) = match i {
                                    1 => ("× 清除投影", p.error),
                                    2 => (
                                        if self.output.top {
                                            "◆ 已置顶"
                                        } else {
                                            "◇ 置顶"
                                        },
                                        p.ink,
                                    ),
                                    _ => (
                                        if self.output.blank {
                                            "◉ 恢复文字"
                                        } else {
                                            "◌ 清除文字"
                                        },
                                        p.ink,
                                    ),
                                };
                                let active =
                                    (i == 3 && self.output.blank) || (i == 2 && self.output.top);
                                if ui
                                    .add(
                                        Button::new(RichText::new(label).size(12.).color(ink))
                                            .fill(if active { p.selection() } else { p.surface })
                                            .stroke(Stroke::new(1.0_f32, p.card_edge()))
                                            .corner_radius(8)
                                            .min_size(vec2(width, 30.)),
                                    )
                                    .clicked()
                                {
                                    match i {
                                        1 => {
                                            self.cancel_pending_live();
                                            self.output.visible = false;
                                        }
                                        2 => self.output.top = !self.output.top,
                                        _ => self.output.blank = !self.output.blank,
                                    }
                                }
                            }
                            if (i + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
                if ui
                    .add_sized(
                        [ui.available_width(), 29.],
                        Button::new(RichText::new("■ 结束放映").color(p.error))
                            .fill(p.error.gamma_multiply(0.1))
                            .stroke(Stroke::new(1.0_f32, p.error.gamma_multiply(0.35)))
                            .corner_radius(8),
                    )
                    .clicked()
                {
                    self.cancel_pending_live();
                    self.output.end();
                }
                ui.add_space(6.);
                let live = self.output.visible;
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(
                        if live { p.error } else { p.muted },
                        if live { "● ON AIR" } else { "○ 待机" },
                    );
                    ui.small(if self.fullscreen {
                        format!("全屏 · 显示器 {}", self.target + 1)
                    } else {
                        "窗口投影".into()
                    });
                });
                let w = ui.available_width();
                let r = self.renderer.canvas(
                    ui,
                    vec2(w, w * 9. / 16.),
                    &mut self.output,
                    &self.settings,
                    &self.fonts,
                );
                ui.painter().rect_stroke(
                    r.rect,
                    5,
                    Stroke::new(
                        if live { 2.0_f32 } else { 1.0_f32 },
                        if live {
                            p.error.gamma_multiply(0.7)
                        } else {
                            p.card_edge()
                        },
                    ),
                    egui::StrokeKind::Inside,
                );
                ui.weak(match &self.output.content {
                    Content::Empty => "等待放映".into(),
                    Content::Scripture { verses, .. } => verses
                        .first()
                        .map(|v| v.display_reference())
                        .unwrap_or_default(),
                    Content::Media(page) => format!(
                        "{} · {}/{}",
                        page.document.title(),
                        page.index + 1,
                        page.document.len()
                    ),
                    Content::Picture(path) => path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    Content::Announcement(a) => a.caption().to_owned(),
                });
                if !live
                    && !matches!(self.output.content, Content::Empty)
                    && filter_chip(ui, p, false, "恢复投影").clicked()
                {
                    self.output.visible = true;
                }
            });
    }
    fn projector(&mut self, ctx: &Context) {
        let id = Self::projector_id();
        let builder = ViewportBuilder::default()
            .with_title("见证之窗 · 现场投影")
            .with_inner_size(vec2(800., 450.))
            .with_resizable(true)
            .with_active(false)
            .with_window_level(if self.output.top {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            });
        ctx.show_viewport_immediate(id, builder, |ctx, _| {
            if ctx.input(|i| i.viewport().close_requested()) {
                self.output.visible = false;
                self.cancel_pending_live();
                self.placed = None;
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                ctx.request_repaint_of(egui::ViewportId::ROOT);
                return;
            }
            self.keyboard(ctx, true);
            let desired = if self.fullscreen {
                self.target
            } else {
                usize::MAX
            };
            if self.placed != Some(desired) {
                if self.placed.is_some() {
                    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
                }
                if self.fullscreen {
                    let screen = &self.screens[self.target];
                    let dest = screen.bounds.center().to_vec2() / ctx.pixels_per_point();
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(320., 180.)));
                    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(
                        (dest - vec2(160., 90.)).to_pos2(),
                    ));
                    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
                } else {
                    let s = &self.screens[0];
                    let size = (s.work.size() / s.scale * 0.70).min(vec2(800., 450.));
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
                    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(
                        (s.work.center().to_vec2() / ctx.pixels_per_point() - size / 2.).to_pos2(),
                    ));
                }
                self.placed = Some(desired);
            }
            CentralPanel::default()
                .frame(Frame::NONE.fill(Color32::BLACK))
                .show(ctx, |ui| {
                    let r = self.renderer.canvas(
                        ui,
                        ui.available_size(),
                        &mut self.output,
                        &self.settings,
                        &self.fonts,
                    );
                    r.context_menu(|ui| {
                        for (i, label) in [
                            "全屏 / 窗口模式 (F11)",
                            "窗口置顶 (T)",
                            "临时黑屏 (B)",
                            "上一页 / 上一节",
                            "下一页 / 下一节",
                            "隐藏投影 (Esc)",
                        ]
                        .iter()
                        .enumerate()
                        {
                            if ui.button(*label).clicked() {
                                match i {
                                    0 => {
                                        self.fullscreen = !self.fullscreen;
                                        self.target_changed();
                                    }
                                    1 => self.output.top = !self.output.top,
                                    2 => self.output.blank = !self.output.blank,
                                    3 => self.navigate(-1, true),
                                    4 => self.navigate(1, true),
                                    _ => {
                                        self.output.visible = false;
                                        self.cancel_pending_live();
                                    }
                                }
                                ctx.request_repaint_of(egui::ViewportId::ROOT);
                                ui.close_menu();
                            }
                        }
                    });
                });
        });
    }
    pub fn set_theme(&mut self, ctx: &Context, name: &str) {
        self.palette = appearance::palette(name);
        appearance::theme(ctx, self.palette);
        self.settings
            .extra
            .insert("theme".into(), self.palette.name.to_uppercase().into());
        self.save();
    }
}
