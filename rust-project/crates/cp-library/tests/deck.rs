use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use cp_library::deck::load_deck;
use image::codecs::jpeg::JpegEncoder;
use image::ExtendedColorType;
use lopdf::{dictionary, Document, Object, Stream, StringFormat};
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn write_pptx(path: &Path, entries: &[(&str, &[u8])]) {
    let file = File::create(path).expect("create pptx fixture");
    let mut archive = ZipWriter::new(file);

    for (name, contents) in entries {
        archive
            .start_file(*name, SimpleFileOptions::default())
            .expect("start pptx entry");
        archive.write_all(contents).expect("write pptx entry");
    }

    archive.finish().expect("finish pptx archive");
}

fn simple_slide(text: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
       xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld><p:spTree>
    <p:sp><p:nvSpPr><p:cNvPr id="2" name="Text"/></p:nvSpPr>
      <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr/><a:t>{text}</a:t></a:r></a:p></p:txBody>
    </p:sp>
  </p:spTree></p:cSld>
</p:sld>"#
    )
}

fn first_text(deck: &cp_library::deck::Deck, slide_index: usize) -> &str {
    deck.slides[slide_index].shapes[0].paragraphs[0][0].as_str()
}

fn jpeg_fixture(rgb: [u8; 3]) -> Vec<u8> {
    let mut bytes = Vec::new();
    JpegEncoder::new(&mut bytes)
        .encode(&rgb, 1, 1, ExtendedColorType::Rgb8)
        .expect("encode JPEG fixture");
    bytes
}

#[test]
fn loads_slides_in_presentation_relationship_order() {
    let temp = tempdir().expect("create temporary directory");
    let path = temp.path().join("ordered.ppsx");
    let presentation = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
 xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldIdLst><p:sldId id="257" r:id="rIdSecond"/><p:sldId id="256" r:id="rIdFirst"/></p:sldIdLst>
</p:presentation>"#;
    let presentation_rels = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdFirst" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide2.xml"/>
  <Relationship Id="rIdSecond" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide7.xml"/>
</Relationships>"#;
    let slide_two = simple_slide("relationship second");
    let slide_seven = simple_slide("relationship first");

    write_pptx(
        &path,
        &[
            ("ppt/presentation.xml", presentation),
            ("ppt/_rels/presentation.xml.rels", presentation_rels),
            ("ppt/slides/slide2.xml", slide_two.as_bytes()),
            ("ppt/slides/slide7.xml", slide_seven.as_bytes()),
        ],
    );

    let deck = load_deck(&path).expect("load presentation");

    assert_eq!(deck.slides.len(), 2);
    assert_eq!(first_text(&deck, 0), "relationship first");
    assert_eq!(first_text(&deck, 1), "relationship second");
}

#[test]
fn falls_back_to_numeric_slide_filename_order_without_relationships() {
    let temp = tempdir().expect("create temporary directory");
    let path = temp.path().join("numeric-fallback.pptx");
    let slide_one = simple_slide("slide one");
    let slide_two = simple_slide("slide two");
    let slide_ten = simple_slide("slide ten");

    write_pptx(
        &path,
        &[
            ("ppt/slides/slide10.xml", slide_ten.as_bytes()),
            ("ppt/slides/slide2.xml", slide_two.as_bytes()),
            ("ppt/slides/slide1.xml", slide_one.as_bytes()),
        ],
    );

    let deck = load_deck(&path).expect("load presentation with numeric fallback");

    assert_eq!(deck.slides.len(), 3);
    assert_eq!(first_text(&deck, 0), "slide one");
    assert_eq!(first_text(&deck, 1), "slide two");
    assert_eq!(first_text(&deck, 2), "slide ten");
}

