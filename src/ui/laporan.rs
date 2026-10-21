//! Halaman Laporan: pratinjau dan ekspor PDF / Excel.

use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};
use jiff::civil::Date;

use super::ikon;
use super::{Bulan, MyDigitApp, pilih_bulan, pilih_tahun, tema};
use crate::excel;
use crate::format::hari_ini;
use crate::laporan::{JenisLaporan, Periode, Tabel};
use crate::model::JenisAkun;
use crate::pdf;
use tema::palet;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModePeriode {
    Bulanan,
    Rentang,
}

pub struct State {
    jenis: JenisLaporan,
    mode: ModePeriode,
    bulan: Bulan,
    mulai: Date,
    sampai: Date,
    akun_bank: i64,
}

impl Default for State {
    fn default() -> Self {
        let h = hari_ini();
        Self {
            jenis: JenisLaporan::Bku,
            mode: ModePeriode::Bulanan,
            bulan: Bulan::sekarang(),
            mulai: h.first_of_month(),
            sampai: h,
            akun_bank: 0,
        }
    }
}

pub fn tampil(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let p = palet(ui);
    tema::judul(
        ui,
        "Laporan",
        "Cetak dan tandatangani BKU serta buku pembantu minimal sebulan sekali.",
    );

    // Pilihan jenis laporan sebagai kisi kartu kecil (3 kolom).
    let pajak_tampil = app.pajak_tampil();
    let jenis_tampil: Vec<JenisLaporan> = JenisLaporan::SEMUA
        .into_iter()
        .filter(|j| pajak_tampil || *j != JenisLaporan::PembantuPajak)
        .collect();
    if !jenis_tampil.contains(&app.ui_laporan.jenis) {
        app.ui_laporan.jenis = JenisLaporan::Bku;
    }
    for baris in jenis_tampil.chunks(3) {
        ui.columns(3, |c| {
            for (ui, j) in c.iter_mut().zip(baris) {
                let aktif = app.ui_laporan.jenis == *j;
                let frame = egui::Frame::new()
                    .fill(if aktif { p.pilih } else { p.kartu })
                    .stroke(egui::Stroke::new(
                        1.0,
                        if aktif { p.aksen } else { p.garis },
                    ))
                    .corner_radius(10)
                    .inner_margin(egui::Margin::symmetric(12, 10));
                let r = frame
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_min_height(38.0);
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new(j.judul()).strong());
                        ui.add(
                            egui::Label::new(
                                RichText::new(j.keterangan()).size(11.0).color(p.teks2),
                            )
                            .truncate(),
                        );
                    })
                    .response
                    .interact(egui::Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if r.clicked() {
                    app.ui_laporan.jenis = *j;
                }
            }
        });
        ui.add_space(4.0);
    }
    ui.add_space(12.0);

    let bank: Vec<(i64, String)> = app
        .akun
        .iter()
        .filter(|a| a.jenis == JenisAkun::Bank)
        .map(|a| (a.id, a.nama.clone()))
        .collect();
    let st = &mut app.ui_laporan;
    if !bank.iter().any(|(id, _)| *id == st.akun_bank) {
        st.akun_bank = bank.first().map(|b| b.0).unwrap_or(0);
    }

    ui.horizontal_wrapped(|ui| {
        if st.jenis == JenisLaporan::RealisasiAnggaran {
            tema::label_redup(ui, "Tahun anggaran");
            pilih_tahun(ui, &mut st.bulan.tahun);
        } else {
            tema::segmen(
                ui,
                &mut st.mode,
                &[
                    (ModePeriode::Bulanan, "Bulanan / Tahunan"),
                    (ModePeriode::Rentang, "Rentang Tanggal"),
                ],
            );
            ui.add_space(6.0);
            match st.mode {
                ModePeriode::Bulanan => {
                    pilih_bulan(ui, "laporan-bulan", &mut st.bulan, true);
                }
                ModePeriode::Rentang => {
                    tema::pilih_tanggal(ui, "lap-mulai", &mut st.mulai);
                    ui.label("s.d.");
                    tema::pilih_tanggal(ui, "lap-sampai", &mut st.sampai);
                }
            }
        }
        if st.jenis == JenisLaporan::PembantuBank {
            ui.add_space(6.0);
            let nama = bank
                .iter()
                .find(|b| b.0 == st.akun_bank)
                .map(|b| b.1.clone())
                .unwrap_or_else(|| "Tidak ada akun bank".into());
            egui::ComboBox::from_id_salt("lap-bank")
                .icon(tema::ikon_dropdown)
                .selected_text(nama)
                .show_ui(ui, |ui| {
                    for (id, n) in &bank {
                        ui.selectable_value(&mut st.akun_bank, *id, n);
                    }
                });
        }
    });

    let periode = match st.mode {
        ModePeriode::Bulanan => st.bulan.periode(),
        ModePeriode::Rentang => Periode {
            mulai: st.mulai.min(st.sampai),
            sampai: st.sampai.max(st.mulai),
        },
    };
    let (jenis, akun_bank, tahun) = (st.jenis, st.akun_bank, st.bulan.tahun);
    let s = app.sumber();
    let tabel = match jenis {
        JenisLaporan::Bku => s.bku(periode),
        JenisLaporan::PembantuKas => s.pembantu_kas(periode),
        JenisLaporan::PembantuBank => s.pembantu_bank(akun_bank, periode),
        JenisLaporan::PembantuPajak => s.pembantu_pajak(periode),
        JenisLaporan::RekapKategori => s.rekap_kategori(periode),
        JenisLaporan::RealisasiAnggaran => {
            let anggaran = app.db().daftar_anggaran(tahun).unwrap_or_default();
            s.realisasi_anggaran(tahun, &anggaran)
        }
    };

    ui.add_space(8.0);
    tema::kartu(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                tema::subjudul(ui, &tabel.judul);
                tema::label_redup(ui, &tabel.subjudul);
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if tema::tombol_utama(ui, ikon::FILE_PDF, "Ekspor PDF").clicked() {
                    let bytes =
                        pdf::laporan(&app.profil, app.gambar_pdf().as_ref(), &tabel, hari_ini());
                    app.simpan_berkas(&nama_berkas(&tabel, "pdf"), "pdf", &bytes);
                }
                if tema::tombol(ui, ikon::FILE_XLS, "Ekspor Excel").clicked() {
                    let logo = app.path_logo_lembaga();
                    match excel::laporan(&app.profil, logo.as_deref(), &tabel, hari_ini()) {
                        Ok(bytes) => {
                            app.simpan_berkas(&nama_berkas(&tabel, "xlsx"), "xlsx", &bytes)
                        }
                        Err(e) => app.galat(e),
                    }
                }
            });
        });
        ui.add_space(8.0);
        pratinjau(ui, &tabel);
    });
}

