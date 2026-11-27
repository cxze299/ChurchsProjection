use crate::appearance::Palette;
use egui::{
    vec2, Align, Button, Color32, Frame, Layout, Margin, Response, RichText, Sense, Stroke, Ui,
    Vec2,
};

pub fn bold(text:impl Into<String>)->RichText {
    RichText::new(text).family(egui::FontFamily::Name("UI bold".into())).strong()
}
pub fn card_frame(p: Palette) -> Frame {
    Frame::new()
        .fill(p.surface_card)
        .stroke(Stroke::new(1.0_f32, p.card_edge()))
        .corner_radius(14)
        .inner_margin(12)
}
pub fn sunken_frame(p: Palette, focused: bool) -> Frame {
    Frame::new()
        .fill(p.well_bg)
        .stroke(Stroke::new(
            1.0_f32,
            if focused { p.primary } else { p.well_border },
        ))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(10, 6))
}
pub fn raised_action_button(
    ui: &mut Ui,
    p: Palette,
    label: &str,
    bg: Color32,
    fg: Color32,
    size: Vec2,
    tooltip: &str,
) -> Response {
    let text=bold(label).color(fg);
    let galley=egui::WidgetText::from(text).into_galley(ui,Some(egui::TextWrapMode::Extend),f32::INFINITY,egui::TextStyle::Button);
    let size=size.max(galley.size()+vec2(16.,8.));
    let (rect,response)=ui.allocate_exact_size(size,Sense::click());
    let fill=if response.hovered() || response.has_focus(){bg.lerp_to_gamma(Color32::WHITE,if p.dark{0.13}else{0.08})}else{bg};
    ui.painter().rect_filled(rect,8,fill);
    ui.painter().rect_stroke(rect,8,Stroke::new(1.0_f32,p.card_edge()),egui::StrokeKind::Inside);
    ui.painter().galley(rect.center()-galley.size()/2.,galley,fg);
    response.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::Button,ui.is_enabled(),tooltip));
    response.on_hover_text(tooltip)
}
pub fn primary(ui: &mut Ui, p: Palette, label: &str) -> Response {
    raised_action_button(ui, p, label, p.primary, p.on_primary, vec2(34., 34.), label)
}
pub fn schedule_button(ui: &mut Ui, p: Palette) -> Response {
    raised_action_button(
        ui,
        p,
        "＋ 加入程序单",
        p.secondary,
        p.on_secondary,
        vec2(34., 34.),
        "加入程序单",
    )
}
pub fn filter_chip(ui: &mut Ui, p: Palette, selected: bool, text: &str) -> Response {
    let ink = if selected { p.selected_ink() } else { p.ink };
    ui.add(
        Button::new(RichText::new(text).size(12.).color(ink))
            .corner_radius(8)
            .fill(if selected { p.selection() } else { p.well_bg })
            .stroke(Stroke::new(
                1.0_f32,
                if selected {
                    p.primary.gamma_multiply(0.45)
                } else {
                    p.card_edge()
                },
            )),
    )
}
pub fn section_header_label(ui: &mut Ui, p: Palette, text: &str) {
    ui.label(
        bold(text.to_uppercase())
            .size(11.5)
            .color(p.muted),
    );
}
pub fn list_row(
    ui: &mut Ui,
    p: Palette,
    selected: bool,
    text: &str,
    align_center: bool,
) -> Response {
    row(ui, p, selected, align_center, |ui| {
        let text = if selected { bold(text).color(p.selected_ink()) } else { RichText::new(text).color(p.ink) };
        ui.add(
            egui::Label::new(text)
                .selectable(false)
                .wrap(),
        );
    })
}
pub fn row(
    ui: &mut Ui,
    p: Palette,
    selected: bool,
    center: bool,
    contents: impl FnOnce(&mut Ui),
) -> Response {
    let mut frame = Frame::new()
        .corner_radius(9)
        .inner_margin(Margin::symmetric(7, 4))
        .begin(ui);
    frame
        .content_ui
        .set_width(frame.content_ui.available_width().max(1.));
    frame.content_ui.with_layout(
        Layout::top_down(if center { Align::Center } else { Align::Min }),
        contents,
    );
    let response = frame.allocate_space(ui).interact(Sense::click());
    frame.frame.fill = if selected {
        p.selection()
    } else if response.hovered() {
        p.ink.gamma_multiply(0.05)
    } else {
        Color32::TRANSPARENT
    };
    frame.paint(ui);
    response
}
pub fn well_edit(
    ui: &mut Ui,
    p: Palette,
    id: &str,
    value: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    let id = ui.make_persistent_id(id);
    let mut frame = sunken_frame(p, ui.memory(|m| m.has_focus(id))).begin(ui);
    let response = frame.content_ui.add(
        egui::TextEdit::singleline(value)
            .id(id)
            .frame(false)
            .desired_width((width - 22.).max(12.))
            .hint_text(hint),
    );
    if response.hovered() || response.has_focus() {
        frame.frame.stroke.color = p.primary;
    }
    frame.end(ui);
    response
}