#[test]
fn groups_text_runs_by_paragraph_and_decodes_xml_entities() {
    let temp = tempdir().expect("create temporary directory");
    let path = temp.path().join("text.pptx");
    let slide = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
       xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld><p:spTree>
    <p:sp><p:nvSpPr><p:cNvPr id="2" name="Grouped text"/></p:nvSpPr><p:txBody>
      <a:bodyPr/><a:lstStyle/>
      <a:p><a:r><a:rPr/><a:t>Open </a:t></a:r><a:r><a:rPr/><a:t>source &amp; docs</a:t></a:r></a:p>
      <a:p><a:r><a:rPr/><a:t>&lt;second line&gt;</a:t></a:r></a:p>
    </p:txBody></p:sp>
    <p:sp><p:nvSpPr><p:cNvPr id="3" name="Separate shape"/></p:nvSpPr><p:txBody>
      <a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr/><a:t>Separate shape text</a:t></a:r></a:p>
    </p:txBody></p:sp>
  </p:spTree></p:cSld>
</p:sld>"#;

    write_pptx(&path, &[("ppt/slides/slide1.xml", slide.as_bytes())]);
    let deck = load_deck(&path).expect("load text presentation");

    assert_eq!(deck.slides.len(), 1);
    let shapes = &deck.slides[0].shapes;
    assert_eq!(shapes.len(), 2);
    assert_eq!(
        shapes[0].paragraphs,
        vec![
            vec!["Open ".to_owned(), "source & docs".to_owned()],
            vec!["<second line>".to_owned()],
        ]
    );
    assert_eq!(
        shapes[1].paragraphs,
        vec![vec!["Separate shape text".to_owned()]]
    );
}

#[test]
fn associates_slide_images_with_relationship_targets_and_background_flag() {
    let temp = tempdir().expect("create temporary directory");
    let path = temp.path().join("images.pptx");
    let slide = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
       xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:bg><p:bgPr><a:blipFill><a:blip r:embed="rIdBackground"/><a:stretch><a:fillRect/></a:stretch></a:blipFill></p:bgPr></p:bg>
    <p:spTree>
      <p:sp><p:nvSpPr><p:cNvPr id="2" name="Slide text"/></p:nvSpPr>
        <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr/><a:t>slide text with images</a:t></a:r></a:p></p:txBody>
      </p:sp>
      <p:pic><p:nvPicPr><p:cNvPr id="3" name="Foreground picture"/></p:nvPicPr>
        <p:blipFill><a:blip r:embed="rIdPicture"/></p:blipFill><p:spPr/>
      </p:pic>
    </p:spTree>
  </p:cSld>
</p:sld>"#;
    let slide_rels = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdPicture" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/foreground.jpg"/>
  <Relationship Id="rIdBackground" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/background.jpg"/>
</Relationships>"#;
    let foreground_bytes = jpeg_fixture([240, 30, 20]);
    let background_bytes = jpeg_fixture([20, 30, 240]);
    let other_slide = simple_slide("separate slide text");

    write_pptx(
        &path,
        &[
            ("ppt/slides/slide1.xml", slide),
            ("ppt/slides/_rels/slide1.xml.rels", slide_rels),
            ("ppt/slides/slide2.xml", other_slide.as_bytes()),
            ("ppt/media/foreground.jpg", foreground_bytes.as_slice()),
            ("ppt/media/background.jpg", background_bytes.as_slice()),
        ],
    );

    let deck = load_deck(&path).expect("load image presentation");

    assert_eq!(deck.slides.len(), 2);
    assert_eq!(first_text(&deck, 0), "slide text with images");
    let images = &deck.slides[0].images;
    assert_eq!(images.len(), 2);
    let foreground = images
        .iter()
        .find(|image| !image.is_background)
        .expect("foreground image");
    let background = images
        .iter()
        .find(|image| image.is_background)
        .expect("background image");
    assert_eq!(foreground.bytes.as_ref(), foreground_bytes.as_slice());
    assert_eq!(background.bytes.as_ref(), background_bytes.as_slice());
    assert_eq!(first_text(&deck, 1), "separate slide text");
    assert!(deck.slides[1].images.is_empty());
}

