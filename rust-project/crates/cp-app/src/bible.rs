use cp_bible::{
    append_passage, publish_accumulated_passages, resolve_quick_passages, resolve_selection, Bible,
    BibleVerse, SmartReference, VerseSelection, CHINESE_BOOK_NAMES,
};
use cp_core::{AppSettings, ResourceDirectory, SelectedVerse};
use std::{collections::HashSet, fs, path::PathBuf};

const SAMPLE: &str =
    r"D:\churchpre\project\composeApp\src\jvmMain\composeResources\files\bible_samples\kjv1769.spb";

pub fn load_modules(settings: &AppSettings) -> (Vec<Bible>, Vec<String>) {
    let mut paths = Vec::new();
    for dir in settings.resource_directories(ResourceDirectory::Bible) {
        let mut files: Vec<_> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("spb")))
            .collect();
        files.sort();
        paths.extend(files);
    }
    if paths.is_empty() && std::path::Path::new(SAMPLE).is_file() {
        paths.push(PathBuf::from(SAMPLE));
    }
    let mut names = HashSet::new();
    let mut modules = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if names.contains(&name.to_lowercase()) {
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(data) => {
                let bible = Bible::parse_spb_with_names(&data, &name, &CHINESE_BOOK_NAMES);
                if bible.books().is_empty() {
                    errors.push(format!("{} 缺少经文", path.display()));
                } else {
                    names.insert(name.to_lowercase());
                    modules.push(bible);
                }
            }
            Err(e) => errors.push(format!("{} · {e}", path.display())),
        }
    }
    (modules, errors)
}

#[derive(Default)]
pub struct Browser {
    pub modules: Vec<Bible>,
    pub stack: Vec<usize>,
    pub book: i32,
    pub chapter: i32,
    pub rows: Vec<BibleVerse>,
    pub selection: VerseSelection,
    pub accumulated: Vec<Vec<SelectedVerse>>,
    // An edited command can retain already published segments at its beginning.
    committed: Vec<SmartReference>,
}

impl Browser {
    pub fn new(modules: Vec<Bible>, settings: &AppSettings) -> Self {
        let mut stack = Vec::new();
        for style in settings.bible_settings.effective_translations() {
            if let Some(i) = modules
                .iter()
                .position(|b| b.filename().eq_ignore_ascii_case(&style.file_name))
            {
                if !stack.contains(&i) {
                    stack.push(i);
                }
            }
        }
        if stack.is_empty() && !modules.is_empty() {
            stack.push(0);
        }
        let mut this = Self {
            modules,
            stack,
            ..Default::default()
        };
        this.set_location(1, 1);
        this
    }
    pub fn primary(&self) -> Option<&Bible> {
        self.stack.first().map(|&i| &self.modules[i])
    }
    fn parallel(&self) -> Vec<&Bible> {
        self.stack
            .iter()
            .skip(1)
            .map(|&i| &self.modules[i])
            .collect()
    }
    pub fn set_location(&mut self, book: i32, chapter: i32) {
        let Some(bible) = self.primary() else { return };
        let Some(b) = bible.book(book).or_else(|| bible.books().first()) else {
            return;
        };
        let (book, chapter) = (b.book_id, chapter.clamp(1, b.chapter_count.max(1)));
        self.rows = bible.get_chapter(book, chapter);
        self.book = book;
        self.chapter = chapter;
        self.selection = VerseSelection::new(self.rows.len());
    }
    pub fn selected(&self) -> Vec<SelectedVerse> {
        self.primary()
            .map(|b| {
                resolve_selection(
                    b,
                    &self.parallel(),
                    self.book,
                    self.chapter,
                    &self.selection.selected_rows(),
                )
            })
            .unwrap_or_default()
    }
    pub fn select_numbers(&mut self, book: i32, chapter: i32, first: i32, last: i32) {
        self.set_location(book, chapter);
        if let Some(i) = self.rows.iter().position(|v| v.verse_number == first) {
            self.selection.click_verse(i);
            if let Some(j) = self.rows.iter().position(|v| v.verse_number == last) {
                self.selection.shift_click_verse(j);
            }
        }
    }

