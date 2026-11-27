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
        let ui_family=defs.families.entry(FontFamily::Proportional).or_default();
        if let Ok(bytes)=fs::read(r"C:\Windows\Fonts\seguisym.ttf") {
            defs.font_data.insert("UI symbols".into(),FontData::from_owned(bytes).into());
            ui_family.push("UI symbols".into());
        }
        let fallback=ui_family.clone();
        let mut bold=fallback.clone();
        for path in [r"C:\Windows\Fonts\msyhbd.ttc","/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc","/System/Library/Fonts/Supplemental/Arial Bold.ttf"] {
            if let Ok(bytes)=fs::read(path) {
                defs.font_data.insert("UI bold".into(),FontData::from_owned(bytes).into());
                bold.insert(0,"UI bold".into());break;
            }
        }
        defs.families.insert(FontFamily::Name("UI bold".into()),bold);
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
const fn rgb(value: u32) -> Color32 {
    Color32::from_rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub name: &'static str,
    pub dark: bool,
    pub background: Color32,
    pub surface: Color32,
    pub surface_card: Color32,
    pub primary: Color32,
    pub on_primary: Color32,
    pub secondary: Color32,
    pub on_secondary: Color32,
    pub secondary_container: Color32,
    pub on_secondary_container: Color32,
    pub tertiary: Color32,
    pub on_tertiary: Color32,
    pub error: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub well_bg: Color32,
    pub well_border: Color32,
}

