use cp_core::{AppSettings, LyricSlide, ResourceDirectory, ScheduleEntry, SongItem};
use cp_library::{
    discover, load_deck, load_songs_from_directories, parse_song_file, quick_play, Deck,
};
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
};

pub fn extension(path: &Path) -> String {
    path.extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase()
}
#[derive(Default)]
pub struct Library {
    pub roots: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
    pub books: Vec<String>,
    pub songs: HashMap<PathBuf, SongItem>,
}

impl Library {
    pub fn load(settings: &AppSettings, added: &[PathBuf]) -> Self {
        let mut roots: Vec<PathBuf> = settings
            .resource_directories(ResourceDirectory::Presentations)
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let presentation_count = roots.len();
        roots.extend(
            settings
                .resource_directories(ResourceDirectory::Songs)
                .into_iter()
                .map(PathBuf::from),
        );
        let mut files = discover(&roots);
        for path in added {
            if path.is_file() && !files.contains(path) {
                files.push(path.clone());
            }
        }
        let books: BTreeSet<_> = files
            .iter()
            .map(|p| quick_play::book(p, &roots[..presentation_count]))
            .filter(|s| !s.is_empty())
            .collect();
        let mut songs: HashMap<_, _> = load_songs_from_directories(&roots)
            .into_iter()
            .map(|s| (PathBuf::from(&s.song.source_file), s.song))
            .collect();
        for path in added {
            if extension(path) == "song" && !songs.contains_key(path) {
                if let Ok(item) = parse_song_file(path, &quick_play::book(path, &roots)) {
                    songs.insert(path.clone(), item.song);
                }
            }
        }
        Self {
            roots,
            files,
            books: books.into_iter().collect(),
            songs,
        }
    }

    pub fn find(&self, query: &str, book: &str, format: &str) -> Vec<PathBuf> {
        let mut matches = quick_play::find(&self.files, &self.roots, query, book, format);
        // Song headers may contain a different hymn number or title from the filename.
        let query = query.trim();
        if !query.is_empty() {
            let numeric = query.parse::<u32>().ok();
            let lower = query.to_lowercase();
            let mut extra: Vec<_> = self
                .songs
                .iter()
                .filter(|(path, s)| {
                    (book.is_empty() || quick_play::book(path, &self.roots) == book)
                        && match numeric {
                            Some(n) => s.number.parse::<u32>().ok() == Some(n),
                            None => {
                                s.title.to_lowercase().contains(&lower)
                                    || cp_library::pinyin_match::matches(&s.title, query)
                            }
                        }
                })
                .map(|(p, _)| p.clone())
                .collect();
            extra.sort();
            for p in extra {
                if !matches.contains(&p) {
                    matches.push(p);
                }
            }
        }
        matches
    }
    pub fn title(&self, path: &Path) -> String {
        self.songs
            .get(path)
            .map(SongItem::display_title)
            .unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            })
    }
}

pub enum Document {
    Song {
        song: SongItem,
        slides: Vec<LyricSlide>,
    },
    Deck(Deck),
}

impl Document {
    pub fn open(path: &Path, book: &str) -> Result<Arc<Self>, String> {
        if extension(path) == "song" {
            let song = parse_song_file(path, book).map_err(|e| e.to_string())?.song;
            let slides = song.lyric_slides();
            if slides.is_empty() {
                return Err("诗歌文件中没有歌词".into());
            }
            Ok(Arc::new(Self::Song { song, slides }))
        } else {
            load_deck(path)
                .map(Self::Deck)
                .map(Arc::new)
                .map_err(|e| e.to_string())
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Song { slides, .. } => slides.len(),
            Self::Deck(d) => d.slides.len(),
        }
    }
    pub fn title(&self) -> String {
        match self {
            Self::Song { song, .. } => song.display_title(),
            Self::Deck(d) => d.title.clone().unwrap_or_default(),
        }
    }
    pub fn page_label(&self, page: usize) -> String {
        match self {
            Self::Song { slides, .. } => slides[page].lines.first().cloned().unwrap_or_default(),
            Self::Deck(d) => d.slides[page]
                .shapes
                .iter()
                .flat_map(|s| &s.paragraphs)
                .map(|p| p.concat())
                .find(|s| !s.trim().is_empty())
                .unwrap_or_default(),
        }
    }
    pub fn entry(&self, id: String, book: String) -> ScheduleEntry {
        match self {
            Self::Song { song, .. } => ScheduleEntry::Song {
                id,
                number: song.number.clone(),
                title: song.title.clone(),
                songbook: song.songbook.clone(),
                source_file: song.source_file.clone(),
            },
            Self::Deck(d) => ScheduleEntry::Presentation {
                id,
                file_path: d.source.display().to_string(),
                file_name: self.title(),
                book,
                slide_count: self.len(),
            },
        }
    }
    pub fn path(&self) -> &Path {
        match self {
            Self::Song { song, .. } => Path::new(&song.source_file),
            Self::Deck(d) => &d.source,
        }
    }
}

#[derive(Clone)]
pub struct Page {
    pub document: Arc<Document>,
    pub index: usize,
}
impl Page {
    pub fn step(&mut self, delta: isize) {
        self.index = self
            .index
            .saturating_add_signed(delta)
            .min(self.document.len().saturating_sub(1));
    }
}
