//! Data master: Akun Kas & Bank, dan Kategori.

use eframe::egui::{self, RichText};

use super::ikon;
use super::{MyDigitApp, pemisah, tema};
use crate::format::{parse_rupiah, ribuan, rupiah};
use crate::model::*;
use tema::palet;

#[derive(Default)]
pub struct State {
    akun: Option<FormAkun>,
    kategori: Option<FormKategori>,
    hapus_akun: Option<i64>,
    hapus_kategori: Option<i64>,
}

struct FormAkun {
    id: i64,
    nama: String,
    jenis: JenisAkun,
    no_rekening: String,
    saldo_awal: String,
    galat: Option<String>,
}

struct FormKategori {
    id: i64,
    kode: String,
    nama: String,
    arus: Arus,
    galat: Option<String>,
}

pub fn tampil_akun(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let contoh_bank = match app.instansi.sumber_dana.first() {
        Some(s) => format!("mis. Bank Riau Kepri — {s}"),
        None => "mis. Bank Riau Kepri".to_string(),
    };
    let ctx = ui.ctx().clone();
    let p = palet(ui);
    tema::baris_kepala(ui, |ui| {
        ui.vertical(|ui| {
            tema::judul(
                ui,
                "Akun Kas & Bank",
                "Tempat uang lembaga disimpan: brankas tunai dan rekening bank.",
            )
        });
        tema::kanan(ui, |ui| {
            if tema::tombol_utama(ui, ikon::PLUS, "Akun Baru").clicked() {
                app.ui_akun.akun = Some(FormAkun {
                    id: 0,
                    nama: String::new(),
                    jenis: JenisAkun::Bank,
                    no_rekening: String::new(),
                    saldo_awal: String::new(),
                    galat: None,
                });
            }
        });
    });

    tema::kartu(ui, |ui| {
        let saldo: Vec<i64> = app
            .akun
            .iter()
            .map(|a| app.sumber().saldo_akun(a.id))
            .collect();
        let daftar = app.akun.clone();
        for (i, a) in daftar.iter().enumerate() {
            if i > 0 {
                pemisah(ui);
            }
            ui.horizontal(|ui| {
                ui.label(
                    ikon::rt(if a.jenis == JenisAkun::Bank {
                        ikon::BANK
                    } else {
                        ikon::MONEY
                    })
                    .size(22.0)
                    .color(p.aksen),
                );
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new(&a.nama).strong());
                    let mut info = a.jenis.label().to_string();
                    if !a.no_rekening.is_empty() {
                        info += &format!(" · No. {}", a.no_rekening);
                    }
                    info += &format!(" · Saldo awal {}", rupiah(a.saldo_awal));
                    tema::label_redup(ui, info);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if tema::tombol_bahaya(ui, ikon::TRASH, "Hapus").clicked() {
                        app.ui_akun.hapus_akun = Some(a.id);
                    }
                    if tema::tombol(ui, ikon::PENCIL_SIMPLE, "Ubah").clicked() {
                        app.ui_akun.akun = Some(FormAkun {
                            id: a.id,
                            nama: a.nama.clone(),
                            jenis: a.jenis,
                            no_rekening: a.no_rekening.clone(),
                            saldo_awal: ribuan(a.saldo_awal),
                            galat: None,
                        });
                    }
                    ui.add_space(12.0);
                    ui.label(RichText::new(rupiah(saldo[i])).size(16.0).strong());
                });
            });
        }
    });
    ui.add_space(8.0);
    tema::label_redup(
        ui,
        "Saldo awal adalah saldo saat mulai memakai My Digit (mis. saldo per 1 Januari).",
    );

    if let Some(mut f) = app.ui_akun.akun.take() {
        let mut tutup = false;
        let mut simpan = false;
        let r = egui::Modal::new(egui::Id::unique("form-akun")).show(&ctx, |ui| {
            ui.set_width(tema::lebar_modal(ui, 440.0));
            tema::subjudul(ui, if f.id == 0 { "Akun Baru" } else { "Ubah Akun" });
            ui.add_space(6.0);
            tema::baris_form(ui, "Jenis", |ui| {
                tema::segmen(
                    ui,
                    &mut f.jenis,
                    &[(JenisAkun::Tunai, "Kas Tunai"), (JenisAkun::Bank, "Bank")],
                );
            });
            tema::baris_form(ui, "Nama akun", |ui| {
                tema::isian(ui, &mut f.nama, &contoh_bank)
            });
            if f.jenis == JenisAkun::Bank {
                tema::baris_form(ui, "No. rekening", |ui| {
                    tema::isian(ui, &mut f.no_rekening, "")
                });
            }
            tema::baris_form(ui, "Saldo awal (Rp)", |ui| {
                tema::isian_rupiah(ui, &mut f.saldo_awal, 200.0)
            });
            if let Some(g) = &f.galat {
                ui.label(RichText::new(g).color(palet(ui).merah));
            }
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                simpan = tema::tombol_utama(ui, "", "Simpan").clicked();
                tutup = tema::tombol(ui, "", "Batal").clicked();
            });
        });
        tutup |= r.should_close();
        if simpan {
            match (f.nama.trim().is_empty(), parse_rupiah(&f.saldo_awal)) {
                (true, _) => f.galat = Some("Nama akun wajib diisi.".into()),
                (_, None) => f.galat = Some("Saldo awal harus berupa angka.".into()),
                (false, Some(saldo)) => {
                    let a = Akun {
                        id: f.id,
                        nama: f.nama.trim().into(),
                        jenis: f.jenis,
                        no_rekening: if f.jenis == JenisAkun::Bank {
                            f.no_rekening.trim().into()
                        } else {
                            String::new()
                        },
                        saldo_awal: saldo,
                    };
                    tutup = app.ubah(&ctx, "Akun tersimpan.", |db| db.simpan_akun(&a));
                }
            }
        }
        if !tutup {
            app.ui_akun.akun = Some(f);
        }
    }

    if let Some(id) = app.ui_akun.hapus_akun {
        let nama = app.nama_akun(id).to_string();
        if let Some(ya) = konfirmasi(&ctx, "hapus-akun", &format!("Hapus akun \"{nama}\"?")) {
            if ya {
                app.ubah(&ctx, "Akun dihapus.", |db| db.hapus_akun(id));
            }
            app.ui_akun.hapus_akun = None;
        }
    }
}