#[test]
fn inherits_master_background_through_layout_unless_slide_overrides_it() {
    let temp = tempdir().expect("create temporary directory");
    let path = temp.path().join("background-inheritance.pptx");
    let inheriting_slide = simple_slide("inherits master background");
    let overriding_slide = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
       xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:bg><p:bgPr><a:blipFill><a:blip r:embed="rIdOverride"/><a:stretch><a:fillRect/></a:stretch></a:blipFill></p:bgPr></p:bg>
    <p:spTree>
      <p:sp><p:nvSpPr><p:cNvPr id="2" name="Slide text"/></p:nvSpPr>
        <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr/><a:t>overrides master background</a:t></a:r></a:p></p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>"#;
    let inheriting_slide_relationships = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdLayout" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#;
    let overriding_slide_relationships = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdLayout" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rIdOverride" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/slide-background.jpg"/>
</Relationships>"#;
    let layout = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
             xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld><p:spTree/></p:cSld>
</p:sldLayout>"#;
    let layout_relationships = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdMaster" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#;
    let master = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
             xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:bg><p:bgPr><a:blipFill><a:blip r:embed="rIdMasterBackground"/><a:stretch><a:fillRect/></a:stretch></a:blipFill></p:bgPr></p:bg>
    <p:spTree/>
  </p:cSld>
</p:sldMaster>"#;
    let master_relationships = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdMasterBackground" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/master-background.jpg"/>
</Relationships>"#;
    let master_background = jpeg_fixture([40, 150, 220]);
    let slide_background = jpeg_fixture([220, 120, 30]);

    write_pptx(
        &path,
        &[
            ("ppt/slides/slide1.xml", inheriting_slide.as_bytes()),
            ("ppt/slides/slide2.xml", overriding_slide),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                inheriting_slide_relationships,
            ),
            (
                "ppt/slides/_rels/slide2.xml.rels",
                overriding_slide_relationships,
            ),
            ("ppt/slideLayouts/slideLayout1.xml", layout),
            (
                "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
                layout_relationships,
            ),
            ("ppt/slideMasters/slideMaster1.xml", master),
            (
                "ppt/slideMasters/_rels/slideMaster1.xml.rels",
                master_relationships,
            ),
            (
                "ppt/media/master-background.jpg",
                master_background.as_slice(),
            ),
            (
                "ppt/media/slide-background.jpg",
                slide_background.as_slice(),
            ),
        ],
    );

    let deck = load_deck(&path).expect("load background inheritance presentation");

    assert_eq!(deck.slides.len(), 2);
    assert_eq!(first_text(&deck, 0), "inherits master background");
    assert_eq!(deck.slides[0].images.len(), 1);
    assert!(deck.slides[0].images[0].is_background);
    assert_eq!(
        deck.slides[0].images[0].bytes.as_ref(),
        master_background.as_slice()
    );

    assert_eq!(first_text(&deck, 1), "overrides master background");
    assert_eq!(deck.slides[1].images.len(), 1);
    assert!(deck.slides[1].images[0].is_background);
    assert_eq!(
        deck.slides[1].images[0].bytes.as_ref(),
        slide_background.as_slice()
    );
}