pub fn thumbnail_grid(
    ui: &mut Ui,
    id: egui::Id,
    count: usize,
    mut tile: impl FnMut(&mut Ui, usize, f32),
) {
    let gap = 8.;
    let cols = ((ui.available_width() + gap) / 180.).floor().clamp(2., 4.) as usize;
    let width = ((ui.available_width() - gap * (cols - 1) as f32 - 10.) / cols as f32).max(16.);
    let height = width * 9. / 16. + 27.;
    egui::ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink([false, false])
        .show_rows(ui, height, count.div_ceil(cols), |ui, rows| {
            for row in rows {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for i in row * cols..((row + 1) * cols).min(count) {
                        ui.push_id(i, |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                ui.spacing_mut().item_spacing.y = 0.;
                                tile(ui, i, width);
                            });
                        });
                    }
                });
            }
        });
}

pub fn thumbnail_caption(
    ui: &mut Ui,
    p: Palette,
    canvas: Response,
    title: &str,
    active: bool,
) -> Response {
    let width = canvas.rect.width();
    let (label, response) = ui.allocate_exact_size(vec2(width, 27.), Sense::click());
    let area = canvas.rect.union(label);
    ui.painter().rect_filled(
        label,
        egui::CornerRadius {
            sw: 6,
            se: 6,
            ..Default::default()
        },
        if active {
            p.secondary_container
        } else {
            p.surface
        },
    );
    ui.painter_at(label.shrink(2.)).text(
        label.left_center() + vec2(7., 0.),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(12.),
        if active {
            p.on_secondary_container
        } else {
            p.ink
        },
    );
    ui.painter().rect_stroke(
        area,
        6,
        Stroke::new(
            if active { 2.0_f32 } else { 1.0_f32 },
            if active { p.secondary } else { p.card_edge() },
        ),
        egui::StrokeKind::Inside,
    );
    canvas.union(response)
}

pub fn switch(ui: &mut Ui, p: Palette, on: &mut bool, label: &str) -> bool {
    let result = ui
        .horizontal(|ui| {
            let (rect, r) = ui.allocate_exact_size(vec2(36., 21.), Sense::click());
            ui.painter()
                .rect_filled(rect, 12, if *on { p.primary } else { p.well_bg });
            ui.painter().rect_stroke(
                rect,
                12,
                Stroke::new(1.0_f32, if *on { p.primary } else { p.muted }),
                egui::StrokeKind::Inside,
            );
            ui.painter().circle_filled(
                egui::pos2(
                    if *on {
                        rect.right() - 10.5
                    } else {
                        rect.left() + 10.5
                    },
                    rect.center().y,
                ),
                7.,
                if *on { p.on_primary } else { p.muted },
            );
            r.clicked()
                | ui.add(
                    egui::Label::new(label)
                        .sense(Sense::click())
                        .selectable(false),
                )
                .clicked()
        })
        .inner;
    if result {
        *on = !*on;
    }
    result
}
