//! My Digit — Aplikasi Bendahara Keuangan Lembaga (desktop).
//! made by SIDIGIT-23 Proyek IT SMP Negeri 23 Dumai

// Sembunyikan jendela konsol di Windows untuk build rilis.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    my_digit::jalankan()
}
