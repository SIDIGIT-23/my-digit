//! My Digit — Aplikasi Bendahara Keuangan Lembaga.
//! made by SIDIGIT-23 Proyek IT SMP Negeri 23 Dumai

// Sembunyikan jendela konsol di Windows untuk build rilis.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod branding;
mod db;
mod format;
mod laporan;
mod model;
mod pdf;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(branding::APP_NAME)
        .with_app_id("my-digit")
        .with_icon(branding::window_icon())
        .with_inner_size([1180.0, 760.0])
        .with_min_inner_size([860.0, 560.0]);
    // macOS: judul jendela transparan agar sidebar menyatu dengan tombol jendela,
    // seperti aplikasi bawaan Apple.
    if cfg!(target_os = "macos") {
        viewport = viewport
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        branding::APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(ui::MyDigitApp::new(cc)))),
    )
}
