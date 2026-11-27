use std::fs;
use std::path::{Path, PathBuf};

use cp_core::SongTranslation;
use cp_library::{
    load_songs_from_directories, parse_song_content, parse_song_file, write_song_file, SongFile,
};

fn parse(content: &str, path: &str) -> SongFile {
    parse_song_content(content, Path::new(path), "Hymnal")
}

#[test]
fn parses_known_header_fields_and_preserves_unknown_fields() {
    let song = parse(
        "---\nAUTHOR: John Newton\ncomposer: Traditional\ntune: NEW BRITAIN\nccli: 22025\ncopyright: Public domain: 1779\nstray note\n---\n[Primary]\ntitle: Amazing Grace\n\nAmazing grace\n",
        "Amazing Grace.song",
    );

    assert_eq!(song.song.author, "John Newton");
    assert_eq!(song.song.composer, "Traditional");
    assert_eq!(song.song.tune, "NEW BRITAIN");
    assert_eq!(song.song.ccli_number, "22025");
    assert_eq!(song.song.title, "Amazing Grace");
    assert_eq!(song.song.lyrics, vec![String::from("Amazing grace")]);
    assert_eq!(song.song.songbook, "Hymnal");
    assert_eq!(
        song.extra_header.get("copyright").map(String::as_str),
        Some("Public domain: 1779")
    );
}

#[test]
fn keeps_lyric_section_headers_and_inner_blank_lines() {
    let song = parse(
        "stray text\n[Primary]\ntitle: Test\n\n[Verse 1]\nFirst line\n\n[Chorus]\nChorus line\n\n",
        "Test.song",
    );

    assert_eq!(
        song.song.lyrics,
        vec![
            String::from("[Verse 1]"),
            String::from("First line"),
            String::new(),
            String::from("[Chorus]"),
            String::from("Chorus line"),
        ]
    );
}

#[test]
fn parses_all_extra_language_slots_and_mirrors_the_secondary_language() {
    let song = parse(
        "[Primary]\ntitle: Amazing Grace\n\nAmazing grace\n\n[Secondary: Spanish]\ntitle: Sublime gracia\n\nSublime gracia\n\n[Translation 3: Ukrainian]\ntitle: Дивна благодать\n\nДивна благодать\n\n[Translation 4: Russian]\ntitle: Удивительная благодать\n\nУдивительная благодать\n",
        "Amazing Grace.song",
    );

    assert_eq!(song.song.title, "Amazing Grace");
    assert_eq!(song.song.secondary_title, "Sublime gracia");
    assert_eq!(
        song.song.secondary_lyrics,
        vec![String::from("Sublime gracia")]
    );
    assert_eq!(
        song.song.translations,
        vec![
            SongTranslation {
                label: String::from("Spanish"),
                title: String::from("Sublime gracia"),
                lyrics: vec![String::from("Sublime gracia")],
            },
            SongTranslation {
                label: String::from("Ukrainian"),
                title: String::from("Дивна благодать"),
                lyrics: vec![String::from("Дивна благодать")],
            },
            SongTranslation {
                label: String::from("Russian"),
                title: String::from("Удивительная благодать"),
                lyrics: vec![String::from("Удивительная благодать")],
            },
        ]
    );
}

#[test]
fn ignores_out_of_range_translation_sections_until_the_next_supported_section() {
    let song = parse(
        "[Primary]\ntitle: Main\n\nMain line\n\n[Translation 5]\ntitle: Unsupported\n\nMust be ignored\n\n[Secondary]\ntitle: Second\n\nSecond line\n",
        "Main.song",
    );

    assert_eq!(song.song.lyrics, vec![String::from("Main line")]);
    assert_eq!(song.song.secondary_title, "Second");
    assert_eq!(
        song.song.secondary_lyrics,
        vec![String::from("Second line")]
    );
    assert_eq!(song.song.translations.len(), 1);
}