pub const PALETTES: &[Palette] = &[
    Palette {
        name: "Light",
        dark: false,
        background: rgb(0xF4F4F5),
        surface: rgb(0xEFEFF1),
        surface_card: rgb(0xF9F9FA),
        primary: rgb(0x4A5568),
        on_primary: rgb(0xFFFFFF),
        secondary: rgb(0x607D8B),
        on_secondary: rgb(0xFFFFFF),
        secondary_container: rgb(0xCFD8DC),
        on_secondary_container: rgb(0x263238),
        tertiary: rgb(0x78909C),
        on_tertiary: rgb(0xFFFFFF),
        error: rgb(0xB00020),
        ink: rgb(0x1A1A1A),
        muted: rgb(0x44444A),
        well_bg: rgb(0xE5E5E8),
        well_border: rgb(0xCCCCD0),
    },
    Palette {
        name: "Warm",
        dark: false,
        background: rgb(0xFAF3E8),
        surface: rgb(0xF5ECD8),
        surface_card: rgb(0xFBF8F0),
        primary: rgb(0x7C5C3A),
        on_primary: rgb(0xFFFFFF),
        secondary: rgb(0x8D6E4A),
        on_secondary: rgb(0xFFFFFF),
        secondary_container: rgb(0xE8D0B5),
        on_secondary_container: rgb(0x2E1A05),
        tertiary: rgb(0xA07850),
        on_tertiary: rgb(0xFFFFFF),
        error: rgb(0xB00020),
        ink: rgb(0x1E150A),
        muted: rgb(0x4A3820),
        well_bg: rgb(0xEEECE4),
        well_border: rgb(0xD9C8A8),
    },
    Palette {
        name: "Ocean",
        dark: false,
        background: rgb(0xEDF5FA),
        surface: rgb(0xE0EEF6),
        surface_card: rgb(0xF3F9FC),
        primary: rgb(0x2A6B8A),
        on_primary: rgb(0xFFFFFF),
        secondary: rgb(0x4A7E96),
        on_secondary: rgb(0xFFFFFF),
        secondary_container: rgb(0xCAE3EF),
        on_secondary_container: rgb(0x001F2E),
        tertiary: rgb(0x3A8FA8),
        on_tertiary: rgb(0xFFFFFF),
        error: rgb(0xB00020),
        ink: rgb(0x0A1A22),
        muted: rgb(0x1E3D50),
        well_bg: rgb(0xE7EDEF),
        well_border: rgb(0xAAD0E0),
    },
    Palette {
        name: "Dark",
        dark: true,
        background: rgb(0x10131A),
        surface: rgb(0x1E1E1E),
        surface_card: rgb(0x2E2E2E),
        primary: rgb(0x90CAF9),
        on_primary: rgb(0x003258),
        secondary: rgb(0x4DB6AC),
        on_secondary: rgb(0x003D36),
        secondary_container: rgb(0x005B4F),
        on_secondary_container: rgb(0x70F0DD),
        tertiary: rgb(0xE1BEE7),
        on_tertiary: rgb(0x4A148C),
        error: rgb(0xF44336),
        ink: rgb(0xE6E1E5),
        muted: rgb(0xCAC4D0),
        well_bg: rgb(0x14171D),
        well_border: rgb(0x0C0E12),
    },
    Palette {
        name: "Studio",
        dark: true,
        background: rgb(0x111520),
        surface: rgb(0x141820),
        surface_card: rgb(0x272A32),
        primary: rgb(0xC4972A),
        on_primary: rgb(0x141820),
        secondary: rgb(0x5ABCA8),
        on_secondary: rgb(0x003D36),
        secondary_container: rgb(0x004D40),
        on_secondary_container: rgb(0x9ADFD4),
        tertiary: rgb(0x4A6FCC),
        on_tertiary: rgb(0x001A45),
        error: rgb(0xF44336),
        ink: rgb(0xE8E4D8),
        muted: rgb(0xBFBCB0),
        well_bg: rgb(0x30333A),
        well_border: rgb(0x252830),
    },
    Palette {
        name: "Midnight",
        dark: true,
        background: rgb(0x080E1A),
        surface: rgb(0x0E1520),
        surface_card: rgb(0x2D333D),
        primary: rgb(0x82AAFF),
        on_primary: rgb(0x001A45),
        secondary: rgb(0x7EB8D4),
        on_secondary: rgb(0x003547),
        secondary_container: rgb(0x004D66),
        on_secondary_container: rgb(0xBDE9FF),
        tertiary: rgb(0xA8C8E8),
        on_tertiary: rgb(0x0A2540),
        error: rgb(0xF44336),
        ink: rgb(0xDDE4F0),
        muted: rgb(0xB0BDD0),
        well_bg: rgb(0x353B45),
        well_border: rgb(0x1E2A3A),
    },
    Palette {
        name: "Forest",
        dark: true,
        background: rgb(0x080E0A),
        surface: rgb(0x0E1810),
        surface_card: rgb(0x29312A),
        primary: rgb(0x80C8A0),
        on_primary: rgb(0x003820),
        secondary: rgb(0x60A880),
        on_secondary: rgb(0x003018),
        secondary_container: rgb(0x004020),
        on_secondary_container: rgb(0x90D8A8),
        tertiary: rgb(0x90C8A0),
        on_tertiary: rgb(0x003818),
        error: rgb(0xF44336),
        ink: rgb(0xD0E8D8),
        muted: rgb(0xA8C8B0),
        well_bg: rgb(0x323933),
        well_border: rgb(0x1A2A1E),
    },
    Palette {
        name: "Mocha",
        dark: true,
        background: rgb(0x1E1928),
        surface: rgb(0x181622),
        surface_card: rgb(0x36343F),
        primary: rgb(0xCBA6F7),
        on_primary: rgb(0x1E0040),
        secondary: rgb(0xF5C2E7),
        on_secondary: rgb(0x3A0030),
        secondary_container: rgb(0x4A1040),
        on_secondary_container: rgb(0xFFD6F0),
        tertiary: rgb(0x89DCEB),
        on_tertiary: rgb(0x003040),
        error: rgb(0xF38BA8),
        ink: rgb(0xCDD6F4),
        muted: rgb(0xBAC2E8),
        well_bg: rgb(0x3E3C47),
        well_border: rgb(0x302840),
    },
    Palette {
        name: "Slate",
        dark: false,
        background: rgb(0xEAEEF3),
        surface: rgb(0xDFE5EC),
        surface_card: rgb(0xF4F7FA),
        primary: rgb(0x44607A),
        on_primary: rgb(0xFFFFFF),
        secondary: rgb(0x5C7789),
        on_secondary: rgb(0xFFFFFF),
        secondary_container: rgb(0xCCDAE3),
        on_secondary_container: rgb(0x13242E),
        tertiary: rgb(0x7A91A4),
        on_tertiary: rgb(0xFFFFFF),
        error: rgb(0xB00020),
        ink: rgb(0x141A20),
        muted: rgb(0x3D4A56),
        well_bg: rgb(0xE2E8EF),
        well_border: rgb(0xB6C2CF),
    },
    Palette {
        name: "Sand",
        dark: false,
        background: rgb(0xF7F4EF),
        surface: rgb(0xF1ECE4),
        surface_card: rgb(0xFAF8F5),
        primary: rgb(0x6B5D4B),
        on_primary: rgb(0xFFFFFF),
        secondary: rgb(0x7D7160),
        on_secondary: rgb(0xFFFFFF),
        secondary_container: rgb(0xE2D8C9),
        on_secondary_container: rgb(0x241C11),
        tertiary: rgb(0x978975),
        on_tertiary: rgb(0xFFFFFF),
        error: rgb(0xB00020),
        ink: rgb(0x1F1A14),
        muted: rgb(0x4C443A),
        well_bg: rgb(0xEEE9E1),
        well_border: rgb(0xCFC5B7),
    },
    Palette {
        name: "Plum",
        dark: true,
        background: rgb(0x1F1520),
        surface: rgb(0x1A1019),
        surface_card: rgb(0x3E2C3A),
        primary: rgb(0xE3A6CE),
        on_primary: rgb(0x2C0A22),
        secondary: rgb(0xD9A8BE),
        on_secondary: rgb(0x2E0E1E),
        secondary_container: rgb(0x482032),
        on_secondary_container: rgb(0xF8D9E5),
        tertiary: rgb(0xE8B98C),
        on_tertiary: rgb(0x33190A),
        error: rgb(0xF38BA8),
        ink: rgb(0xEEDCE8),
        muted: rgb(0xD3BCCA),
        well_bg: rgb(0x4A3646),
        well_border: rgb(0x5E4759),
    },
];

