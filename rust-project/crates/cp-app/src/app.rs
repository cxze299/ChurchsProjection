use crate::{
    appearance::{self, Fonts},
    bible::{load_modules, Browser},
    media::{Document, Library, Page},
    output::{worker, Content, Output, Renderer},
    screens::Screen,
};
use cp_core::{
    AppSettings, ResourceDirectory, ScheduleEntry, SelectedVerse, SettingsManager, ShortcutAction,
};
use egui::{Context, ViewportId};
use std::{
    path::{Path, PathBuf},
    sync::{mpsc::Receiver, Arc},
    time::Instant,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Bible,
    Library,
    Songs,
    Pictures,
    Announcements,
    Settings,
}
pub struct Scheduled {
    pub entry: Option<ScheduleEntry>,
    pub content: Content,
}
type BibleLoad = (Vec<cp_bible::Bible>, Vec<String>);
type DocumentLoad = Receiver<Result<Arc<Document>, String>>;
pub struct App {
    pub manager: SettingsManager,
    pub settings: AppSettings,
    pub fonts: Fonts,
    pub palette: appearance::Palette,
    pub pictures: crate::pictures_ui::Pictures,
    pub announcement: crate::announcements_ui::Announcement,
    pub song_results: Vec<PathBuf>,
    pub song_query: String,
    pub song_book: String,
    pub song_page: Option<Page>,
    pub deck_page: Option<Page>,
    pub browse_results: bool,
    pub history: Vec<Arc<Vec<SelectedVerse>>>,
    pub history_open: bool,
    pub search_mode: crate::bible_ui::SearchMode,
    pub scoped: bool,
    pub hits: Option<Vec<cp_bible::BibleSearch>>,
    pub bible_widths: [f32; 2],
    pub toast: Option<(String, Instant)>,
    pub schedule_loads: std::collections::VecDeque<Receiver<Vec<Result<Content, String>>>>,
    pub tab: Tab,
    pub bible: Browser,
    pub library: Library,
    pub query: String,
    pub quick: String,
    pub deadline: Option<Instant>,
    pub continuous: bool,
    pub style_open: bool,
    pub record: Option<ShortcutAction>,
    pub directory_text: String,
    pub directory_section: ResourceDirectory,
    pub output: Output,
    pub renderer: Renderer,
    pub screens: Vec<Screen>,
    pub target: usize,
    pub fullscreen: bool,
    pub placed: Option<usize>,
    pub left: bool,
    pub right: bool,
    pub schedule: Vec<Scheduled>,
    pub serial: u64,
    pub staged: Option<Page>,
    pub added: Vec<PathBuf>,
    pub results: Vec<PathBuf>,
    pub reset_results: bool,
    pub focus_search: bool,
    pub message: String,
    pub bible_load: Option<Receiver<BibleLoad>>,
    pub library_load: Option<Receiver<Library>>,
    pub opening: Option<(DocumentLoad, bool)>,
}
impl App {
    pub fn new(ctx: &Context, screens: Vec<Screen>) -> Self {
        Self::new_with_manager(ctx, screens, SettingsManager::new())
    }