#[test]
fn orders_image_folder_slides_lexicographically_by_filename() {
    let temp = tempdir().expect("create temporary directory");
    let images_dir = temp.path().join("images");
    fs::create_dir(&images_dir).expect("create image folder");
    let image_one = jpeg_fixture([240, 30, 20]);
    let image_ten = jpeg_fixture([20, 240, 30]);
    let image_two = jpeg_fixture([20, 30, 240]);
    fs::write(images_dir.join("10.jpg"), &image_ten).expect("write image ten");
    fs::write(images_dir.join("2.jpg"), &image_two).expect("write image two");
    fs::write(images_dir.join("1.jpg"), &image_one).expect("write image one");

    let deck = load_deck(&images_dir).expect("load image folder");

    assert_eq!(deck.slides.len(), 3);
    assert_eq!(deck.slides[0].images.len(), 1);
    assert_eq!(
        deck.slides[0].images[0].bytes.as_ref(),
        image_one.as_slice()
    );
    assert_eq!(deck.slides[1].images.len(), 1);
    assert_eq!(
        deck.slides[1].images[0].bytes.as_ref(),
        image_ten.as_slice()
    );
    assert_eq!(deck.slides[2].images.len(), 1);
    assert_eq!(
        deck.slides[2].images[0].bytes.as_ref(),
        image_two.as_slice()
    );

    let direct_deck = load_deck(images_dir.join("1.jpg")).expect("load direct image file");
    assert_eq!(direct_deck.slides.len(), 1);
    let decoded = direct_deck.slides[0].images[0]
        .decode()
        .expect("decode direct image");
    assert_eq!((decoded.width(), decoded.height()), (1, 1));
}

#[test]
fn reads_pdf_title_metadata() {
    let temp = tempdir().expect("create temporary directory");
    let path = temp.path().join("metadata.pdf");
    let mut document = Document::with_version("1.5");

    let pages_id = document.new_object_id();
    let first_page_id = document.new_object_id();
    let second_page_id = document.new_object_id();
    let first_contents_id = document.add_object(Stream::new(
        dictionary! {},
        b"q 0.9 0.1 0.1 rg 10 10 80 130 re f Q".to_vec(),
    ));
    let second_contents_id = document.add_object(Stream::new(
        dictionary! {},
        b"q 0.1 0.1 0.9 rg 10 10 280 180 re f Q".to_vec(),
    ));
    let catalog_id = document.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    document.objects.insert(
        pages_id,
        dictionary! {
            "Type" => "Pages",
            "Kids" => vec![
                Object::Reference(first_page_id),
                Object::Reference(second_page_id),
            ],
            "Count" => Object::Integer(2),
        }
        .into(),
    );
    document.objects.insert(
        first_page_id,
        dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(100),
                Object::Integer(150),
            ],
            "Contents" => Object::Reference(first_contents_id),
            "Resources" => dictionary! {},
        }
        .into(),
    );
    document.objects.insert(
        second_page_id,
        dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(300),
                Object::Integer(200),
            ],
            "Contents" => Object::Reference(second_contents_id),
            "Resources" => dictionary! {},
        }
        .into(),
    );
    let info_id = document.add_object(dictionary! {
        "Title" => Object::String(b"Sample PDF deck".to_vec(), StringFormat::Literal),
    });
    document.trailer.set("Root", Object::Reference(catalog_id));
    document.trailer.set("Info", Object::Reference(info_id));
    document.save(&path).expect("write PDF fixture");

    let deck = load_deck(&path).expect("load PDF");

    let metadata = deck.pdf_metadata.as_ref().expect("PDF metadata");
    assert_eq!(metadata.title.as_deref(), Some("Sample PDF deck"));
    assert_eq!(metadata.page_count, 2);
    assert!((metadata.page_width_pt - 100.0).abs() < 0.01);
    assert!((metadata.page_height_pt - 150.0).abs() < 0.01);
    assert_eq!(deck.title.as_deref(), Some("Sample PDF deck"));

    let first_page_png = deck.render_page(0, 200).expect("render first PDF page");
    let first_page = image::load_from_memory(&first_page_png).expect("decode first-page PNG");
    assert!((first_page.width() as i32 - 133).abs() <= 1);
    assert_eq!(first_page.height(), 200);

    let second_page_png = deck.render_page(1, 200).expect("render second PDF page");
    let second_page = image::load_from_memory(&second_page_png).expect("decode second-page PNG");
    assert_eq!(second_page.width(), 200);
    assert!((second_page.height() as i32 - 133).abs() <= 1);
    assert!(deck.render_page(2, 200).is_err());
}
