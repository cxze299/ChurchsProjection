use crate::{
    app::App,
    output::{Content, Output},
    widgets::*,
};
use egui::{vec2, Align2, Color32, FontId, Rect, RichText, Ui};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Countdown {
    duration: u32,
    left: u32,
    started: Option<Instant>,
}
impl Default for Countdown {
    fn default() -> Self {
        Self {
            duration: 300,
            left: 300,
            started: None,
        }
    }
}
impl Countdown {
    pub fn set_seconds(&mut self, seconds: u32) {
        self.duration = seconds.min(359999);
        self.reset();
    }
    pub fn remaining(&self, now: Instant) -> u32 {
        self.left.saturating_sub(self.started.map_or(0, |start| {
            now.saturating_duration_since(start)
                .as_secs()
                .min(u32::MAX as u64) as u32
        }))
    }
    pub fn toggle(&mut self, now: Instant) {
        let left = self.remaining(now);
        if self.started.is_some() && left > 0 {
            self.left = left;
            self.started = None;
        } else {
            self.left = if left == 0 { self.duration } else { left };
            self.started = Some(now);
        }
    }
    pub fn reset(&mut self) {
        self.started = None;
        self.left = self.duration;
    }
    fn running(&self, now: Instant) -> bool {
        self.started.is_some() && self.remaining(now) > 0
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Text,
    Countdown,
    Clock,
}
#[derive(Clone)]
pub struct Announcement {
    pub text: String,
    size: u32,
    bold: bool,
    ink: Color32,
    background: Color32,
    position: usize,
    mode: Mode,
    timer: Countdown,
}
impl Default for Announcement {
    fn default() -> Self {
        Self {
            text: String::new(),
            size: 64,
            bold: false,
            ink: Color32::WHITE,
            background: Color32::BLACK,
            position: 4,
            mode: Mode::Text,
            timer: Default::default(),
        }
    }
}
fn time_label(seconds: u32) -> String {
    if seconds >= 3600 {
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    } else {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }
}
#[cfg(windows)]
fn clock_text() -> String {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetLocalTime(time: *mut windows_sys::Win32::Foundation::SYSTEMTIME);
    }
    // GetLocalTime initializes the caller-owned SYSTEMTIME structure.
    let mut time = std::mem::MaybeUninit::uninit();
    let time = unsafe {
        GetLocalTime(time.as_mut_ptr());
        time.assume_init()
    };
    format!("{:02}:{:02}:{:02}", time.wHour, time.wMinute, time.wSecond)
}
#[cfg(not(windows))]
fn clock_text() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        % 86400;
    format!(
        "{:02}:{:02}:{:02} UTC",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}
