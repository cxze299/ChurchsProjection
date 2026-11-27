use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, TextStyle, Visuals};
use std::{collections::BTreeMap, fs};

#[derive(Default)]
pub struct Fonts {
    families: BTreeMap<String, FontFamily>,
}

impl Fonts {
    pub fn install(ctx: &Context) -> Self {
        let mut defs = FontDefinitions::default();
        let mut choices = Self::default();
        let faces: &[(&str, &[&str])] = &[
            (
                "Microsoft YaHei",
                &[r"C:\Windows\Fonts\msyh.ttc", r"C:\Windows\Fonts\msyh.ttf"],
            ),
            (
                "PingFang",
                &[
                    "/System/Library/Fonts/PingFang.ttc",
                    "/System/Library/Fonts/Supplemental/PingFang.ttc",
                ],
            ),
            ("SimHei", &[r"C:\Windows\Fonts\simhei.ttf"]),
            ("SimSun", &[r"C:\Windows\Fonts\simsun.ttc"]),
            (
                "Noto Sans CJK",
                &["/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"],
            ),
            (
                "Arial",
                &[
                    r"C:\Windows\Fonts\arial.ttf",
                    "/System/Library/Fonts/Supplemental/Arial.ttf",
                ],
            ),
        ];
        let mut first = None;
        for &(name, paths) in faces {
            if let Some(bytes) = paths.iter().find_map(|p| fs::read(p).ok()) {
                defs.font_data
                    .insert(name.into(), FontData::from_owned(bytes).into());
                if first.is_none() {
                    first = Some(name.to_owned());
                }
                choices
                    .families
                    .insert(name.into(), FontFamily::Name(name.into()));
            }
        }
        if let Some(first) = first {
            for family in [FontFamily::Proportional, FontFamily::Monospace] {
                defs.families
                    .entry(family)
                    .or_default()
                    .insert(0, first.clone());
            }
        }
        let fallback = defs.families[&FontFamily::Proportional].clone();
        for (name, family) in &choices.families {
            let mut fonts = vec![name.clone()];
            fonts.extend(fallback.iter().filter(|s| *s != name).cloned());
            defs.families.insert(family.clone(), fonts);
        }
        choices
            .families
            .insert("系统默认".into(), FontFamily::Proportional);
        choices
            .families
            .insert("Monospace".into(), FontFamily::Monospace);
        ctx.set_fonts(defs);
        choices
    }
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.families.keys()
    }
    pub fn family(&self, name: &str) -> FontFamily {
        self.families.get(name).cloned().unwrap_or_default()
    }
}

pub const GOLD: Color32 = Color32::from_rgb(229, 190, 118);
pub fn theme(ctx: &Context, dark: bool) {
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    let (base, panel, accent, selected) = if dark {
        (
            Color32::from_rgb(21, 29, 33),
            Color32::from_rgb(29, 39, 43),
            GOLD,
            Color32::from_rgb(65, 79, 80),
        )
    } else {
        (
            Color32::from_rgb(250, 249, 245),
            Color32::from_rgb(240, 240, 234),
            Color32::from_rgb(112, 78, 30),
            Color32::from_rgb(217, 230, 222),
        )
    };
    v.panel_fill = panel;
    v.window_fill = base;
    v.faint_bg_color = base;
    v.selection.bg_fill = selected;
    v.selection.stroke.color = accent;
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8., 7.);
        s.spacing.button_padding = egui::vec2(9., 5.);
        s.spacing.interact_size.y = 28.;
        for (style, size) in [
            (TextStyle::Body, 15.),
            (TextStyle::Button, 15.),
            (TextStyle::Heading, 23.),
            (TextStyle::Small, 12.),
        ] {
            if let Some(font) = s.text_styles.get_mut(&style) {
                font.size = size;
            }
        }
    });
}
pub fn color(value: &str, fallback: Color32) -> Color32 {
    Color32::from_hex(value).unwrap_or(fallback)
}
