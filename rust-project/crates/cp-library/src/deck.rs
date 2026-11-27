//! In-memory slide content and on-demand document rendering.
mod office;
mod pdf;
mod pptx;

pub use office::{convert_word_to_pdf, document_pdf_cache_file};
pub use pdf::{read_pdf_metadata, render_pdf_page, PdfMetadata};

use crate::discovery::extension;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum DeckError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Xml(#[from] roxmltree::Error),
    #[error(transparent)]
    Pdf(#[from] lopdf::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[cfg(windows)]
    #[error("Windows PDF renderer: {0}")]
    Windows(#[from] windows::core::Error),
    #[error("Unsupported presentation format: {0}")]
    Unsupported(PathBuf),
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Renderer(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextShape {
    pub name: String,
    /// Paragraphs in display order, each containing its original text runs.
    pub paragraphs: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SlideImage {
    /// Archive part name or filesystem path.
    pub source: String,
    pub bytes: Arc<[u8]>,
    pub is_background: bool,
}

impl SlideImage {
    pub fn decode(&self) -> Result<image::DynamicImage, image::ImageError> {
        image::load_from_memory(&self.bytes)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Slide {
    pub shapes: Vec<TextShape>,
    pub images: Vec<SlideImage>,
}

#[derive(Debug, Clone)]
pub struct Deck {
    pub source: PathBuf,
    pub title: Option<String>,
    pub slides: Vec<Slide>,
    pub pdf_metadata: Option<PdfMetadata>,
    /// Original PDF, or the cached conversion of a Word document.
    pub pdf_path: Option<PathBuf>,
}

impl Deck {
    /// Renders a zero-based page index, scaling the longest edge to the requested pixel count.
    pub fn render_page(&self, index: usize, max_edge: u32) -> Result<Vec<u8>, DeckError> {
        let pdf = self
            .pdf_path
            .as_ref()
            .ok_or_else(|| DeckError::Invalid("This deck has no PDF pages".into()))?;
        if index >= self.slides.len() {
            return Err(DeckError::Invalid(format!(
                "Page index {index} is out of range"
            )));
        }
        render_pdf_page(pdf, index, max_edge)
    }
}

pub(crate) fn is_image(path: &Path) -> bool {
    matches!(extension(path).as_str(), "png" | "jpg" | "jpeg")
}

fn image_slide(path: &Path) -> Result<Slide, DeckError> {
    Ok(Slide {
        images: vec![SlideImage {
            source: path.to_string_lossy().into_owned(),
            bytes: fs::read(path)?.into(),
            is_background: false,
        }],
        ..Default::default()
    })
}

pub fn load_deck(path: impl AsRef<Path>) -> Result<Deck, DeckError> {
    let source = path.as_ref();
    let mut deck = Deck {
        source: source.to_owned(),
        title: source.file_stem().map(|s| s.to_string_lossy().into_owned()),
        slides: Vec::new(),
        pdf_path: None,
        pdf_metadata: None,
    };
    if fs::metadata(source)?.is_dir() {
        let mut images = Vec::new();
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            if entry.file_type()?.is_file() && is_image(&entry.path()) {
                images.push(entry.path());
            }
        }
        images.sort();
        deck.slides = images
            .iter()
            .map(|img| image_slide(img))
            .collect::<Result<_, _>>()?;
    } else if is_image(source) {
        deck.slides.push(image_slide(source)?);
    } else {
        match extension(source).as_str() {
            "pptx" | "ppsx" => deck.slides = pptx::load(source)?,
            ext @ ("pdf" | "doc" | "docx") => {
                let path = if ext == "pdf" {
                    dunce::canonicalize(source)?
                } else {
                    convert_word_to_pdf(source)?
                };
                let metadata = read_pdf_metadata(&path)?;
                if let Some(title) = &metadata.title {
                    deck.title = Some(title.clone());
                }
                deck.slides.resize_with(metadata.page_count, Slide::default);
                deck.pdf_path = Some(path);
                deck.pdf_metadata = Some(metadata);
            }
            _ => return Err(DeckError::Unsupported(source.to_owned())),
        }
    }
    if deck.slides.is_empty() {
        return Err(DeckError::Invalid(format!(
            "No slides in {}",
            source.display()
        )));
    }
    Ok(deck)
}