pub fn tampil_kategori(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    tema::baris_kepala(ui, |ui| {
        ui.vertical(|ui| {
            let sub = format!(
                "Pengelompokan penerimaan dan pengeluaran, sesuaikan dengan kode rekening {}.",
                app.instansi.istilah_anggaran
            );
            tema::judul(ui, "Kategori", &sub)
        });
        tema::kanan(ui, |ui| {
            if tema::tombol_utama(ui, ikon::PLUS, "Kategori Baru").clicked() {
                app.ui_akun.kategori = Some(FormKategori {
                    id: 0,
                    kode: String::new(),
                    nama: String::new(),
                    arus: Arus::Keluar,
                    galat: None,
                });
            }
        });
    });

    let kolom = |app: &mut MyDigitApp, ui: &mut egui::Ui, arus: Arus| {
        tema::kartu(ui, |ui| {
            tema::subjudul(ui, arus.label());
            ui.add_space(4.0);
            let daftar: Vec<Kategori> = app
                .kategori
                .iter()
                .filter(|k| k.arus == arus)
                .cloned()
                .collect();
            for k in daftar {
                let r = ui.horizontal(|ui| {
                    ui.add_sized(
                        [44.0, 24.0],
                        egui::Label::new(RichText::new(&k.kode).monospace().color(palet(ui).teks2)),
                    );
                    ui.label(&k.nama);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let p = palet(ui);
                        if tema::tombol_ikon(ui, ikon::TRASH, "Hapus", p.merah).clicked() {
                            app.ui_akun.hapus_kategori = Some(k.id);
                        }
                        if tema::tombol_ikon(ui, ikon::PENCIL_SIMPLE, "Ubah", p.teks2).clicked() {
                            app.ui_akun.kategori = Some(FormKategori {
                                id: k.id,
                                kode: k.kode.clone(),
                                nama: k.nama.clone(),
                                arus: k.arus,
                                galat: None,
                            });
                        }
                    });
                });
                let _ = r;
            }
        });
    };
    if ui.available_width() > 700.0 {
        ui.columns(2, |c| {
            kolom(app, &mut c[0], Arus::Masuk);
            kolom(app, &mut c[1], Arus::Keluar);
        });
    } else {
        kolom(app, ui, Arus::Masuk);
        ui.add_space(12.0);
        kolom(app, ui, Arus::Keluar);
    }

    if let Some(mut f) = app.ui_akun.kategori.take() {
        let mut tutup = false;
        let mut simpan = false;
        let r = egui::Modal::new(egui::Id::unique("form-kategori")).show(&ctx, |ui| {
            ui.set_width(tema::lebar_modal(ui, 420.0));
            tema::subjudul(
                ui,
                if f.id == 0 {
                    "Kategori Baru"
                } else {
                    "Ubah Kategori"
                },
            );
            ui.add_space(6.0);
            tema::baris_form(ui, "Jenis", |ui| {
                tema::segmen(
                    ui,
                    &mut f.arus,
                    &[(Arus::Masuk, "Penerimaan"), (Arus::Keluar, "Pengeluaran")],
                );
            });
            tema::baris_form(ui, "Kode", |ui| tema::isian(ui, &mut f.kode, "mis. 5.2.1"));
            tema::baris_form(ui, "Nama", |ui| {
                tema::isian(ui, &mut f.nama, "mis. Belanja ATK")
            });
            if let Some(g) = &f.galat {
                ui.label(RichText::new(g).color(palet(ui).merah));
            }
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                simpan = tema::tombol_utama(ui, "", "Simpan").clicked();
                tutup = tema::tombol(ui, "", "Batal").clicked();
            });
        });
        tutup |= r.should_close();
        if simpan {
            if f.nama.trim().is_empty() {
                f.galat = Some("Nama kategori wajib diisi.".into());
            } else {
                let k = Kategori {
                    id: f.id,
                    kode: f.kode.trim().into(),
                    nama: f.nama.trim().into(),
                    arus: f.arus,
                };
                tutup = app.ubah(&ctx, "Kategori tersimpan.", |db| db.simpan_kategori(&k));
            }
        }
        if !tutup {
            app.ui_akun.kategori = Some(f);
        }
    }

    if let Some(id) = app.ui_akun.hapus_kategori {
        let nama = app
            .kategori(Some(id))
            .map(|k| k.nama.clone())
            .unwrap_or_default();
        if let Some(ya) = konfirmasi(
            &ctx,
            "hapus-kategori",
            &format!("Hapus kategori \"{nama}\"?"),
        ) {
            if ya {
                app.ubah(&ctx, "Kategori dihapus.", |db| db.hapus_kategori(id));
            }
            app.ui_akun.hapus_kategori = None;
        }
    }
}

/// Dialog ya/tidak. `Some(true)` bila disetujui, `Some(false)` bila batal.
fn konfirmasi(ctx: &egui::Context, id: &str, pertanyaan: &str) -> Option<bool> {
    let mut hasil = None;
    let r = egui::Modal::new(egui::Id::unique(id)).show(ctx, |ui| {
        ui.set_width(tema::lebar_modal(ui, 340.0));
        tema::subjudul(ui, pertanyaan);
        tema::label_redup(ui, "Tindakan ini tidak dapat dibatalkan.");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if tema::tombol_bahaya(ui, "", "Hapus").clicked() {
                hasil = Some(true);
            }
            if tema::tombol(ui, "", "Batal").clicked() {
                hasil = Some(false);
            }
        });
    });
    if r.should_close() && hasil.is_none() {
        hasil = Some(false);
    }
    hasil
}
