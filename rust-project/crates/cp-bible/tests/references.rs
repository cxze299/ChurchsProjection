use cp_bible::{
    candidates, parse_reference, quick_passage_label, quick_passage_references,
    unique_quick_book_id, SmartReference,
};

fn reference(
    book_id: i32,
    chapter: Option<i32>,
    verse_start: Option<i32>,
    verse_end: Option<i32>,
) -> SmartReference {
    SmartReference {
        book_id,
        chapter,
        verse_start,
        verse_end,
    }
}

#[test]
fn all_66_pinyin_initials_have_the_expected_exact_candidate() {
    let books = [
        ("创世记", "Genesis", "csj", 1),
        ("出埃及记", "Exodus", "caj", 2),
        ("利未记", "Leviticus", "lwj", 3),
        ("民数记", "Numbers", "msj", 4),
        ("申命记", "Deuteronomy", "smj", 5),
        ("约书亚记", "Joshua", "ysyj", 6),
        ("士师记", "Judges", "ssj", 7),
        ("路得记", "Ruth", "ldj", 8),
        ("撒母耳记上", "1 Samuel", "smejs", 9),
        ("撒母耳记下", "2 Samuel", "smejx", 10),
        ("列王纪上", "1 Kings", "lwjs", 11),
        ("列王纪下", "2 Kings", "lwjx", 12),
        ("历代志上", "1 Chronicles", "ldzs", 13),
        ("历代志下", "2 Chronicles", "ldzx", 14),
        ("以斯拉记", "Ezra", "yslj", 15),
        ("尼希米记", "Nehemiah", "nxmj", 16),
        ("以斯帖记", "Esther", "ystj", 17),
        ("约伯记", "Job", "ybj", 18),
        ("诗篇", "Psalms", "sp", 19),
        ("箴言", "Proverbs", "zy", 20),
        ("传道书", "Ecclesiastes", "cds", 21),
        ("雅歌", "Song of Solomon", "yg", 22),
        ("以赛亚书", "Isaiah", "ysys", 23),
        ("耶利米书", "Jeremiah", "ylms", 24),
        ("耶利米哀歌", "Lamentations", "ylag", 25),
        ("以西结书", "Ezekiel", "yxjs", 26),
        ("但以理书", "Daniel", "dyls", 27),
        ("何西阿书", "Hosea", "hxs", 28),
        ("约珥书", "Joel", "yes", 29),
        ("阿摩司书", "Amos", "ams", 30),
        ("俄巴底亚书", "Obadiah", "ebdys", 31),
        ("约拿书", "Jonah", "yns", 32),
        ("弥迦书", "Micah", "mjs", 33),
        ("那鸿书", "Nahum", "nhs", 34),
        ("哈巴谷书", "Habakkuk", "hbgs", 35),
        ("西番雅书", "Zephaniah", "xfys", 36),
        ("哈该书", "Haggai", "hgs", 37),
        ("撒迦利亚书", "Zechariah", "sjlys", 38),
        ("玛拉基书", "Malachi", "mljs", 39),
        ("马太福音", "Matthew", "mtfy", 40),
        ("马可福音", "Mark", "mkfy", 41),
        ("路加福音", "Luke", "ljfy", 42),
        ("约翰福音", "John", "yhfy", 43),
        ("使徒行传", "Acts", "stxz", 44),
        ("罗马书", "Romans", "lms", 45),
        ("哥林多前书", "1 Corinthians", "gldqs", 46),
        ("哥林多后书", "2 Corinthians", "gldhs", 47),
        ("加拉太书", "Galatians", "jlts", 48),
        ("以弗所书", "Ephesians", "yfss", 49),
        ("腓立比书", "Philippians", "flbs", 50),
        ("歌罗西书", "Colossians", "glxs", 51),
        ("帖撒罗尼迦前书", "1 Thessalonians", "tsljqs", 52),
        ("帖撒罗尼迦后书", "2 Thessalonians", "tsljhs", 53),
        ("提摩太前书", "1 Timothy", "tmtqs", 54),
        ("提摩太后书", "2 Timothy", "tmths", 55),
        ("提多书", "Titus", "tds", 56),
        ("腓利门书", "Philemon", "flms", 57),
        ("希伯来书", "Hebrews", "xbls", 58),
        ("雅各书", "James", "ygs", 59),
        ("彼得前书", "1 Peter", "bdqs", 60),
        ("彼得后书", "2 Peter", "bdhs", 61),
        ("约翰一书", "1 John", "yhys", 62),
        ("约翰二书", "2 John", "yhes", 63),
        ("约翰三书", "3 John", "yhss", 64),
        ("犹大书", "Jude", "yds", 65),
        ("启示录", "Revelation", "qsl", 66),
    ];

    assert_eq!(books.len(), 66);
    for (chinese_name, english_name, initials, expected_id) in books {
        assert_eq!(
            candidates(initials).first().copied(),
            Some(expected_id),
            "initials {initials:?} should rank book id {expected_id} first"
        );
        assert_eq!(
            parse_reference(chinese_name),
            Some(reference(expected_id, None, None, None)),
            "Chinese name {chinese_name:?} should resolve to book id {expected_id}"
        );
        assert_eq!(
            parse_reference(english_name),
            Some(reference(expected_id, None, None, None)),
            "English name {english_name:?} should resolve to book id {expected_id}"
        );
    }
}