#[test]
fn filename_numbers_preserve_legacy_padding_and_normalize_unhyphenated_prefixes() {
    let hyphenated = parse("[Primary]\ntitle: Grace\n", "0007 - Grace.song");
    let unhyphenated = parse("[Primary]\ntitle: Grace\n", "0007 Grace.song");
    let embedded = parse("[Primary]\ntitle: Shepherd\n", "Psalm 23 - Shepherd.song");

    assert_eq!(hyphenated.song.number, "0007");
    assert_eq!(unhyphenated.song.number, "7");
    assert!(embedded.song.number.is_empty());
}

#[test]
fn write_then_read_preserves_song_data_and_unknown_header_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path: PathBuf = dir.path().join("0001 - Grace.song");
    let source = parse(
        "---\nauthor: John Newton\nrights: freely usable\n---\n[Primary]\ntitle: Amazing Grace\n\nAmazing grace\n\n[Secondary: Spanish]\ntitle: Sublime gracia\n\nSublime gracia\n",
        path.to_str().unwrap(),
    );

    write_song_file(&source, &path).unwrap();
    let loaded = parse_song_file(&path, "Hymnal").unwrap();

    assert_eq!(loaded.song, source.song);
    assert_eq!(loaded.extra_header, source.extra_header);
}

#[test]
fn reading_a_missing_song_file_returns_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.song");

    assert!(parse_song_file(&missing, "Hymnal").is_err());
}

#[test]
fn loads_nested_and_overlapping_roots_once_with_songbook_paths_relative_to_first_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let hymnal = root.join("Hymnal");
    let kids = hymnal.join("Kids");
    fs::create_dir_all(&kids).unwrap();

    let root_song_path = root.join("001 - Root.song");
    let hymnal_song_path = hymnal.join("002 - Hymn.song");
    let kids_song_path = kids.join("003 - Child.song");
    fs::write(&root_song_path, "[Primary]\ntitle: Root\n").unwrap();
    fs::write(&hymnal_song_path, "[Primary]\ntitle: Hymn\n").unwrap();
    fs::write(&kids_song_path, "[Primary]\ntitle: Child\n").unwrap();

    let songs = load_songs_from_directories(&[root.to_path_buf(), hymnal]);

    assert_eq!(songs.len(), 3);
    let root_song = songs.iter().find(|song| song.song.title == "Root").unwrap();
    let hymnal_song = songs.iter().find(|song| song.song.title == "Hymn").unwrap();
    let kids_song = songs
        .iter()
        .find(|song| song.song.title == "Child")
        .unwrap();
    assert_eq!(root_song.song.songbook, "");
    assert_eq!(hymnal_song.song.songbook, "Hymnal");
    assert_eq!(kids_song.song.songbook, "Hymnal/Kids");
    assert_eq!(
        hymnal_song.song.source_file,
        hymnal_song_path.to_string_lossy().into_owned()
    );
}

#[test]
fn reads_utf8_song_files_with_a_bom() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bom.song");
    let content = "[Primary]\ntitle: Благодать\n\nДивна благодать\n";
    let mut bytes = b"\xef\xbb\xbf".to_vec();
    bytes.extend_from_slice(content.as_bytes());
    fs::write(&path, bytes).unwrap();

    let song = parse_song_file(&path, "Hymnal").unwrap();

    assert_eq!(song.song.title, "Благодать");
    assert_eq!(song.song.lyrics, vec![String::from("Дивна благодать")]);
}

#[test]
fn falls_back_to_windows_1251_for_legacy_cyrillic_song_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cyrillic.song");
    let content = "[Primary]\ntitle: Слава\n\nСлава Богу\n";
    let (encoded, _, had_errors) = encoding_rs::WINDOWS_1251.encode(content);
    assert!(!had_errors);
    fs::write(&path, encoded.as_ref()).unwrap();

    let song = parse_song_file(&path, "Hymnal").unwrap();

    assert_eq!(song.song.title, "Слава");
    assert_eq!(song.song.lyrics, vec![String::from("Слава Богу")]);
}
