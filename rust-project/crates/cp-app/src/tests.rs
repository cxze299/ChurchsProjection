use crate::{
    bible::Browser,
    media::{Document, Library, Page},
};
use cp_bible::Bible;
use cp_core::{AppSettings, BibleTranslationSettings, ScheduleEntry};
use std::{
    fmt::Write as FmtWrite,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new() -> Self {
        let root = std::env::temp_dir();
        for _ in 0..1000 {
            let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!(
                "church-presenter-cp-app-test-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create temporary test directory: {error}"),
            }
        }
        panic!("could not find a unique temporary test directory name");
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bible_fixture(filename: &str, title: &str, verses: &[(i32, i32, &str)]) -> Bible {
    let chapter_count = verses
        .iter()
        .map(|(chapter, _, _)| *chapter)
        .max()
        .unwrap_or(1);
    let mut content = format!("##Title:{title}\n1 Genesis {chapter_count}\n-----\n");
    for (chapter, verse, text) in verses {
        writeln!(
            &mut content,
            "B001C{chapter:03}V{verse:03} 1 {chapter} {verse} {text}"
        )
        .unwrap();
    }
    Bible::parse_spb(&content, filename)
}

fn settings_with_translations(file_names: &[&str]) -> AppSettings {
    let mut settings = AppSettings::default();
    settings.bible_settings.translations = file_names
        .iter()
        .map(|file_name| BibleTranslationSettings {
            file_name: (*file_name).to_owned(),
            ..Default::default()
        })
        .collect();
    settings
}

#[test]
fn browser_continuous_edits_append_only_new_passages() {
    let bible = bible_fixture(
        "one.spb",
        "Translation One",
        &[
            (1, 1, "first verse"),
            (1, 2, "second verse"),
            (1, 3, "third verse"),
        ],
    );
    let mut browser = Browser::new(vec![bible], &settings_with_translations(&["one.spb"]));

    let first = browser
        .apply_quick("Genesis 1:1 Genesis 1:2", true)
        .expect("valid two-passage command should be accepted");
    assert_eq!(browser.accumulated.len(), 2);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].continuous_passages.len(), 2);
    assert!(first[0].verse_text.contains("Genesis 1:1\nfirst verse"));
    assert!(first[0].verse_text.contains("Genesis 1:2\nsecond verse"));

    let repeated = browser
        .apply_quick("Genesis 1:1 Genesis 1:2", true)
        .expect("repeated command should still publish accumulated content");
    assert_eq!(browser.accumulated.len(), 2);
    assert_eq!(repeated[0].continuous_passages.len(), 2);

    let extended = browser
        .apply_quick("Genesis 1:1 Genesis 1:2 Genesis 1:3", true)
        .expect("extending the command should append its new passage");
    assert_eq!(browser.accumulated.len(), 3);
    assert_eq!(extended[0].continuous_passages.len(), 3);
    assert!(extended[0].verse_text.contains("Genesis 1:3\nthird verse"));
}

#[test]
fn browser_invalid_multi_passage_command_is_atomic() {
    let bible = bible_fixture(
        "one.spb",
        "Translation One",
        &[
            (1, 1, "first verse"),
            (1, 2, "second verse"),
            (1, 3, "third verse"),
        ],
    );
    let mut browser = Browser::new(vec![bible], &settings_with_translations(&["one.spb"]));
    browser
        .apply_quick("Genesis 1:1 Genesis 1:2", true)
        .expect("initial command should succeed");

    let accumulated_before = browser.accumulated.clone();
    let selected_before = browser.selected();
    let location_before = (browser.book, browser.chapter);

    assert!(browser
        .apply_quick("Genesis 1:1 Genesis 1:99", true)
        .is_none());
    assert_eq!(browser.accumulated, accumulated_before);
    assert_eq!(browser.selected(), selected_before);
    assert_eq!((browser.book, browser.chapter), location_before);

    let recovered = browser
        .apply_quick("Genesis 1:1 Genesis 1:2 Genesis 1:3", true)
        .expect("valid command after rejection should succeed");
    assert_eq!(browser.accumulated.len(), 3);
    assert_eq!(recovered[0].continuous_passages.len(), 3);
}