impl Announcement {
    pub fn caption(&self) -> &str {
        match self.mode {
            Mode::Text => &self.text,
            Mode::Countdown => "倒计时",
            Mode::Clock => "时钟",
        }
    }
    pub fn paint(&self, ui: &Ui, rect: Rect, blank: bool) {
        let now = Instant::now();
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0, self.background);
        if blank {
            return;
        }
        let text = match self.mode {
            Mode::Text => self.text.clone(),
            Mode::Countdown => time_label(self.timer.remaining(now)),
            Mode::Clock => clock_text(),
        };
        if self.mode == Mode::Clock || (self.mode == Mode::Countdown && self.timer.running(now)) {
            ui.ctx().request_repaint_after(Duration::from_millis(200));
        }
        let bounds = rect.shrink(rect.width() * 0.04);
        let mut size = self.size as f32 * rect.width() / 1280.;
        let mut galley = painter.layout(
            text.clone(),
            FontId::proportional(size.max(1.)),
            self.ink,
            bounds.width(),
        );
        if galley.size().y > bounds.height() {
            size *= bounds.height() / galley.size().y;
            galley = painter.layout(
                text,
                FontId::proportional(size.max(1.)),
                self.ink,
                bounds.width(),
            );
        }
        let align = [
            Align2::LEFT_TOP,
            Align2::CENTER_TOP,
            Align2::RIGHT_TOP,
            Align2::LEFT_CENTER,
            Align2::CENTER_CENTER,
            Align2::RIGHT_CENTER,
            Align2::LEFT_BOTTOM,
            Align2::CENTER_BOTTOM,
            Align2::RIGHT_BOTTOM,
        ][self.position];
        let pos = align.align_size_within_rect(galley.size(), bounds).min;
        if self.bold {
            painter.galley(pos + vec2(size * 0.035, 0.), galley.clone(), self.ink);
        }
        painter.galley(pos, galley, self.ink);
    }
}
impl App {
    pub fn go_announcement(&mut self) {
        self.cancel_pending_live();
        self.output
            .publish(Content::Announcement(self.announcement.clone()));
    }
    fn update_live_announcement(&mut self) {
        if matches!(self.output.content, Content::Announcement(_)) {
            self.output.content = Content::Announcement(self.announcement.clone());
        }
    }
    pub fn announcements_tab(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .id_salt("announcement_workspace")
            .show(ui, |ui| {
                if ui.available_width() < 500. {
                    self.announcement_editor(ui);
                    self.announcement_preview(ui);
                } else {
                    ui.columns(2, |cols| {
                        self.announcement_editor(&mut cols[0]);
                        self.announcement_preview(&mut cols[1]);
                    });
                }
            });
    }
    fn announcement_editor(&mut self, ui: &mut Ui) {
        let p = self.palette;
        let a = &mut self.announcement;
        card_frame(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            section_header_label(ui, p, "布告文字");
            if ui
                .add(
                    egui::TextEdit::multiline(&mut a.text)
                        .hint_text("输入布告内容…")
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                )
                .changed()
            {
                a.mode = Mode::Text;
            }
            ui.horizontal_wrapped(|ui| {
                ui.label("字号");
                ui.add(egui::DragValue::new(&mut a.size).range(12..=240));
                ui.checkbox(&mut a.bold, "粗体");
                ui.label("文字");
                ui.color_edit_button_srgba(&mut a.ink);
                ui.label("背景");
                ui.color_edit_button_srgba(&mut a.background);
            });
        });
        card_frame(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                for (mode, label) in [
                    (Mode::Text, "布告"),
                    (Mode::Countdown, "倒计时"),
                    (Mode::Clock, "时钟"),
                ] {
                    if filter_chip(ui, p, a.mode == mode, label).clicked() {
                        a.mode = mode;
                    }
                }
            });
            let now = Instant::now();
            ui.vertical_centered(|ui| {
                ui.add_space(8.);
                ui.label(
                    RichText::new(if a.mode == Mode::Clock {
                        clock_text()
                    } else {
                        time_label(a.timer.remaining(now))
                    })
                    .size(38.)
                    .strong(),
                );
                ui.add_space(6.);
                ui.horizontal(|ui| {
                    for (step, label) in [(3600, "时"), (60, "分"), (1, "秒")] {
                        ui.vertical_centered(|ui| {
                            ui.set_width(38.);
                            if ui.small_button("＋").clicked() {
                                a.timer.set_seconds(a.timer.duration.saturating_add(step));
                                a.mode = Mode::Countdown;
                            }
                            ui.label(format!(
                                "{:02}",
                                if step == 3600 {
                                    a.timer.duration / step
                                } else {
                                    a.timer.duration / step % 60
                                }
                            ));
                            if ui.small_button("−").clicked() {
                                a.timer.set_seconds(a.timer.duration.saturating_sub(step));
                                a.mode = Mode::Countdown;
                            }
                            ui.small(label);
                        });
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    if primary(
                        ui,
                        p,
                        if a.timer.running(now) {
                            "Ⅱ 暂停"
                        } else {
                            "▶ 开始"
                        },
                    )
                    .clicked()
                    {
                        a.timer.toggle(now);
                        a.mode = Mode::Countdown;
                    }
                    if filter_chip(ui, p, false, "↺ 重置").clicked() {
                        a.timer.reset();
                        a.mode = Mode::Countdown;
                    }
                });
            });
            if a.timer.running(now) || a.mode == Mode::Clock {
                ui.ctx().request_repaint_after(Duration::from_millis(200));
            }
        });
        self.update_live_announcement();
    }
    fn announcement_preview(&mut self, ui: &mut Ui) {
        let p = self.palette;
        card_frame(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            section_header_label(ui, p, "布告预览");
            let w = ui.available_width();
            self.renderer.canvas(
                ui,
                vec2(w, w * 9. / 16.),
                &mut Output {
                    content: Content::Announcement(self.announcement.clone()),
                    ..Default::default()
                },
                &self.settings,
                &self.fonts,
            );
            ui.add_space(8.);
            section_header_label(ui, p, "屏幕位置");
            for r in 0..3 {
                ui.horizontal(|ui| {
                    for c in 0..3 {
                        let i = r * 3 + c;
                        if filter_chip(
                            ui,
                            p,
                            self.announcement.position == i,
                            if self.announcement.position == i {
                                " ● "
                            } else {
                                " · "
                            },
                        )
                        .clicked()
                        {
                            self.announcement.position = i;
                            self.update_live_announcement();
                        }
                    }
                });
            }
            ui.horizontal_wrapped(|ui| {
                if primary(ui, p, "▶ 立即上屏").clicked() {
                    self.go_announcement();
                }
                if raised_action_button(
                    ui,
                    p,
                    "清除",
                    p.error,
                    Color32::WHITE,
                    vec2(48., 34.),
                    "清除布告投影",
                )
                .clicked()
                    && matches!(self.output.content, Content::Announcement(_))
                {
                    self.output.end();
                }
            });
        });
    }
}
