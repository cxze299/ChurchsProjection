#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod appearance;
mod bible;
mod bible_ui;
mod keyboard;
mod library_ui;
mod media;
mod output;
mod screens;
mod settings_ui;
mod shell;
#[cfg(test)]
mod tests;

fn main() -> eframe::Result<()> {
    let screens = screens::screens();
    let mut viewport = screens::main_window(&screens[0]);
    if let Ok(image) = image::open(r"D:\churchpre\project\assets\branding\app-icon.png") {
        let icon = image.thumbnail(256, 256).into_rgba8();
        viewport = viewport.with_icon(egui::IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_raw(),
        });
    }
    let options = eframe::NativeOptions {
        viewport,
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        "ChurchPresenter",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(&cc.egui_ctx, screens)))),
    )
}