#[test]
fn browser_uses_configured_primary_and_parallel_order_for_ranges() {
    let first = bible_fixture(
        "first.spb",
        "First Translation",
        &[
            (1, 1, "first 1"),
            (1, 2, "first 2"),
            (1, 3, "first 3"),
            (1, 4, "first 4"),
        ],
    );
    let second = bible_fixture(
        "second.spb",
        "Second Translation",
        &[
            (1, 1, "second 1"),
            (1, 2, "second 2"),
            (1, 3, "second 3"),
            (1, 4, "second 4"),
        ],
    );
    let settings =
        settings_with_translations(&["missing.spb", "second.spb", "first.spb", "second.spb"]);
    let mut browser = Browser::new(vec![first, second], &settings);

    assert_eq!(browser.primary().unwrap().filename(), "second.spb");
    let selected = browser
        .apply_quick("Genesis 1:2-4", false)
        .expect("complete verse range should be accepted");
    assert_eq!(
        selected
            .iter()
            .map(|verse| verse.translation_file_name.as_str())
            .collect::<Vec<_>>(),
        ["second.spb", "first.spb"]
    );
    assert_eq!(selected[0].verse_range, "2-4");
    assert_eq!(selected[0].verse_text, "second 2 second 3 second 4");
    assert_eq!(selected[1].verse_range, "2-4");
    assert_eq!(selected[1].verse_text, "first 2 first 3 first 4");
    assert_eq!(browser.selection.selected_rows(), vec![1, 2, 3]);
}

#[test]
fn library_searches_song_title_metadata_and_opens_song_slides() {
    let temp = TempDirectory::new();
    let songbook = temp.path().join("English Hymns");
    fs::create_dir_all(&songbook).unwrap();
    let song_path = songbook.join("042 - Tune Name.song");
    fs::write(
        &song_path,
        "---\nauthor: Isaac Watts\n---\n\
         [Primary]\n\
         title: Crown Him with Many Crowns\n\n\
         Crown Him with many crowns\n\
         The Lamb upon His throne\n\n\
         [Verse 2]\n\
         Awake, my soul, and sing\n\
         Of Him who died for thee\n",
    )
    .unwrap();

    let mut settings = AppSettings::default();
    settings.song_settings.storage_directory = temp.path().to_string_lossy().into_owned();
    let library = Library::load(&settings, &[]);
    let title_matches = library.find("Crown Him with Many Crowns", "", "song");
    assert_eq!(title_matches, vec![song_path.clone()]);
    assert_eq!(library.find("42", "", "song"), vec![song_path.clone()]);
    assert_eq!(
        library.title(&song_path),
        "042 - Crown Him with Many Crowns"
    );

    let document = Document::open(&title_matches[0], "English Hymns")
        .expect("selected song should open as a document");
    assert_eq!(document.len(), 2);
    assert_eq!(document.title(), "042 - Crown Him with Many Crowns");
    assert_eq!(document.page_label(0), "Crown Him with many crowns");
    assert_eq!(document.page_label(1), "Awake, my soul, and sing");

    match document.entry("song-id".into(), "English Hymns".into()) {
        ScheduleEntry::Song {
            number,
            title,
            songbook,
            ..
        } => {
            assert_eq!(number, "042");
            assert_eq!(title, "Crown Him with Many Crowns");
            assert_eq!(songbook, "English Hymns");
        }
        _ => panic!("song document should produce a song schedule entry"),
    }

    let mut page = Page {
        document: document.clone(),
        index: 0,
    };
    page.step(1);
    assert_eq!(page.index, 1);
    page.step(isize::MAX);
    assert_eq!(page.index, 1);
    page.step(-1);
    assert_eq!(page.index, 0);
    page.step(isize::MIN);
    assert_eq!(page.index, 0);
}

