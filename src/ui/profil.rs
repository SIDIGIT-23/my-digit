//! Halaman Profil Lembaga.

use std::path::PathBuf;

use eframe::egui::{self, RichText, Vec2};

use super::{MyDigitApp, tema};
use crate::db::folder_data;
use crate::model::ProfilLembaga;
use tema::palet;

#[derive(Default)]
pub struct State {
    /// Salinan profil yang sedang diedit.
    pub draf: Option<ProfilLembaga>,
    /// Berkas cadangan yang menunggu konfirmasi pemulihan (halaman Pengaturan).
    pub pulihkan: Option<PathBuf>,
}

pub fn tampil(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let p = palet(ui);
    let mut d = app
        .ui_profil
        .draf
        .take()
        .unwrap_or_else(|| app.profil.clone());
    let berubah = d != app.profil;

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            tema::judul(
                ui,
                "Profil Lembaga",
                "Dipakai untuk kop, tanda tangan laporan, dan kwitansi.",
            )
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(berubah, egui::Button::new(RichText::new("Simpan").strong()))
                .clicked()
            {
                let simpan = d.clone();
                app.ubah(&ctx, "Profil lembaga tersimpan.", |db| {
                    db.simpan_profil(&simpan)
                });
            }
            if berubah && tema::tombol(ui, "Batalkan").clicked() {
                d = app.profil.clone();
            }
        });
    });

    tema::kartu(ui, |ui| {
        tema::subjudul(ui, "Identitas");
        ui.add_space(4.0);
        tema::baris_form(ui, "Nama lembaga", |ui| {
            tema::isian(ui, &mut d.nama, "mis. SMP Negeri 23 Dumai")
        });
        tema::baris_form(ui, "Alamat", |ui| {
            tema::isian(ui, &mut d.alamat, "Jalan, kelurahan, kecamatan")
        });
        tema::baris_form(ui, "Kota / kabupaten", |ui| {
            tema::isian(ui, &mut d.kota, "mis. Dumai")
        });
        tema::baris_form(ui, "Telepon", |ui| tema::isian(ui, &mut d.telepon, ""));
        tema::baris_form(ui, "Email", |ui| tema::isian(ui, &mut d.email, ""));
    });
    ui.add_space(12.0);

    tema::kartu(ui, |ui| {
        tema::subjudul(ui, "Penanggung Jawab");
        ui.add_space(4.0);
        tema::baris_form(ui, "Jabatan pimpinan", |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut d.jabatan_pimpinan).desired_width(220.0));
                for j in ["Kepala Sekolah", "Ketua Yayasan", "Ketua"] {
                    if ui.small_button(j).clicked() {
                        d.jabatan_pimpinan = j.into();
                    }
                }
            });
        });
        tema::baris_form(ui, "Nama pimpinan", |ui| {
            tema::isian(ui, &mut d.nama_pimpinan, "")
        });
        tema::baris_form(ui, "NIP pimpinan", |ui| {
            tema::isian(ui, &mut d.nip_pimpinan, "kosongkan bila tidak ada")
        });
        tema::baris_form(ui, "Nama bendahara", |ui| {
            tema::isian(ui, &mut d.nama_bendahara, "")
        });
        tema::baris_form(ui, "NIP bendahara", |ui| {
            tema::isian(ui, &mut d.nip_bendahara, "kosongkan bila tidak ada")
        });
    });
    ui.add_space(12.0);

    tema::kartu(ui, |ui| {
        tema::subjudul(ui, "Logo Lembaga");
        tema::label_redup(
            ui,
            "Tampil di kop laporan. Terpisah dari logo aplikasi My Digit.",
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(84.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 12, p.latar);
            match &app.logo_lembaga {
                Some(tex) if d.logo.is_some() => {
                    egui::Image::new(tex)
                        .fit_to_exact_size(Vec2::splat(76.0))
                        .paint_at(ui, rect.shrink(4.0));
                }
                _ => {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🏫",
                        egui::FontId::proportional(30.0),
                        p.teks2,
                    );
                }
            }
            ui.vertical(|ui| {
                if tema::tombol(ui, "Pilih Gambar…").clicked() {
                    pilih_logo(app, &mut d, &ctx);
                }
                if d.logo.is_some() && tema::tombol_bahaya(ui, "Hapus Logo").clicked() {
                    d.logo = None;
                }
            });
        });
    });

    app.ui_profil.draf = Some(d);
}

/// Salin gambar pilihan ke folder data, lalu langsung simpan ke profil.
fn pilih_logo(app: &mut MyDigitApp, d: &mut ProfilLembaga, ctx: &egui::Context) {
    let Some(src) = rfd::FileDialog::new()
        .add_filter("Gambar", &["png", "jpg", "jpeg"])
        .pick_file()
    else {
        return;
    };
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let nama = format!("logo-lembaga.{ext}");
    if image::open(&src).is_err() {
        app.galat("Berkas gambar tidak dapat dibaca.");
        return;
    }
    if let Err(e) = std::fs::copy(&src, folder_data().join(&nama)) {
        app.galat(format!("Gagal menyalin logo: {e}"));
        return;
    }
    d.logo = Some(nama);
    let mut simpan = app.profil.clone();
    simpan.logo = d.logo.clone();
    let draf = d.clone();
    app.ubah(ctx, "Logo lembaga diperbarui.", |db| {
        db.simpan_profil(&simpan)
    });
    // Pertahankan isian lain yang belum disimpan.
    *d = draf;
}
