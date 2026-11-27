use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SelectedVerse {
    #[serde(default)]
    pub translation_file_name: String,
    #[serde(default)]
    pub bible_abbreviation: String,
    #[serde(default)]
    pub bible_name: String,
    #[serde(default)]
    pub book_name: String,
    #[serde(default)]
    pub chapter: i32,
    #[serde(default)]
    pub verse_number: i32,
    #[serde(default)]
    pub verse_text: String,
    #[serde(default)]
    pub verse_range: String,
    #[serde(default)]
    pub book_id: i32,
    #[serde(default)]
    pub continuous_passages: Vec<Vec<SelectedVerse>>,
}

impl SelectedVerse {
    pub fn display_reference(&self) -> String {
        let range = if self.verse_range.trim().is_empty() {
            self.verse_number.to_string()
        } else {
            self.verse_range.clone()
        };
        if self.book_name.is_empty() {
            format!("{}:{}", self.chapter, range)
        } else {
            format!("{} {}:{}", self.book_name, self.chapter, range)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongTranslation {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub lyrics: Vec<String>,
}

impl SongTranslation {
    pub fn is_empty(&self) -> bool {
        self.title.trim().is_empty() && self.lyrics.iter().all(|l| l.trim().is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongItem {
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub songbook: String,
    #[serde(default)]
    pub tune: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub composer: String,
    #[serde(default)]
    pub lyrics: Vec<String>,
    #[serde(default)]
    pub secondary_title: String,
    #[serde(default)]
    pub secondary_lyrics: Vec<String>,
    #[serde(default)]
    pub translations: Vec<SongTranslation>,
    #[serde(default)]
    pub source_file: String,
    #[serde(default)]
    pub ccli_number: String,
}

impl SongItem {
    pub fn song_id(&self) -> String {
        if !self.number.trim().is_empty() {
            format!("{}::{}", self.songbook, self.number)
        } else {
            format!("{}::{}", self.songbook, self.title)
        }
    }

    pub fn display_title(&self) -> String {
        if !self.number.trim().is_empty() {
            format!("{} - {}", self.number, self.title)
        } else {
            self.title.clone()
        }
    }

    /// Splits primary lyrics into presentation slides (grouped by blank lines or `[Section]` headers).
    pub fn lyric_slides(&self) -> Vec<LyricSlide> {
        let mut slides = Vec::new();
        let mut current_section = String::new();
        let mut current_lines: Vec<String> = Vec::new();

        for raw_line in &self.lyrics {
            let trimmed = raw_line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() > 2 {
                if !current_lines.is_empty() {
                    slides.push(LyricSlide {
                        section: current_section.clone(),
                        lines: std::mem::take(&mut current_lines),
                    });
                }
                current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
            } else if trimmed.is_empty() {
                if !current_lines.is_empty() {
                    slides.push(LyricSlide {
                        section: current_section.clone(),
                        lines: std::mem::take(&mut current_lines),
                    });
                }
            } else {
                current_lines.push(raw_line.clone());
            }
        }
        if !current_lines.is_empty() {
            slides.push(LyricSlide {
                section: current_section,
                lines: current_lines,
            });
        }
        slides
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LyricSlide {
    pub section: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleEntry {
    Bible {
        id: String,
        book_id: i32,
        book_name: String,
        chapter: i32,
        verse_number: i32,
        verse_range: String,
        verse_text: String,
    },
    Song {
        id: String,
        number: String,
        title: String,
        songbook: String,
        source_file: String,
    },
    Presentation {
        id: String,
        file_path: String,
        file_name: String,
        book: String,
        slide_count: usize,
    },
}

impl ScheduleEntry {
    pub fn display_label(&self) -> String {
        match self {
            ScheduleEntry::Bible {
                book_name,
                chapter,
                verse_number,
                verse_range,
                ..
            } => {
                if verse_range.is_empty() {
                    format!("📖 {} {}:{}", book_name, chapter, verse_number)
                } else {
                    format!("📖 {} {}:{}", book_name, chapter, verse_range)
                }
            }
            ScheduleEntry::Song {
                number,
                title,
                songbook,
                ..
            } => {
                let prefix = if number.is_empty() {
                    title.clone()
                } else {
                    format!("{} - {}", number, title)
                };
                if songbook.is_empty() {
                    format!("🎵 {}", prefix)
                } else {
                    format!("🎵 [{}] {}", songbook, prefix)
                }
            }
            ScheduleEntry::Presentation {
                file_name,
                book,
                slide_count,
                ..
            } => {
                if book.is_empty() {
                    format!("📊 {} ({}页)", file_name, slide_count)
                } else {
                    format!("📊 [{}] {} ({}页)", book, file_name, slide_count)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LiveProjectionContent {
    #[default]
    None,
    Bible {
        verses: Vec<SelectedVerse>,
        is_continuous: bool,
    },
    Song {
        song_title: String,
        section: String,
        lines: Vec<String>,
        slide_index: usize,
        total_slides: usize,
    },
    DeckSlide {
        deck_title: String,
        slide_index: usize,
        total_slides: usize,
        slide_title: String,
        text_blocks: Vec<String>,
        image_path: Option<String>,
        image_bytes: Option<Vec<u8>>,
    },
}