#[test]
fn presentation_document_loads_sorted_image_slides_and_clamps_navigation() {
    let temp = TempDirectory::new();
    let presentation = temp.path().join("Slides");
    fs::create_dir_all(&presentation).unwrap();
    for name in ["02.png", "01.png"] {
        let image = image::RgbaImage::from_pixel(2, 2, image::Rgba([12, 34, 56, 255]));
        image.save(presentation.join(name)).unwrap();
    }

    let document = Document::open(&presentation, "Slides")
        .expect("image directory should open as a presentation document");
    assert_eq!(document.len(), 2);
    assert_eq!(document.title(), "Slides");
    match document.as_ref() {
        Document::Deck(deck) => {
            assert_eq!(deck.slides.len(), 2);
            assert!(deck.slides[0].images[0].source.ends_with("01.png"));
            assert!(deck.slides[1].images[0].source.ends_with("02.png"));
        }
        Document::Song { .. } => panic!("image directory should open as a deck"),
    }

    let mut page = Page { document, index: 0 };
    page.step(isize::MAX);
    assert_eq!(page.index, 1);
    page.step(1);
    assert_eq!(page.index, 1);
    page.step(isize::MIN);
    assert_eq!(page.index, 0);
}

#[test]
fn appearance_palettes_resolve_names_and_selection_colors() {
    let light = crate::appearance::palette("Light");
    let dark = crate::appearance::palette("dark");
    let ocean = crate::appearance::palette("oCeAn");
    let fallback = crate::appearance::palette("unknown palette");

    assert_eq!(light.name, "Light");
    assert!(!light.dark);
    assert_eq!(dark.name, "Dark");
    assert!(dark.dark);
    assert_eq!(ocean.name, "Ocean");
    assert!(!ocean.dark);
    assert_eq!(fallback.name, "Dark");

    assert_eq!(light.selection(), light.primary.gamma_multiply(0.10));
    assert_eq!(dark.selection(), dark.primary.gamma_multiply(0.18));
}

#[test]
fn picture_scan_recurses_sorts_and_deduplicates_roots() {
    use crate::pictures_ui::scan_pictures;

    let temp = TempDirectory::new();
    let root = temp.path().join("Pictures");
    let nested = root.join("Nested");
    fs::create_dir_all(&nested).unwrap();
    for name in ["a.jpg", "b.PNG", "ignored.gif"] {
        fs::write(root.join(name), b"picture fixture").unwrap();
    }
    fs::write(nested.join("c.JpEg"), b"picture fixture").unwrap();

    let mut expected = vec![
        root.join("a.jpg"),
        root.join("b.PNG"),
        nested.join("c.JpEg"),
    ];
    expected.sort();
    let found = scan_pictures(&[root.clone(), nested, root]);

    assert_eq!(found, expected);
}

#[test]
fn countdown_pauses_resumes_resets_and_expires() {
    use crate::announcements_ui::Countdown;
    use std::time::{Duration, Instant};

    let start = Instant::now();
    let mut countdown = Countdown::default();
    countdown.set_seconds(12);
    assert_eq!(countdown.remaining(start), 12);

    countdown.toggle(start);
    assert_eq!(countdown.remaining(start + Duration::from_secs(3)), 9);

    countdown.toggle(start + Duration::from_secs(3));
    assert_eq!(countdown.remaining(start + Duration::from_secs(8)), 9);

    countdown.toggle(start + Duration::from_secs(8));
    assert_eq!(countdown.remaining(start + Duration::from_secs(11)), 6);

    countdown.reset();
    assert_eq!(countdown.remaining(start + Duration::from_secs(30)), 12);

    countdown.set_seconds(2);
    countdown.toggle(start + Duration::from_secs(30));
    assert_eq!(countdown.remaining(start + Duration::from_secs(32)), 0);
    assert_eq!(countdown.remaining(start + Duration::from_secs(40)), 0);
}

#[test]
fn workspace_renders_each_tab_headlessly() {
    use crate::app::{App, Tab};
    use cp_core::SettingsManager;
    use egui::{pos2, vec2, Context, RawInput, Rect};

    let temp = TempDirectory::new();
    let context = Context::default();
    let mut app = App::new_with_manager(
        &context,
        Vec::new(),
        SettingsManager::with_dir(temp.path().to_owned()),
    );
    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(1280., 800.))),
        ..Default::default()
    };

    for tab in [
        Tab::Bible,
        Tab::Songs,
        Tab::Library,
        Tab::Pictures,
        Tab::Announcements,
        Tab::Settings,
    ] {
        app.change_tab(tab);
        let output = context.run(input.clone(), |ctx| app.workspace(ctx));
        assert!(
            !output.shapes.is_empty(),
            "workspace should create UI shapes for every tab"
        );
    }
}
