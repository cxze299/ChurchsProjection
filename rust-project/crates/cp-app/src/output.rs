use crate::{
    appearance::{color, Fonts, GOLD},
    media::{Document, Page},
};
use cp_core::{AppSettings, BibleTranslationSettings, SelectedVerse};
use egui::{
    pos2, vec2, Color32, Context, FontId, Galley, Painter, Rect, Response, Sense, Stroke,
    TextureHandle, Ui,
};
use std::{
    collections::VecDeque,
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
};

pub fn worker<T: Send + 'static>(
    ctx: &Context,
    f: impl FnOnce() -> T + Send + 'static,
) -> Receiver<T> {
    let (tx, rx) = mpsc::channel();
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        if tx.send(f()).is_ok() {
            ctx.request_repaint_of(egui::ViewportId::ROOT);
        }
    });
    rx
}

#[derive(Clone, Default)]
pub enum Content {
    #[default]
    Empty,
    Scripture {
        verses: Arc<Vec<SelectedVerse>>,
        continuous: bool,
    },
    Media(Page),
    Picture(std::path::PathBuf),
    Announcement(crate::announcements_ui::Announcement),
}
pub struct Output {
    pub content: Content,
    pub visible: bool,
    pub blank: bool,
    pub top: bool,
    pub scroll: f32,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            content: Content::Empty,
            visible: false,
            blank: false,
            top: false,
            scroll: 1.,
        }
    }
}
impl Output {
    pub fn publish(&mut self, content: Content) {
        self.content = content;
        self.scroll = 1.;
        self.visible = true;
        self.blank = false;
    }
    pub fn end(&mut self) {
        *self = Self {
            top: self.top,
            ..Self::default()
        };
    }
}

