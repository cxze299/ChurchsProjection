use crate::{
    app::App,
    media::extension,
    output::{worker, Content, Output},
    widgets::*,
};
use cp_core::{AppSettings, ResourceDirectory};
use egui::{vec2, Context, Ui};
use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
};

pub fn is_picture(path: &Path) -> bool {
    matches!(extension(path).as_str(), "png" | "jpg" | "jpeg")
}
pub fn scan_pictures(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut pending = roots.to_vec();
    let mut seen = HashSet::new();
    let mut files = BTreeSet::new();
    while let Some(dir) = pending.pop() {
        let Ok(key) = dir.canonicalize() else {
            continue;
        };
        if !seen.insert(key) {
            continue;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() && is_picture(&path) {
                files.insert(path);
            }
        }
    }
    files.into_iter().collect()
}
#[derive(Default)]
pub struct Pictures {
    folder: Option<PathBuf>,
    pub selected: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub loading: Option<Receiver<Vec<PathBuf>>>,
}
impl Pictures {
    pub fn refresh(&mut self, ctx: &Context, settings: &AppSettings) {
        self.folder=None;
        let roots = settings
            .resource_directories(ResourceDirectory::Pictures)
            .into_iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        self.loading = Some(worker(ctx, move || scan_pictures(&roots)));
    }
    pub fn poll(&mut self) {
        if let Some(files) = self.loading.as_ref().and_then(|r| r.try_recv().ok()) {
            self.loading = None;
            if !self.selected.as_ref().is_some_and(|p| files.contains(p)) {
                self.selected = files.first().cloned();
            }
            self.files = files;
        }
    }
}
impl App {
    pub fn go_picture(&mut self) {
        if let Some(path) = self.pictures.selected.clone() {
            self.cancel_pending_live();
            self.output.publish(Content::Picture(path));
        }
    }
    pub fn step_picture(&mut self, delta: isize, projector: bool) {
        let current = if projector {
            match &self.output.content {
                Content::Picture(path) => Some(path),
                _ => None,
            }
        } else {
            self.pictures.selected.as_ref()
        };
        let Some(index) =
            current.and_then(|path| self.pictures.files.iter().position(|p| p == path))
        else {
            return;
        };
        let live = matches!(&self.output.content, Content::Picture(path) if Some(path)==current);
        let next = index
            .saturating_add_signed(delta)
            .min(self.pictures.files.len() - 1);
        self.pictures.selected = Some(self.pictures.files[next].clone());
        if projector || self.settings.selection_goes_live {
            self.go_picture();
        } else if live {
            self.output.content = Content::Picture(self.pictures.files[next].clone());
        }
    }
    pub fn pictures_tab(&mut self, ui: &mut Ui) {
        let p = self.palette;
        card_frame(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                if primary(ui, p, "▣ 选择文件夹").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.pictures.folder=Some(folder.clone());
                        self.pictures.loading=Some(worker(ui.ctx(),move||scan_pictures(&[folder])));
                    }
                }
                if self.pictures.folder.is_some() && filter_chip(ui,p,false,"所有目录").clicked(){self.pictures.refresh(ui.ctx(),&self.settings);}
                let path=self.pictures.folder.as_ref().and_then(|p|p.to_str()).unwrap_or_else(||self.settings.primary_directory(ResourceDirectory::Pictures));
                if !path.is_empty() {
                    ui.add(egui::Label::new(path).truncate())
                        .on_hover_text(path);
                }
            });
            ui.horizontal_wrapped(|ui| {
                let i = self
                    .pictures
                    .selected
                    .as_ref()
                    .and_then(|path| self.pictures.files.iter().position(|p| p == path));
                ui.add_enabled_ui(i.is_some_and(|i| i > 0), |ui| {
                    if filter_chip(ui, p, false, "◀ 上一张").clicked() {
                        self.step_picture(-1, false);
                    }
                });
                ui.small(format!(
                    "{} / {}",
                    i.map_or(0, |i| i + 1),
                    self.pictures.files.len()
                ));
                ui.add_enabled_ui(i.is_some_and(|i| i + 1 < self.pictures.files.len()), |ui| {
                    if filter_chip(ui, p, false, "下一张 ▶").clicked() {
                        self.step_picture(1, false);
                    }
                });
                ui.add_enabled_ui(self.pictures.selected.is_some(), |ui| {
                    if schedule_button(ui, p).clicked() {
                        self.add_schedule();
                    }
                    if primary(ui, p, "▶ 立即上屏").clicked() {
                        self.go_picture();
                    }
                });
            });
        });
        card_frame(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(ui.available_height());
            if self.pictures.files.is_empty() {
                section_header_label(ui, p, "图片");
                ui.weak("选择包含 PNG 或 JPEG 图片的文件夹。");
                return;
            }
            thumbnail_grid(
                ui,
                egui::Id::new("pictures_grid"),
                self.pictures.files.len(),
                |ui, i, w| {
                    let path = &self.pictures.files[i];
                    let canvas = self.renderer.canvas(
                        ui,
                        vec2(w, w * 9. / 16.),
                        &mut Output {
                            content: Content::Picture(path.clone()),
                            ..Default::default()
                        },
                        &self.settings,
                        &self.fonts,
                    );
                    let r = thumbnail_caption(
                        ui,
                        p,
                        canvas,
                        &path.file_stem().unwrap_or_default().to_string_lossy(),
                        Some(path) == self.pictures.selected.as_ref(),
                    )
                    .on_hover_text(path.display().to_string());
                    if r.clicked() || r.double_clicked() {
                        self.pictures.selected = Some(path.clone());
                        if r.double_clicked() || self.settings.selection_goes_live {
                            self.go_picture();
                        }
                    }
                },
            );
        });
    }
}