pub fn palette(name: &str) -> Palette {
    PALETTES
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(name))
        .copied()
        .unwrap_or_else(|| PALETTES.iter().find(|p| p.name == "Dark").copied().unwrap())
}

impl Palette {
    pub fn card_edge(self) -> Color32 {
        if self.dark {
            Color32::from_white_alpha(15)
        } else {
            Color32::from_black_alpha(18)
        }
    }
    pub fn selection(self) -> Color32 {
        self.primary
            .gamma_multiply(if self.dark { 0.18 } else { 0.10 })
    }
    pub fn selected_ink(self) -> Color32 {
        match self.name {
            "Dark" => rgb(0xB8DEFA),
            "Light" => rgb(0x2E3746),
            _ => self.primary,
        }
    }
}
pub fn theme(ctx: &Context, p: Palette) {
    let mut v = if p.dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.panel_fill = p.background;
    v.window_fill = p.surface_card;
    v.faint_bg_color = p.surface;
    v.extreme_bg_color = p.well_bg;
    v.override_text_color = Some(p.ink);
    v.hyperlink_color = p.primary;
    v.selection.bg_fill = p.selection();
    v.selection.stroke.color = p.selected_ink();
    for state in [
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        state.corner_radius = 8.into();
        state.bg_fill = p.well_bg;
        state.weak_bg_fill = p.well_bg;
        state.bg_stroke = egui::Stroke::new(1.0_f32, p.well_border);
        state.fg_stroke.color = p.ink;
        state.expansion = 0.;
    }
    v.widgets.noninteractive.fg_stroke.color = p.ink;
    v.widgets.noninteractive.bg_stroke.color = p.card_edge();
    v.widgets.hovered.bg_stroke.color = p.primary;
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6., 5.);
        s.spacing.button_padding = egui::vec2(8., 4.);
        s.spacing.interact_size.y = 26.;
        s.spacing.combo_width = 64.;
        for (style, size) in [
            (TextStyle::Body, 14.),
            (TextStyle::Button, 13.),
            (TextStyle::Heading, 20.),
            (TextStyle::Small, 11.5),
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