#[test]
fn initials_matching_supports_exact_prefix_subsequence_and_ambiguity() {
    assert_eq!(candidates(" LWJ "), [3, 11, 12]);
    assert!(candidates("c").is_empty());
    assert!(candidates("csj 12").is_empty());
    assert_eq!(candidates("csj").first().copied(), Some(1));
    assert_eq!(unique_quick_book_id("csj"), Some(1));
    assert_eq!(candidates("cs").first().copied(), Some(1));
    assert_eq!(unique_quick_book_id("cs"), None);
    assert!(candidates("cj").contains(&1));
    assert!(candidates("cj").contains(&2));
    assert_eq!(unique_quick_book_id("cj"), None);

    let ambiguous = candidates("yh");
    for expected_id in [43, 62, 63, 64] {
        assert!(ambiguous.contains(&expected_id));
    }
    assert_eq!(unique_quick_book_id("yh"), None);

    assert!(candidates("Genesis").is_empty());
    assert_eq!(unique_quick_book_id("Genesis"), None);
    assert!(candidates("创世记").is_empty());
    assert_eq!(unique_quick_book_id("创世记"), None);
}

#[test]
fn parser_normalizes_case_spacing_and_colon_spacing() {
    assert_eq!(
        parse_reference("  jOhN   3 : 16  "),
        Some(reference(43, Some(3), Some(16), None))
    );
}

#[test]
fn attached_quick_passage_parses_ranges_and_single_verses() {
    assert_eq!(
        quick_passage_references("csj 1 1 3Jhn 3 16"),
        Some(vec![
            reference(1, Some(1), Some(1), Some(3)),
            reference(43, Some(3), Some(16), Some(16)),
        ])
    );
}

#[test]
fn quick_passage_label_uses_full_chinese_names_and_reading_format() {
    assert_eq!(
        quick_passage_references("csj 1 1 3 yhfy 3 16 18"),
        quick_passage_references("csj 1 1 3yhfy 3 16 18"),
    );
    assert_eq!(
        quick_passage_label("csj 1 1 3yhfy 3 16 18"),
        Some("创世记 1章1-3节；约翰福音 3章16-18节".to_string())
    );
}

#[test]
fn reversed_quick_passage_ranges_are_normalized() {
    assert_eq!(
        quick_passage_references("csj 1 3-1"),
        Some(vec![reference(1, Some(1), Some(1), Some(3))])
    );
    for query in ["Genesis 1:3-1", "csj 1 3 1"] {
        assert_eq!(
            parse_reference(query),
            Some(reference(1, Some(1), Some(1), Some(3)))
        );
    }
}

#[test]
fn malformed_trailing_text_is_rejected() {
    assert_eq!(parse_reference("John 3:16xyz"), None);
    assert_eq!(quick_passage_references("csj 1 1 xyz"), None);
}

#[test]
fn chapter_and_verse_must_be_positive_and_fit_in_i32() {
    for malformed in [
        "John 0:16",
        "John 3:0",
        "John -1:16",
        "John 3:-16",
        "John 2147483648:16",
        "John 3:2147483648",
    ] {
        assert_eq!(parse_reference(malformed), None, "input {malformed:?}");
    }

    for malformed in ["csj 0 1", "csj 1 0", "csj 2147483648 1", "csj 1 2147483648"] {
        assert_eq!(
            quick_passage_references(malformed),
            None,
            "input {malformed:?}"
        );
    }
}

#[test]
fn quick_passage_parser_accepts_unlimited_attached_segments() {
    let build_passage = |count: usize| vec!["csj 1 1"; count].join(" ");
    let expected_reference = reference(1, Some(1), Some(1), Some(1));

    for count in [128, 129] {
        let passage = quick_passage_references(&build_passage(count)).unwrap();
        assert_eq!(passage.len(), count);
        assert!(passage.iter().all(|item| *item == expected_reference));
    }
}