    pub fn new_with_manager(ctx: &Context, screens: Vec<Screen>, manager: SettingsManager) -> Self {
        let mut settings = manager.load();
        let added = settings
            .extra
            .get("nativePresentationFiles")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        if let Some(profile) = settings.projection_settings.output_profiles.first() {
            let b = &mut settings.bible_settings;
            b.continuous_line_spacing_percent = profile
                .bible_settings
                .continuous_line_spacing_percent
                .or(b.continuous_line_spacing_percent);
            b.continuous_passage_spacing = profile
                .bible_settings
                .continuous_passage_spacing
                .or(b.continuous_passage_spacing);
        }
        if let Some(p) = settings.extra.get("presentationSettings") {
            if let Some(f) = p["quickPlayFormat"].as_str() {
                settings.preferred_presentation_format = f.to_lowercase();
            }
            if let Some(b) = p["quickPlayBook"].as_str() {
                settings.preferred_hymn_book = b.to_owned();
            }
        }
        let prefs = &settings.projection_settings;
        let target = prefs
            .quick_play_display
            .and_then(|i| usize::try_from(i).ok())
            .filter(|&i| i < screens.len())
            .unwrap_or(usize::from(screens.len() > 1));
        let fullscreen = !prefs.windowed_projection
            && prefs
                .quick_play_display
                .is_some_and(|i| usize::try_from(i).is_ok_and(|n| n < screens.len()));
        let palette = appearance::palette(
            settings
                .extra
                .get("theme")
                .and_then(|v| v.as_str())
                .unwrap_or("Dark"),
        );
        let fonts = Fonts::install(ctx);
        appearance::theme(ctx, palette);
        let mut app = Self {
            manager,
            settings,
            fonts,
            palette,
            pictures: Default::default(),
            announcement: Default::default(),
            song_results: Vec::new(),
            song_query: String::new(),
            song_book: String::new(),
            song_page: None,
            deck_page: None,
            browse_results: false,
            history: Vec::new(),
            history_open: true,
            search_mode: Default::default(),
            scoped: false,
            hits: None,
            bible_widths: [0.19, 0.105],
            toast: None,
            schedule_loads: Default::default(),
            tab: Tab::Bible,
            bible: Default::default(),
            library: Default::default(),
            query: String::new(),
            quick: String::new(),
            deadline: None,
            continuous: false,
            style_open: false,
            record: None,
            directory_text: String::new(),
            directory_section: ResourceDirectory::Bible,
            output: Default::default(),
            renderer: Default::default(),
            screens,
            target,
            fullscreen,
            placed: None,
            left: true,
            right: true,
            schedule: Vec::new(),
            serial: 0,
            staged: None,
            added,
            results: vec![],
            reset_results: true,
            focus_search: false,
            message: String::new(),
            bible_load: None,
            library_load: None,
            opening: None,
        };
        app.refresh(ctx, ResourceDirectory::Bible);
        app.refresh(ctx, ResourceDirectory::Presentations);
        app.refresh(ctx, ResourceDirectory::Pictures);
        app
    }
    pub fn save(&mut self) {
        if let Err(e) = self.manager.save(&self.settings) {
            self.message = format!("无法保存设置 · {e}");
        }
    }
    pub fn refresh(&mut self, ctx: &Context, section: ResourceDirectory) {
        let settings = self.settings.clone();
        match section {
            ResourceDirectory::Bible => {
                self.bible_load = Some(worker(ctx, move || load_modules(&settings)))
            }
            ResourceDirectory::Songs | ResourceDirectory::Presentations => {
                let added = self.added.clone();
                self.library_load = Some(worker(ctx, move || Library::load(&settings, &added)));
            }
            ResourceDirectory::Pictures => self.pictures.refresh(ctx, &settings),
            _ => {}
        }
    }
    pub fn sync_filters(&mut self) {
        let p = self
            .settings
            .extra
            .entry("presentationSettings")
            .or_insert_with(|| serde_json::json!({}));
        if p.is_object() {
            p["quickPlayFormat"] = self.settings.preferred_presentation_format.clone().into();
            p["quickPlayBook"] = self.settings.preferred_hymn_book.clone().into();
        }
        self.save();
        self.search();
    }
    pub fn search(&mut self) {
        self.results = self.library.find(
            &self.query,
            &self.settings.preferred_hymn_book,
            &self.settings.preferred_presentation_format,
        );
        self.reset_results = true;
    }
    pub fn poll(&mut self) {
        self.pictures.poll();
        while let Some(items) = self
            .schedule_loads
            .front()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.schedule_loads.pop_front();
            for result in items {
                match result {
                    Ok(content) => self.enqueue(content),
                    Err(e) => self.message = e,
                }
            }
        }
        if let Some((modules, errors)) = self.bible_load.as_ref().and_then(|r| r.try_recv().ok()) {
            self.bible_load = None;
            self.hits = None;
            let (book, chapter) = (self.bible.book, self.bible.chapter);
            self.bible = Browser::new(modules, &self.settings);
            self.bible.set_location(book, chapter);
            if !errors.is_empty() {
                self.message = errors.join(" · ");
            }
        }
        if let Some(library) = self.library_load.as_ref().and_then(|r| r.try_recv().ok()) {
            self.library_load = None;
            self.library = library;
            self.filter_songs();
            if !self.settings.preferred_hymn_book.is_empty()
                && !self
                    .library
                    .books
                    .contains(&self.settings.preferred_hymn_book)
            {
                self.settings.preferred_hymn_book.clear();
            }
            self.search();
        }
        if let Some(doc) = self.opening.as_ref().and_then(|(r, _)| r.try_recv().ok()) {
            let (_, live) = self.opening.take().unwrap();
            match doc {
                Ok(document) => {
                    let page = Page { document, index: 0 };
                    self.staged = Some(page.clone());
                    if live {
                        self.output.publish(Content::Media(page));
                    }
                }
                Err(e) => self.message = format!("无法打开资源 · {e}"),
            }
        }
    }
    pub fn open(&mut self, ctx: &Context, path: &Path, live: bool) {
        self.message.clear();
        if self
            .staged
            .as_ref()
            .is_some_and(|p| p.document.path() == path)
        {
            self.opening = None;
            if live {
                self.go_media();
            }
            return;
        }
        self.staged = None;
        let book = cp_library::quick_play::book(path, &self.library.roots);
        let path = path.to_owned();
        self.opening = Some((worker(ctx, move || Document::open(&path, &book)), live));
    }
    pub fn go_media(&mut self) {
        if let Some(page) = &self.staged {
            self.output.publish(Content::Media(page.clone()));
        } else if let Some((_, live)) = &mut self.opening {
            *live = true;
        }
    }
    fn publish_bible(&mut self, verses: Vec<SelectedVerse>) {
        if !verses.is_empty() {
            self.cancel_pending_live();
            let verses = Arc::new(verses);
            self.remember_passage(verses.clone());
            self.output.publish(Content::Scripture {
                verses,
                continuous: self.continuous,
            });
        }
    }
    fn remember_passage(&mut self, verses: Arc<Vec<SelectedVerse>>) {
        self.history.retain(|old| old != &verses);
        self.history.insert(0, verses);
        self.history.truncate(64);
    }
    pub fn cancel_pending_live(&mut self) {
        if let Some((_, live)) = &mut self.opening {
            *live = false;
        }
        self.deadline = None;
    }
    pub fn go_bible(&mut self) {
        let verses = if self.continuous {
            self.bible.append_selected()
        } else {
            self.bible.selected()
        };
        self.publish_bible(verses);
    }
    pub fn apply_quick(&mut self, explicit: bool) {
        self.deadline = None;
        if let Some(verses) = self.bible.apply_quick(&self.quick, self.continuous) {
            self.message.clear();
            if self.continuous || explicit || self.settings.selection_goes_live {
                self.publish_bible(verses);
            }
        } else if explicit {
            self.message = "请输入有效的书卷、章和节数".into();
        }
    }
    pub fn clear_passages(&mut self) {
        self.bible.clear();
        self.quick.clear();
        self.hits = None;
        self.deadline = None;
        if matches!(self.output.content, Content::Scripture { .. }) {
            self.output.content = Content::Empty;
        }
    }
    pub fn add_schedule(&mut self) {
        let content = match self.tab {
            Tab::Bible => {
                let verses = self.bible.selected();
                if verses.is_empty() {
                    return;
                }
                Content::Scripture {
                    verses: Arc::new(verses),
                    continuous: false,
                }
            }
            Tab::Library | Tab::Songs => {
                let Some(page) = self.staged.clone() else {
                    return;
                };
                Content::Media(page)
            }
            Tab::Pictures => {
                let Some(path) = self.pictures.selected.clone() else {
                    return;
                };
                Content::Picture(path)
            }
            _ => return,
        };
        self.enqueue(content);
    }
    pub fn enqueue(&mut self, content: Content) {
        self.serial += 1;
        let id = self.serial.to_string();
        let entry = match &content {
            Content::Scripture { verses, .. } => verses.first().map(|v| ScheduleEntry::Bible {
                id,
                book_id: v.book_id,
                book_name: v.book_name.clone(),
                chapter: v.chapter,
                verse_number: v.verse_number,
                verse_range: v.verse_range.clone(),
                verse_text: v.verse_text.clone(),
            }),
            Content::Media(page) => Some(page.document.entry(
                id,
                cp_library::quick_play::book(page.document.path(), &self.library.roots),
            )),
            _ => None,
        };
        self.schedule.push(Scheduled { entry, content });
        self.message = format!("已加入程序单 · 共 {} 项", self.schedule.len());
    }
    pub fn play_schedule(&mut self, index: usize) {
        let content = self.schedule[index].content.clone();
        self.cancel_pending_live();
        self.opening = None;
        match &content {
            Content::Media(p) => {
                self.change_tab(if matches!(&*p.document, Document::Song { .. }) {
                    Tab::Songs
                } else {
                    Tab::Library
                });
                self.staged = Some(p.clone());
            }
            Content::Scripture { verses, .. } => {
                self.change_tab(Tab::Bible);
                self.remember_passage(verses.clone());
                self.bible.restore(verses);
            }
            Content::Picture(path) => {
                self.change_tab(Tab::Pictures);
                self.pictures.selected = Some(path.clone());
            }
            Content::Announcement(_) | Content::Empty => {}
        }
        self.output.publish(content);
    }
    pub fn step_page(&mut self, delta: isize) {
        if let Some(p) = &mut self.staged {
            p.step(delta);
            let on_air = matches!(&self.output.content,Content::Media(live) if Arc::ptr_eq(&live.document,&p.document));
            if on_air {
                self.output.content = Content::Media(p.clone());
            } else if self.settings.selection_goes_live {
                self.output.publish(Content::Media(p.clone()));
            }
        }
    }
    pub fn navigate(&mut self, delta: isize, projector: bool) {
        if (!projector && self.tab == Tab::Pictures)
            || (projector && matches!(self.output.content, Content::Picture(_)))
        {
            self.step_picture(delta, projector);
            return;
        }
        if !projector && matches!(self.tab, Tab::Settings | Tab::Announcements) {
            return;
        }
        if !projector && matches!(self.tab, Tab::Library | Tab::Songs) {
            self.step_page(delta);
            return;
        }
        if projector {
            if let Content::Media(p) = &mut self.output.content {
                p.step(delta);
                for staged in [&mut self.staged, &mut self.song_page, &mut self.deck_page].into_iter().flatten(){
                    if Arc::ptr_eq(&p.document,&staged.document){staged.index=p.index;}
                }
                return;
            }
            if let Content::Scripture { verses, .. } = &self.output.content {
                self.bible.restore(verses);
            } else {
                return;
            }
        }
        if self.bible.step(delta) && (projector || self.settings.selection_goes_live) {
            self.go_bible();
        }
    }
    pub fn target_changed(&mut self) {
        self.settings.projection_settings.windowed_projection = !self.fullscreen;
        self.settings.projection_settings.quick_play_display =
            self.fullscreen.then_some(self.target as i32);
        self.save();
    }
    pub fn change_tab(&mut self, tab: Tab) {
        if self.tab != tab {
            match self.tab {
                Tab::Library => self.deck_page = self.staged.take(),
                Tab::Songs => self.song_page = self.staged.take(),
                _ => {}
            }
            self.opening = None;
            self.staged = match tab {
                Tab::Library => self.deck_page.take(),
                Tab::Songs => self.song_page.take(),
                _ => None,
            };
            self.tab = tab;
            self.focus_search = self
                .settings
                .keyboard_shortcut_settings
                .focus_search_on_tab_open;
            self.deadline = None;
            self.record = None;
        }
    }
    pub fn projector_id() -> ViewportId {
        ViewportId::from_hash_of("live_projector")
    }
}
