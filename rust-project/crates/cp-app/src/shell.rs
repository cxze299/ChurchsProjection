use crate::{
    app::{App, Tab},
    appearance,
    output::Content,
};
use egui::{
    vec2, Align2, Button, CentralPanel, Color32, Context, FontId, Frame, Rect, RichText,
    ScrollArea, Sense, SidePanel, Stroke, TopBottomPanel, Ui, ViewportBuilder, ViewportCommand,
    WindowLevel,
};

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.poll();
        self.keyboard(ctx, false);
        TopBottomPanel::top("title")
            .frame(
                Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(10),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("见证之窗").size(23.).strong());
                    ui.separator();
                    for (tab, label) in [
                        (Tab::Bible, "圣经速投"),
                        (Tab::Library, "诗歌与文稿"),
                        (Tab::Settings, "设置与多目录"),
                    ] {
                        if ui.selectable_label(self.tab == tab, label).clicked() {
                            self.change_tab(tab);
                        }
                    }
                });
            });
        TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let (color, label) = if self.output.visible {
                    (Color32::from_rgb(56, 177, 131), "正在投影")
                } else {
                    (ui.visuals().weak_text_color(), "投影已隐藏")
                };
                ui.colored_label(color, format!("●  {label}"));
                ui.separator();
                if self.bible_load.is_some() || self.library_load.is_some() {
                    ui.spinner();
                    ui.small("正在读取资源");
                } else if !self.message.is_empty() {
                    ui.small(&self.message);
                } else {
                    ui.small(format!(
                        "{} 个圣经译本    {} 个诗歌与文稿资源",
                        self.bible.modules.len(),
                        self.library.files.len()
                    ));
                }
            });
        });
        self.side(ctx, true);
        self.side(ctx, false);
        CentralPanel::default()
            .frame(
                Frame::new()
                    .inner_margin(14)
                    .fill(ctx.style().visuals.window_fill),
            )
            .show(ctx, |ui| match self.tab {
                Tab::Bible => self.bible_tab(ui),
                Tab::Library => self.library_tab(ui),
                Tab::Settings => self.settings_tab(ui),
            });
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
    fn side(&mut self, ctx: &Context, left: bool) {
        let expanded = if left { self.left } else { self.right };
        let width = if left {
            self.settings.schedule_panel_width_dp
        } else {
            self.settings.preview_panel_width_dp
        };
        let max = (ctx.screen_rect().width() * 0.34 - 36.).max(100.);
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
                let fill = match (ui.visuals().dark_mode, response.hovered()) {
                    (true, false) => Color32::from_rgb(63, 83, 88),
                    (true, true) => Color32::from_rgb(80, 105, 108),
                    (false, false) => Color32::from_rgb(203, 220, 212),
                    (false, true) => Color32::from_rgb(178, 203, 191),
                };
                ui.painter().rect_filled(button, 8, fill);
                let arrow = if left != expanded { "▶" } else { "◀" };
                ui.painter().text(
                    button.center(),
                    Align2::CENTER_CENTER,
                    arrow,
                    FontId::proportional(24.),
                    ui.visuals().strong_text_color(),
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
    fn schedule_panel(&mut self, ui: &mut Ui) {
        ui.heading("程序单");
        ui.horizontal(|ui| {
            ui.small(format!("{} 项", self.schedule.len()));
            if ui
                .add_enabled(!self.schedule.is_empty(), Button::new("清空"))
                .clicked()
            {
                self.schedule.clear();
            }
        });
        ui.separator();
        let mut play = None;
        let mut remove = None;
        let mut swap = None;
        ScrollArea::vertical().id_salt("queue").show(ui, |ui| {
            for (i, item) in self.schedule.iter().enumerate() {
                ui.push_id(i, |ui| {
                    if ui
                        .add(Button::new(item.entry.display_label()).wrap())
                        .clicked()
                    {
                        play = Some(i);
                    }
                    ui.horizontal(|ui| {
                        if ui.add_enabled(i > 0, Button::new("上移")).clicked() {
                            swap = Some((i, i - 1));
                        }
                        if ui
                            .add_enabled(i + 1 < self.schedule.len(), Button::new("下移"))
                            .clicked()
                        {
                            swap = Some((i, i + 1));
                        }
                        if ui.small_button("移除").clicked() {
                            remove = Some(i);
                        }
                    });
                    ui.separator();
                });
            }
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
            .selected_text(label)
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
        ScrollArea::vertical()
            .id_salt("live_controls")
            .show(ui, |ui| {
                ui.heading("投影预览与控制");
                let w = ui.available_width();
                self.renderer.canvas(
                    ui,
                    vec2(w, w * 9. / 16.),
                    &mut self.output,
                    &self.settings,
                    &self.fonts,
                );
                ui.small(match &self.output.content {
                    Content::Empty => "等待放映".into(),
                    Content::Scripture { verses, .. } => verses
                        .first()
                        .map(|v| v.display_reference())
                        .unwrap_or_default(),
                    Content::Media(p) => format!(
                        "{} · {}/{}",
                        p.document.title(),
                        p.index + 1,
                        p.document.len()
                    ),
                });
                ui.separator();
                ui.label("目标显示器");
                self.display_selector(ui, "target_preview");
                for (i, label) in [
                    if self.output.visible {
                        "隐藏投影 (Esc)"
                    } else {
                        "显示投影 (Esc)"
                    },
                    if self.output.blank {
                        "恢复文字 (B)"
                    } else {
                        "清除文字 (B)"
                    },
                    if self.fullscreen {
                        "窗口模式 (F11)"
                    } else {
                        "切换全屏 (F11)"
                    },
                    if self.output.top {
                        "取消窗口置顶 (T)"
                    } else {
                        "切换窗口置顶 (T)"
                    },
                    "清除图层 / 结束放映",
                ]
                .iter()
                .enumerate()
                {
                    let b = Button::new(*label)
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(
                            1.0_f32,
                            ui.visuals().widgets.inactive.bg_stroke.color,
                        ))
                        .min_size(vec2(w, 28.));
                    if ui.add(b).clicked() {
                        match i {
                            0 => {
                                self.cancel_pending_live();
                                self.output.visible = !self.output.visible;
                            }
                            1 => self.output.blank = !self.output.blank,
                            2 => {
                                self.fullscreen = !self.fullscreen;
                                self.target_changed();
                            }
                            3 => self.output.top = !self.output.top,
                            _ => {
                                self.cancel_pending_live();
                                self.output.end();
                            }
                        }
                    }
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
    pub fn set_theme(&mut self, ctx: &Context, dark: bool) {
        self.dark = dark;
        appearance::theme(ctx, dark);
        self.settings
            .extra
            .insert("theme".into(), if dark { "DARK" } else { "LIGHT" }.into());
        self.save();
    }
}