enum Images {
    Loading(Receiver<Result<Vec<(egui::ColorImage, bool)>, String>>),
    Ready(Vec<(TextureHandle, bool)>),
    Failed(String),
}
#[derive(Clone)]
enum ImageSource {
    Picture(std::path::PathBuf),
    Slide(Page),
}
impl ImageSource {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Picture(a), Self::Picture(b)) => a == b,
            (Self::Slide(a), Self::Slide(b)) => {
                a.index == b.index && Arc::ptr_eq(&a.document, &b.document)
            }
            _ => false,
        }
    }
    fn decode(&self, edge: u32) -> Result<Vec<(egui::ColorImage, bool)>, String> {
        match self {
            Self::Picture(path) => Ok(vec![(
                pixels(image::open(path).map_err(|e| e.to_string())?, edge),
                false,
            )]),
            Self::Slide(page) => {
                let Document::Deck(d) = &*page.document else {
                    return Ok(vec![]);
                };
                if d.pdf_path.is_some() {
                    let bytes = d.render_page(page.index, edge).map_err(|e| e.to_string())?;
                    return Ok(vec![(
                        pixels(
                            image::load_from_memory(&bytes).map_err(|e| e.to_string())?,
                            edge,
                        ),
                        false,
                    )]);
                }
                d.slides[page.index]
                    .images
                    .iter()
                    .map(|im| {
                        let decoded =
                            image::load_from_memory(&im.bytes).map_err(|e| e.to_string())?;
                        Ok((pixels(decoded, edge), im.is_background))
                    })
                    .collect()
            }
        }
    }
}
struct CachedPage {
    source: ImageSource,
    small: bool,
    images: Images,
}
#[derive(Default)]
pub struct Renderer {
    pages: VecDeque<CachedPage>,
}
fn pixels(image: image::DynamicImage, edge: u32) -> egui::ColorImage {
    let rgba = if image.width().max(image.height()) > edge {
        image.thumbnail(edge, edge)
    } else {
        image
    }
    .into_rgba8();
    egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], &rgba)
}
fn fit(bounds: Rect, size: egui::Vec2) -> Rect {
    let ratio = (bounds.width() / size.x).min(bounds.height() / size.y);
    Rect::from_center_size(bounds.center(), size * ratio)
}
impl Renderer {
    fn images(&mut self, ctx: &Context, source: ImageSource, small: bool) -> Option<&Images> {
        // Complete off-screen requests too, so scrolling cannot strand the decode queue.
        let mut loading = 0;
        for entry in &mut self.pages {
            if let Images::Loading(rx) = &entry.images {
                entry.images = match rx.try_recv() {
                    Ok(Ok(images)) => Images::Ready(
                        images
                            .into_iter()
                            .map(|(im, bg)| {
                                (
                                    ctx.load_texture("投影页面", im, egui::TextureOptions::LINEAR),
                                    bg,
                                )
                            })
                            .collect(),
                    ),
                    Ok(Err(e)) => Images::Failed(e),
                    Err(mpsc::TryRecvError::Disconnected) => Images::Failed("页面读取中断".into()),
                    Err(mpsc::TryRecvError::Empty) => {
                        loading += 1;
                        continue;
                    }
                };
            }
        }
        let index = self
            .pages
            .iter()
            .position(|p| p.small == small && p.source.same(&source));
        let entry = if let Some(i) = index {
            self.pages.remove(i)?
        } else {
            if loading >= 4 {
                return None;
            }
            let job = source.clone();
            let edge = if small { 600 } else { 2560 };
            let images = Images::Loading(worker(ctx, move || job.decode(edge)));
            CachedPage {
                source,
                small,
                images,
            }
        };
        self.pages.push_front(entry);
        let (mut previews, mut full) = (0, 0);
        self.pages.retain(|p| {
            if p.small {
                previews += 1;
                previews <= 48
            } else {
                full += 1;
                full <= 4
            }
        });
        Some(&self.pages.front()?.images)
    }
    pub fn canvas(
        &mut self,
        ui: &mut Ui,
        size: egui::Vec2,
        output: &mut Output,
        settings: &AppSettings,
        fonts: &Fonts,
    ) -> Response {
        let content = &output.content;
        let blank = output.blank;
        let scroll = &mut output.scroll;
        let (outer, response) =
            ui.allocate_exact_size(size.max(vec2(1., 1.)), Sense::click_and_drag());
        let rect = fit(outer, vec2(16., 9.));
        let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
        ui.painter().rect_filled(outer, 0, Color32::BLACK);
        let small = rect.width() * ui.ctx().pixels_per_point() <= 600.;
        let mut blocks = Vec::new();
        let mut continuous = false;
        let mut center = false;
        let fallback = BibleTranslationSettings::default();
        match content {
            Content::Empty => {}
            Content::Picture(path) => {
                match self.images(ui.ctx(), ImageSource::Picture(path.clone()), small) {
                    Some(Images::Ready(textures)) => {
                        if let Some((texture, _)) = textures.first() {
                            painter.image(
                                texture.id(),
                                fit(rect, texture.size_vec2()),
                                Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)),
                                Color32::WHITE,
                            );
                        }
                    }
                    Some(Images::Failed(e)) => {
                        blocks.push(Block::plain(format!("无法读取图片 · {e}"), 28., GOLD))
                    }
                    _ => {}
                }
            }
            Content::Announcement(a) => a.paint(ui, rect, blank),
            Content::Scripture {
                verses,
                continuous: on,
            } if !blank => {
                continuous = *on;
                let mut add = |v: &SelectedVerse, last: bool| {
                    let style = settings
                        .projection_settings
                        .output_profiles
                        .first()
                        .and_then(|p| {
                            p.bible_settings
                                .translations
                                .iter()
                                .find(|s| s.file_name == v.translation_file_name)
                        })
                        .or_else(|| {
                            settings
                                .bible_settings
                                .translations
                                .iter()
                                .find(|s| s.file_name == v.translation_file_name)
                        })
                        .unwrap_or(&fallback);
                    let label = if verses.len() > 1 || style.show_abbreviation {
                        format!("{}   {}", v.display_reference(), v.bible_abbreviation)
                    } else {
                        v.display_reference()
                    };
                    blocks.push(Block {
                        text: label,
                        size: style.reference_font_size as f32,
                        family: fonts.family(&style.reference_font_type),
                        color: color(&style.reference_color, GOLD),
                        bold: style.reference_bold,
                        gap: 6.,
                    });
                    blocks.push(Block {
                        text: v.verse_text.clone(),
                        size: style.text_font_size as f32,
                        family: fonts.family(&style.text_font_type),
                        color: color(&style.text_color, Color32::WHITE),
                        bold: style.text_bold,
                        gap: if continuous && last {
                            settings
                                .bible_settings
                                .continuous_passage_spacing
                                .unwrap_or(24)
                                .max(0) as f32
                        } else {
                            12.
                        },
                    });
                };
                if continuous {
                    if let Some(v) = verses.first() {
                        for group in &v.continuous_passages {
                            for (i, line) in group.iter().enumerate() {
                                add(line, i + 1 == group.len());
                            }
                        }
                    }
                } else {
                    for v in verses.iter() {
                        add(v, true);
                    }
                }
            }
            Content::Media(page) => match &*page.document {
                Document::Song { slides, .. } if !blank => {
                    center = true;
                    let slide = &slides[page.index];
                    if !slide.section.is_empty() {
                        blocks.push(Block::plain(slide.section.clone(), 30., GOLD));
                    }
                    blocks.push(Block::plain(slide.lines.join("\n"), 56., Color32::WHITE));
                }
                Document::Deck(d) => {
                    let slide = &d.slides[page.index];
                    if !slide.images.is_empty() || d.pdf_path.is_some() {
                        match self.images(ui.ctx(), ImageSource::Slide(page.clone()), small) {
                            Some(Images::Ready(images)) => {
                                let count = images.iter().filter(|(_, bg)| !bg).count().max(1);
                                let mut index = 0;
                                for (texture, bg) in images
                                    .iter()
                                    .filter(|(_, b)| *b)
                                    .chain(images.iter().filter(|(_, b)| !*b))
                                {
                                    if blank && !bg {
                                        continue;
                                    }
                                    let bounds = if *bg {
                                        rect
                                    } else {
                                        let width = rect.width() / count as f32;
                                        let cell = Rect::from_min_size(
                                            rect.min + vec2(width * index as f32, 0.),
                                            vec2(width, rect.height()),
                                        );
                                        index += 1;
                                        cell
                                    };
                                    painter.image(
                                        texture.id(),
                                        fit(bounds, texture.size_vec2()),
                                        Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)),
                                        Color32::WHITE,
                                    );
                                }
                            }
                            Some(Images::Failed(e)) if !blank => {
                                blocks.push(Block::plain(format!("无法显示页面 · {e}"), 28., GOLD))
                            }
                            Some(Images::Loading(_)) | None if !blank => {
                                blocks.push(Block::plain("正在加载页面…".into(), 26., GOLD))
                            }
                            _ => {}
                        }
                    }
                    if !blank {
                        let text = slide
                            .shapes
                            .iter()
                            .flat_map(|s| s.paragraphs.iter().map(|p| p.concat()))
                            .collect::<Vec<_>>()
                            .join("\n");
                        if !text.trim().is_empty() {
                            if !slide.images.is_empty() {
                                painter.rect_filled(rect, 0, Color32::from_black_alpha(95));
                            }
                            blocks.push(Block::plain(text, 46., Color32::WHITE));
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
        if !blocks.is_empty() {
            let scale = rect.width() / 1280.;
            let inner = rect.shrink2(vec2(48., 36.) * scale);
            let lines = if continuous {
                settings
                    .bible_settings
                    .continuous_line_spacing_percent
                    .unwrap_or(135)
                    .clamp(60, 300) as f32
                    / 100.
            } else {
                1.20
            };
            let mut factor = scale;
            let (mut laid, mut height) = layout(&blocks, &painter, inner.width(), factor, lines);
            for _ in 0..5 {
                if continuous || height <= inner.height() {
                    break;
                }
                factor *= (inner.height() / height).clamp(0.25, 0.95);
                (laid, height) = layout(&blocks, &painter, inner.width(), factor, lines);
            }
            let overflow = (height - inner.height()).max(0.);
            if continuous && overflow > 0. && response.hovered() {
                let wheel = ui.input(|i| i.smooth_scroll_delta.y);
                *scroll = (*scroll - (wheel + response.drag_delta().y) / overflow).clamp(0., 1.);
                if wheel != 0. || response.dragged() {
                    ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                }
            }
            let mut y = if continuous {
                inner.top() - overflow * *scroll
            } else {
                inner.center().y - height / 2.
            };
            let clip = painter.with_clip_rect(inner.intersect(painter.clip_rect()));
            for (block, galley) in blocks.iter().zip(laid) {
                let x = if center {
                    inner.center().x - galley.size().x / 2.
                } else {
                    inner.left()
                };
                let p = pos2(x, y);
                y += galley.size().y + block.gap * factor;
                if block.bold {
                    clip.galley(
                        p + vec2((factor * 1.1).max(0.3), 0.),
                        galley.clone(),
                        block.color,
                    );
                }
                clip.galley(p, galley, block.color);
            }
            if continuous && overflow > 0. && response.hovered() {
                let h = inner.height() * inner.height() / height;
                let y = inner.top() + (inner.height() - h) * *scroll;
                painter.line_segment(
                    [pos2(rect.right() - 3., y), pos2(rect.right() - 3., y + h)],
                    Stroke::new(2.0_f32, Color32::GRAY),
                );
            }
        }
        response
    }
}
struct Block {
    text: String,
    size: f32,
    family: egui::FontFamily,
    color: Color32,
    bold: bool,
    gap: f32,
}
impl Block {
    fn plain(text: String, size: f32, color: Color32) -> Self {
        Self {
            text,
            size,
            color,
            family: Default::default(),
            bold: false,
            gap: 12.,
        }
    }
}
fn layout(
    blocks: &[Block],
    painter: &Painter,
    width: f32,
    factor: f32,
    lines: f32,
) -> (Vec<Arc<Galley>>, f32) {
    let mut height = 0.;
    let mut output = Vec::with_capacity(blocks.len());
    for b in blocks {
        let size = (b.size.clamp(8., 200.) * factor).max(1.);
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = width.max(1.);
        job.append(
            &b.text,
            0.,
            egui::TextFormat {
                font_id: FontId::new(size, b.family.clone()),
                color: b.color,
                line_height: Some(size * lines),
                ..Default::default()
            },
        );
        let galley = painter.layout_job(job);
        height += galley.size().y + b.gap * factor;
        output.push(galley);
    }
    if let Some(b) = blocks.last() {
        height -= b.gap * factor;
    }
    (output, height)
}
