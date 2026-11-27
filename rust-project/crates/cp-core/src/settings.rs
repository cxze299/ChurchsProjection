use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceDirectory {
    Bible,
    Songs,
    Pictures,
    LowerThird,
    Presentations,
    Media,
    Schedule,
    Calendar,
    Signage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibleTranslationSettings {
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub custom_name: String,
    #[serde(default)]
    pub custom_abbreviation: String,
    #[serde(default)]
    pub book_name_overrides: HashMap<String, String>,
    #[serde(default = "default_white")]
    pub text_color: String,
    #[serde(default = "default_font")]
    pub text_font_type: String,
    #[serde(default = "default_font_size")]
    pub text_font_size: i32,
    #[serde(default)]
    pub text_bold: bool,
    #[serde(default)]
    pub text_italic: bool,
    #[serde(default)]
    pub text_underline: bool,
    #[serde(default)]
    pub text_shadow: bool,
    #[serde(default = "default_white")]
    pub reference_color: String,
    #[serde(default = "default_font")]
    pub reference_font_type: String,
    #[serde(default = "default_ref_font_size")]
    pub reference_font_size: i32,
    #[serde(default)]
    pub reference_bold: bool,
    #[serde(default)]
    pub show_abbreviation: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_white() -> String {
    "#FFFFFF".to_string()
}
fn default_font() -> String {
    "Microsoft YaHei".to_string()
}
fn default_font_size() -> i32 {
    46
}
fn default_ref_font_size() -> i32 {
    34
}

impl Default for BibleTranslationSettings {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            custom_name: String::new(),
            custom_abbreviation: String::new(),
            book_name_overrides: HashMap::new(),
            text_color: default_white(),
            text_font_type: default_font(),
            text_font_size: default_font_size(),
            text_bold: false,
            text_italic: false,
            text_underline: false,
            text_shadow: true,
            reference_color: "#FFD54F".to_string(),
            reference_font_type: default_font(),
            reference_font_size: default_ref_font_size(),
            reference_bold: true,
            show_abbreviation: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibleSettings {
    #[serde(default)]
    pub storage_directory: String,
    #[serde(default)]
    pub bible_files: Vec<String>,
    #[serde(default)]
    pub primary_bible: String,
    #[serde(default)]
    pub secondary_bible: String,
    #[serde(default)]
    pub translations: Vec<BibleTranslationSettings>,
    #[serde(default)]
    pub continuous_line_spacing_percent: Option<i32>,
    #[serde(default)]
    pub continuous_passage_spacing: Option<i32>,
    #[serde(default = "default_book_col_width")]
    pub bible_col_width_book: i32,
    #[serde(default = "default_chapter_col_width")]
    pub bible_col_width_chapter: i32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_book_col_width() -> i32 {
    200
}
fn default_chapter_col_width() -> i32 {
    120
}

impl Default for BibleSettings {
    fn default() -> Self {
        Self {
            storage_directory: String::new(),
            bible_files: Vec::new(),
            primary_bible: String::new(),
            secondary_bible: String::new(),
            translations: Vec::new(),
            continuous_line_spacing_percent: Some(135),
            continuous_passage_spacing: Some(24),
            bible_col_width_book: 200,
            bible_col_width_chapter: 120,
            extra: Map::new(),
        }
    }
}

impl BibleSettings {
    pub fn effective_translations(&self) -> Vec<BibleTranslationSettings> {
        if !self.translations.is_empty() {
            return self.translations.clone();
        }
        let mut list = Vec::new();
        if !self.primary_bible.trim().is_empty() {
            list.push(BibleTranslationSettings {
                file_name: self.primary_bible.clone(),
                ..Default::default()
            });
        }
        if !self.secondary_bible.trim().is_empty() {
            list.push(BibleTranslationSettings {
                file_name: self.secondary_bible.clone(),
                ..Default::default()
            });
        }
        list
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongSettings {
    #[serde(default)]
    pub storage_directory: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PictureSettings {
    #[serde(default)]
    pub storage_directory: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StreamingSettings {
    #[serde(default)]
    pub lower_third_folder: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputProfile {
    #[serde(default = "default_profile_id")]
    pub id: String,
    #[serde(default = "default_profile_name")]
    pub name: String,
    #[serde(default)]
    pub bible_settings: BibleSettings,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_profile_id() -> String {
    "default".to_string()
}
fn default_profile_name() -> String {
    "Default".to_string()
}

impl Default for OutputProfile {
    fn default() -> Self {
        Self {
            id: default_profile_id(),
            name: default_profile_name(),
            bible_settings: BibleSettings::default(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionSettings {
    #[serde(default = "default_true")]
    pub start_outputs_hidden: bool,
    #[serde(default)]
    pub windowed_projection: bool,
    #[serde(default)]
    pub quick_play_display: Option<i32>,
    #[serde(default)]
    pub output_profiles: Vec<OutputProfile>,
    #[serde(default)]
    pub screen_names: HashMap<String, String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_true() -> bool {
    true
}

impl Default for ProjectionSettings {
    fn default() -> Self {
        Self {
            start_outputs_hidden: true,
            windowed_projection: false,
            quick_play_display: None,
            output_profiles: vec![OutputProfile::default()],
            screen_names: HashMap::new(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_settings_version")]
    pub settings_version: i32,
    #[serde(default)]
    pub bible_settings: BibleSettings,
    #[serde(default)]
    pub song_settings: SongSettings,
    #[serde(default)]
    pub picture_settings: PictureSettings,
    #[serde(default)]
    pub streaming_settings: StreamingSettings,
    #[serde(default)]
    pub projection_settings: ProjectionSettings,
    #[serde(default)]
    pub keyboard_shortcut_settings: crate::shortcuts::KeyboardShortcutSettings,
    #[serde(default)]
    pub presentation_storage_directory: String,
    #[serde(default)]
    pub media_storage_directory: String,
    #[serde(default)]
    pub schedule_storage_directory: String,
    #[serde(default)]
    pub calendar_storage_directory: String,
    #[serde(default)]
    pub signage_storage_directory: String,
    #[serde(default)]
    pub additional_resource_directories: HashMap<ResourceDirectory, Vec<String>>,
    #[serde(default)]
    pub selection_goes_live: bool,
    #[serde(default = "default_true")]
    pub schedule_panel_collapsed: bool,
    #[serde(default = "default_true")]
    pub preview_panel_collapsed: bool,
    #[serde(default = "default_panel_width")]
    pub schedule_panel_width_dp: i32,
    #[serde(default = "default_panel_width")]
    pub preview_panel_width_dp: i32,
    #[serde(default = "default_preferred_format")]
    pub preferred_presentation_format: String,
    #[serde(default)]
    pub preferred_hymn_book: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_settings_version() -> i32 {
    27
}
fn default_panel_width() -> i32 {
    280
}
fn default_preferred_format() -> String {
    "pptx".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            settings_version: 27,
            bible_settings: BibleSettings::default(),
            song_settings: SongSettings::default(),
            picture_settings: PictureSettings::default(),
            streaming_settings: StreamingSettings::default(),
            projection_settings: ProjectionSettings::default(),
            keyboard_shortcut_settings: crate::shortcuts::KeyboardShortcutSettings::default(),
            presentation_storage_directory: String::new(),
            media_storage_directory: String::new(),
            schedule_storage_directory: String::new(),
            calendar_storage_directory: String::new(),
            signage_storage_directory: String::new(),
            additional_resource_directories: HashMap::new(),
            selection_goes_live: false,
            schedule_panel_collapsed: true,
            preview_panel_collapsed: true,
            schedule_panel_width_dp: 280,
            preview_panel_width_dp: 280,
            preferred_presentation_format: "pptx".to_string(),
            preferred_hymn_book: String::new(),
            extra: Map::new(),
        }
    }
}

impl AppSettings {
    pub fn primary_directory(&self, section: ResourceDirectory) -> &str {
        match section {
            ResourceDirectory::Bible => &self.bible_settings.storage_directory,
            ResourceDirectory::Songs => &self.song_settings.storage_directory,
            ResourceDirectory::Pictures => &self.picture_settings.storage_directory,
            ResourceDirectory::LowerThird => &self.streaming_settings.lower_third_folder,
            ResourceDirectory::Presentations => &self.presentation_storage_directory,
            ResourceDirectory::Media => &self.media_storage_directory,
            ResourceDirectory::Schedule => &self.schedule_storage_directory,
            ResourceDirectory::Calendar => &self.calendar_storage_directory,
            ResourceDirectory::Signage => &self.signage_storage_directory,
        }
    }

    pub fn resource_directories(&self, section: ResourceDirectory) -> Vec<String> {
        let mut dirs = Vec::new();
        let primary = self.primary_directory(section).trim();
        if !primary.is_empty() {
            dirs.push(primary.to_string());
        }
        if let Some(extras) = self.additional_resource_directories.get(&section) {
            for d in extras {
                let t = d.trim();
                if !t.is_empty() && !dirs.iter().any(|existing| same_path(existing, t)) {
                    dirs.push(t.to_string());
                }
            }
        }
        dirs
    }

    pub fn set_primary_resource_directory(&mut self, section: ResourceDirectory, path: String) {
        let trimmed = path.trim().to_string();
        let prev = self.primary_directory(section).trim().to_string();
        let mut remaining: Vec<String> = Vec::new();
        if !prev.is_empty() && !same_path(&prev, &trimmed) {
            remaining.push(prev);
        }
        if let Some(extras) = self.additional_resource_directories.get(&section) {
            for e in extras {
                let t = e.trim();
                if !t.is_empty()
                    && !same_path(t, &trimmed)
                    && !remaining.iter().any(|r| same_path(r, t))
                {
                    remaining.push(t.to_string());
                }
            }
        }
        match section {
            ResourceDirectory::Bible => self.bible_settings.storage_directory = trimmed,
            ResourceDirectory::Songs => self.song_settings.storage_directory = trimmed,
            ResourceDirectory::Pictures => self.picture_settings.storage_directory = trimmed,
            ResourceDirectory::LowerThird => self.streaming_settings.lower_third_folder = trimmed,
            ResourceDirectory::Presentations => self.presentation_storage_directory = trimmed,
            ResourceDirectory::Media => self.media_storage_directory = trimmed,
            ResourceDirectory::Schedule => self.schedule_storage_directory = trimmed,
            ResourceDirectory::Calendar => self.calendar_storage_directory = trimmed,
            ResourceDirectory::Signage => self.signage_storage_directory = trimmed,
        }
        self.additional_resource_directories
            .insert(section, remaining);
    }

    pub fn add_additional_resource_directory(&mut self, section: ResourceDirectory, path: String) {
        let trimmed = path.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        if self
            .resource_directories(section)
            .iter()
            .any(|existing| same_path(existing, &trimmed))
        {
            return;
        }
        self.additional_resource_directories
            .entry(section)
            .or_default()
            .push(trimmed);
    }

    pub fn remove_additional_resource_directory(&mut self, section: ResourceDirectory, path: &str) {
        if let Some(list) = self.additional_resource_directories.get_mut(&section) {
            list.retain(|item| !same_path(item, path));
        }
    }

    pub fn active_bible_styles(&self) -> Vec<BibleTranslationSettings> {
        let mut styles = self.bible_settings.effective_translations();
        if let Some(profile) = self.projection_settings.output_profiles.first() {
            let prof_styles = profile.bible_settings.effective_translations();
            if !prof_styles.is_empty() {
                for s in &mut styles {
                    if let Some(ps) = prof_styles.iter().find(|p| p.file_name == s.file_name) {
                        s.text_color = ps.text_color.clone();
                        s.text_font_type = ps.text_font_type.clone();
                        s.text_font_size = ps.text_font_size;
                        s.text_bold = ps.text_bold;
                        s.text_italic = ps.text_italic;
                        s.reference_color = ps.reference_color.clone();
                        s.reference_font_type = ps.reference_font_type.clone();
                        s.reference_font_size = ps.reference_font_size;
                        s.reference_bold = ps.reference_bold;
                    }
                }
            }
        }
        styles
    }
}

fn same_path(a: &str, b: &str) -> bool {
    let pa = Path::new(a);
    let pb = Path::new(b);
    pa == pb || a.replace('\\', "/").eq_ignore_ascii_case(&b.replace('\\', "/"))
}

pub struct SettingsManager {
    pub app_data_dir: PathBuf,
    pub settings_file: PathBuf,
}

impl SettingsManager {
    pub fn new() -> Self {
        let app_data_dir = Self::resolve_app_data_dir();
        let _ = fs::create_dir_all(&app_data_dir);
        let settings_file = app_data_dir.join("settings.json");
        Self {
            app_data_dir,
            settings_file,
        }
    }

    pub fn with_dir(app_data_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&app_data_dir);
        let settings_file = app_data_dir.join("settings.json");
        Self {
            app_data_dir,
            settings_file,
        }
    }

    pub fn resolve_app_data_dir() -> PathBuf {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        let primary = PathBuf::from(&home).join(".churchpresenter");
        if fs::create_dir_all(&primary).is_ok() {
            return primary;
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let fallback = PathBuf::from(local_app_data).join("ChurchPresenter");
            if fs::create_dir_all(&fallback).is_ok() {
                return fallback;
            }
        }
        primary
    }

    pub fn default_bibles_dir(&self) -> PathBuf {
        let dir = self.app_data_dir.join("Bibles");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn load(&self) -> AppSettings {
        if let Ok(raw) = fs::read_to_string(&self.settings_file) {
            if let Ok(mut settings) = serde_json::from_str::<AppSettings>(&raw) {
                if settings.bible_settings.storage_directory.trim().is_empty() {
                    settings.bible_settings.storage_directory =
                        self.default_bibles_dir().to_string_lossy().to_string();
                }
                return settings;
            }
        }
        let mut settings = AppSettings::default();
        settings.bible_settings.storage_directory =
            self.default_bibles_dir().to_string_lossy().to_string();
        settings
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let tmp = self.app_data_dir.join("settings.json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &self.settings_file)?;
        Ok(())
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}
