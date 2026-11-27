use super::DeckError;
use lopdf::{decode_text_string, Document, Object, ObjectId};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct PdfMetadata {
    pub page_count: usize,
    /// First page media box dimensions in PDF points.
    pub page_width_pt: f64,
    pub page_height_pt: f64,
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
}

fn media_box(doc: &Document, mut id: ObjectId) -> Result<(f64, f64), DeckError> {
    let mut visited = BTreeSet::new();
    while visited.insert(id) {
        let dict = doc.get_dictionary(id)?;
        if let Ok(object) = dict.get_deref(b"MediaBox", doc) {
            let bounds = object.as_array()?;
            if bounds.len() != 4 {
                break;
            }
            let point = |i: usize| -> Result<f64, lopdf::Error> {
                Ok(doc.dereference(&bounds[i])?.1.as_float()? as f64)
            };
            return Ok(((point(2)? - point(0)?).abs(), (point(3)? - point(1)?).abs()));
        }
        match dict.get(b"Parent").and_then(Object::as_reference) {
            Ok(parent) => id = parent,
            Err(_) => break,
        }
    }
    Err(DeckError::Invalid("PDF page has no valid media box".into()))
}

pub fn read_pdf_metadata(path: impl AsRef<Path>) -> Result<PdfMetadata, DeckError> {
    let mut doc = Document::load(path)?;
    if doc.is_encrypted() {
        doc.decrypt("")?;
    }
    doc.catalog()?;
    let mut pages = doc.page_iter();
    let first = pages
        .next()
        .ok_or_else(|| DeckError::Invalid("PDF contains no pages".into()))?;
    let page_count = 1 + pages.count();
    let (page_width_pt, page_height_pt) = media_box(&doc, first)?;
    let info = doc
        .trailer
        .get_deref(b"Info", &doc)
        .and_then(Object::as_dict)
        .ok();
    let field = |name: &[u8]| decode_text_string(info?.get_deref(name, &doc).ok()?).ok();
    Ok(PdfMetadata {
        page_count,
        page_width_pt,
        page_height_pt,
        title: field(b"Title"),
        author: field(b"Author"),
        subject: field(b"Subject"),
        keywords: field(b"Keywords"),
        creator: field(b"Creator"),
        producer: field(b"Producer"),
    })
}

/// Renders a zero-based PDF page to PNG bytes. Uses Windows.Data.Pdf on Windows and
/// Poppler's pdftoppm on other platforms. The longest edge is scaled to max_edge.
pub fn render_pdf_page(
    path: impl AsRef<Path>,
    index: usize,
    max_edge: u32,
) -> Result<Vec<u8>, DeckError> {
    if max_edge == 0 {
        return Err(DeckError::Invalid("Pixel size must be positive".into()));
    }
    let path = dunce::canonicalize(path)?;
    let index =
        u32::try_from(index).map_err(|_| DeckError::Invalid("Page index is too large".into()))?;
    render(&path, index, max_edge)
}

#[cfg(windows)]
fn render(path: &Path, index: u32, max_edge: u32) -> Result<Vec<u8>, DeckError> {
    use windows::{
        core::HSTRING,
        Data::Pdf::{PdfDocument, PdfPageRenderOptions},
        Graphics::Imaging::BitmapEncoder,
        Storage::{
            StorageFile,
            Streams::{DataReader, InMemoryRandomAccessStream},
        },
    };
    let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path.as_os_str()))?.get()?;
    let document = PdfDocument::LoadFromFileAsync(&file)?.get()?;
    let page = document.GetPage(index)?;
    let size = page.Size()?;
    let options = PdfPageRenderOptions::new()?;
    if size.Width >= size.Height {
        options.SetDestinationWidth(max_edge)?;
    } else {
        options.SetDestinationHeight(max_edge)?;
    }
    options.SetBitmapEncoderId(BitmapEncoder::PngEncoderId()?)?;
    let stream = InMemoryRandomAccessStream::new()?;
    page.RenderWithOptionsToStreamAsync(&stream, &options)?
        .get()?;
    let length = u32::try_from(stream.Size()?)
        .map_err(|_| DeckError::Renderer("Rendered image is too large".into()))?;
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0)?)?;
    let loaded = reader.LoadAsync(length)?.get()?;
    if loaded != length {
        return Err(DeckError::Renderer("Incomplete rendered image".into()));
    }
    let mut bytes = vec![0; length as usize];
    reader.ReadBytes(&mut bytes)?;
    Ok(bytes)
}

#[cfg(not(windows))]
fn render(path: &Path, index: u32, max_edge: u32) -> Result<Vec<u8>, DeckError> {
    use std::process::Command;
    let page = (u64::from(index) + 1).to_string();
    let work = tempfile::tempdir()?;
    let mut command = Command::new("pdftoppm");
    command
        .args([
            "-f",
            &page,
            "-l",
            &page,
            "-scale-to",
            &max_edge.to_string(),
            "-singlefile",
            "-png",
        ])
        .arg(path)
        .arg(work.path().join("page"));
    super::office::run_renderer(command, "Poppler pdftoppm")?;
    Ok(std::fs::read(work.path().join("page.png"))?)
}
