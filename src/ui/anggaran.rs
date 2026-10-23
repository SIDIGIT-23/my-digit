//! Halaman Anggaran: target tahunan per kategori dan realisasinya.

use std::collections::HashMap;

use eframe::egui::{self, RichText};

use super::{MyDigitApp, pilih_tahun, tema};
use crate::format::{hari_ini, parse_rupiah, ribuan, rupiah};
use crate::laporan::{Periode, persen};
use crate::model::*;
use tema::palet;

pub struct State {
    tahun: i16,
    /// Isian anggaran per kategori untuk `tahun`; `None` bila perlu dimuat.
    isian: Option<HashMap<i64, String>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tahun: hari_ini().year(),
            isian: None,
        }
    }
}

pub fn tampil(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let p = palet(ui);
    let sub = format!(
        "Susun anggaran tahunan ({}), lalu pantau realisasinya.",
        app.instansi.istilah_anggaran
    );
    tema::baris_kepala(ui, |ui| {
        ui.vertical(|ui| tema::judul(ui, "Anggaran", &sub));
        tema::kanan(ui, |ui| {
            if tema::tombol_utama(ui, "", "Simpan Anggaran").clicked() {
                simpan(app, &ctx);
            }
            if pilih_tahun(ui, &mut app.ui_anggaran.tahun) {
                app.ui_anggaran.isian = None;
            }
            tema::label_redup(ui, "Tahun anggaran");
        });
    });

    let tahun = app.ui_anggaran.tahun;
    if app.ui_anggaran.isian.is_none() {
        let tersimpan = app.db().daftar_anggaran(tahun).unwrap_or_default();
        let map = app
            .kategori
            .iter()
            .map(|k| {
                let n = tersimpan
                    .iter()
                    .find(|a| a.kategori_id == k.id)
                    .map(|a| a.jumlah)
                    .unwrap_or(0);
                (k.id, if n == 0 { String::new() } else { ribuan(n) })
            })
            .collect();
        app.ui_anggaran.isian = Some(map);
    }

    let realisasi = app.sumber().total_per_kategori(Periode::tahun(tahun));
    let kategori = app.kategori.clone();
    let isian = app.ui_anggaran.isian.as_mut().unwrap();

    for arus in [Arus::Masuk, Arus::Keluar] {
        let (mut ta, mut tr) = (0, 0);
        tema::kartu(ui, |ui| {
            tema::subjudul(
                ui,
                if arus == Arus::Masuk {
                    "Target Penerimaan"
                } else {
                    "Anggaran Belanja"
                },
            );
            ui.add_space(4.0);
            egui::ScrollArea::horizontal()
                .id_salt(("geser-anggaran", arus.kode()))
                .show(ui, |ui| {
                    egui::Grid::new(("anggaran", arus.kode()))
                        .num_columns(5)
                        .spacing([14.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            for h in ["Kategori", "Anggaran (Rp)", "Realisasi", "Sisa", "Capaian"] {
                                ui.label(RichText::new(h).strong().color(p.teks2));
                            }
                            ui.end_row();
                            for k in kategori.iter().filter(|k| k.arus == arus) {
                                let teks = isian.entry(k.id).or_default();
                                let a = parse_rupiah(teks).unwrap_or(0);
                                let r = realisasi.get(&k.id).copied().unwrap_or(0);
                                ta += a;
                                tr += r;
                                ui.label(format!("{}  {}", k.kode, k.nama));
                                tema::isian_rupiah(ui, teks, 150.0);
                                ui.label(rupiah(r));
                                let sisa = a - r;
                                match (arus, sisa < 0) {
                                    // Penerimaan melebihi target adalah surplus, bukan kekurangan.
                                    (Arus::Masuk, true) => ui.label(
                                        RichText::new(format!("+{}", rupiah(-sisa))).color(p.hijau),
                                    ),
                                    (Arus::Keluar, true) => {
                                        ui.label(RichText::new(rupiah(sisa)).color(p.merah))
                                    }
                                    _ => ui.label(rupiah(sisa)),
                                };
                                let rasio = if a > 0 { r as f32 / a as f32 } else { 0.0 };
                                let warna = match (arus, rasio) {
                                    (Arus::Keluar, x) if x > 1.0 => p.merah,
                                    (Arus::Keluar, x) if x > 0.9 => p.oranye,
                                    _ => p.aksen,
                                };
                                tema::bilah(ui, rasio, warna, &persen(r, a));
                                ui.end_row();
                            }
                            ui.label(RichText::new("Jumlah").strong());
                            ui.label(RichText::new(rupiah(ta)).strong());
                            ui.label(RichText::new(rupiah(tr)).strong());
                            ui.label(RichText::new(rupiah(ta - tr)).strong());
                            ui.label(RichText::new(persen(tr, ta)).strong());
                            ui.end_row();
                        });
                });
        });
        ui.add_space(12.0);
    }
    tema::label_redup(
        ui,
        "Laporan Realisasi Anggaran siap cetak tersedia di menu Laporan.",
    );
}

fn simpan(app: &mut MyDigitApp, ctx: &egui::Context) {
    let tahun = app.ui_anggaran.tahun;
    let Some(isian) = app.ui_anggaran.isian.clone() else {
        return;
    };
    let mut nilai = Vec::new();
    for (id, teks) in &isian {
        match parse_rupiah(teks) {
            Some(n) => nilai.push((*id, n)),
            None => {
                app.galat(format!("Nominal tidak valid: \"{teks}\""));
                return;
            }
        }
    }
    if app.ubah(ctx, &format!("Anggaran {tahun} tersimpan."), |db| {
        nilai
            .iter()
            .try_for_each(|(id, n)| db.atur_anggaran(tahun, *id, *n))
    }) {
        app.ui_anggaran.isian = None;
    }
}