    pub fn restore(&mut self, verses: &[SelectedVerse]) {
        let Some(first) = verses.first() else { return };
        let passages = first
            .continuous_passages
            .last()
            .map(Vec::as_slice)
            .unwrap_or(verses);
        let Some(first) = passages.first() else {
            return;
        };
        let stack: Vec<_> = passages
            .iter()
            .filter_map(|v| {
                self.modules
                    .iter()
                    .position(|b| b.filename() == v.translation_file_name)
            })
            .collect();
        if !stack.is_empty() {
            self.stack = stack;
        }
        self.set_location(first.book_id, first.chapter);
        let ranges: Vec<_> = first
            .verse_range
            .split(',')
            .filter_map(|s| {
                let s = s.trim();
                let (a, b) = s.split_once('-').unwrap_or((s, s));
                Some((a.trim().parse::<i32>().ok()?, b.trim().parse::<i32>().ok()?))
            })
            .collect();
        let mut indices = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                v.verse_number == first.verse_number
                    || ranges
                        .iter()
                        .any(|&(a, b)| (a..=b).contains(&v.verse_number))
            })
            .map(|(i, _)| i);
        if let Some(i) = indices.next() {
            self.selection.click_verse(i);
            for j in indices {
                self.selection.ctrl_click_verse(j);
            }
        }
        self.forget_command();
    }

    pub fn step(&mut self, delta: isize) -> bool {
        let selected = self.selection.selected_rows();
        let forward = delta > 0;
        let Some(&row) = (if forward {
            selected.last()
        } else {
            selected.first()
        }) else {
            return false;
        };
        let row = if let Some(next) = row
            .checked_add_signed(delta)
            .filter(|&i| i < self.rows.len())
        {
            next
        } else {
            let bible = self.primary().unwrap();
            let books = bible.books();
            let Some(mut i) = books.iter().position(|b| b.book_id == self.book) else {
                return false;
            };
            let mut chapter = self.chapter;
            // Some modules omit chapters. Walk to the nearest chapter with displayable verses.
            let (book, chapter) = loop {
                chapter += if forward { 1 } else { -1 };
                if chapter > books[i].chapter_count {
                    if i + 1 >= books.len() {
                        return false;
                    }
                    i += 1;
                    chapter = 1;
                } else if chapter < 1 {
                    if i == 0 {
                        return false;
                    }
                    i -= 1;
                    chapter = books[i].chapter_count;
                }
                if !bible
                    .get_chapter_verses(books[i].book_id, chapter)
                    .is_empty()
                {
                    break (books[i].book_id, chapter);
                }
            };
            self.set_location(book, chapter);
            if forward {
                0
            } else {
                self.rows.len() - 1
            }
        };
        self.selection = VerseSelection::new(self.rows.len());
        self.selection.click_verse(row);
        true
    }
    pub fn apply_quick(&mut self, input: &str, continuous: bool) -> Option<Vec<SelectedVerse>> {
        let bible = self.primary()?;
        let refs = bible.quick_passage_references(input)?;
        if !continuous && refs.len() != 1 {
            return None;
        }
        let groups = resolve_quick_passages(bible, &self.parallel(), &refs)?;
        let last = refs.last()?;
        self.select_numbers(
            last.book_id,
            last.chapter?,
            last.verse_start?,
            last.verse_end?,
        );
        if !continuous {
            return groups.into_iter().next();
        }
        let common = self
            .committed
            .iter()
            .zip(&refs)
            .take_while(|(a, b)| a == b)
            .count();
        for group in groups.into_iter().skip(common) {
            append_passage(&mut self.accumulated, group);
        }
        self.committed = refs;
        Some(publish_accumulated_passages(&self.accumulated))
    }
    pub fn forget_command(&mut self) {
        self.committed.clear();
    }
    pub fn append_selected(&mut self) -> Vec<SelectedVerse> {
        self.forget_command();
        let group = self.selected();
        append_passage(&mut self.accumulated, group);
        publish_accumulated_passages(&self.accumulated)
    }
    pub fn clear(&mut self) {
        self.accumulated.clear();
        self.forget_command();
    }
}