fn nama_berkas(t: &Tabel, ekstensi: &str) -> String {
    let sub: String = t
        .subjudul
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let sub = sub.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("{} - {}.{ekstensi}", t.judul, sub)
}

fn pratinjau(ui: &mut egui::Ui, t: &Tabel) {
    let p = palet(ui);
    let satuan: f32 = t.kolom.iter().map(|k| k.lebar).sum();
    let lebar = ui.available_width() - 8.0 * t.kolom.len() as f32;
    let mut tb = TableBuilder::new(ui)
        .id_salt(("pratinjau", t.judul.as_str()))
        .striped(true)
        .max_scroll_height(f32::INFINITY)
        .vscroll(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center));
    for k in &t.kolom {
        tb = tb.column(Column::exact((k.lebar / satuan * lebar).max(40.0)).clip(true));
    }
    let sel = |ui: &mut egui::Ui, s: &str, kanan: bool, tebal: bool| {
        let teks = if tebal {
            RichText::new(s).strong()
        } else {
            RichText::new(s)
        };
        if kanan {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(teks)
            });
        } else {
            ui.add(egui::Label::new(teks).truncate());
        }
    };
    tb.header(26.0, |mut h| {
        for k in &t.kolom {
            h.col(|ui| sel(ui, k.nama, k.kanan, true));
        }
    })
    .body(|mut body| {
        for r in &t.baris {
            let tebal = pdf::baris_bagian(r);
            body.row(24.0, |mut row| {
                for (k, s) in t.kolom.iter().zip(r) {
                    row.col(|ui| sel(ui, s, k.kanan, tebal));
                }
            });
        }
        if let Some(total) = &t.total {
            body.row(28.0, |mut row| {
                for (k, s) in t.kolom.iter().zip(total) {
                    row.col(|ui| {
                        ui.painter().rect_filled(
                            ui.max_rect().expand(2.0),
                            0,
                            p.pilih.gamma_multiply(0.6),
                        );
                        sel(ui, s, k.kanan, true)
                    });
                }
            });
        }
    });
}
