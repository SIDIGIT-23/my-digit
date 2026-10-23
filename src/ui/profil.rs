//! Halaman "Profil {Jenis}" (mis. "Profil Sekolah"): jenis pengguna,
//! identitas untuk kop, penanda tangan, pengaturan pembukuan, dan logo.

use eframe::egui::{self, RichText, Vec2};

use super::ikon;
use super::{MyDigitApp, tema};
use crate::db::folder_data;
use crate::format::{hari_ini, tanggal_panjang};
use crate::instansi::{JenisInstansi, awalan_valid, periksa_nomor};
use crate::model::{JenisTransaksi, Kategori, PengaturanInstansi, ProfilLembaga};
use tema::palet;

#[derive(Default)]
pub struct State {
    /// Salinan profil yang sedang diedit.
    pub draf: Option<ProfilLembaga>,
    pub draf_instansi: Option<PengaturanInstansi>,
    /// Lembar "Ubah jenis" yang sedang terbuka.
    pub ubah: Option<UbahJenis>,
    sumber_baru: String,
}

impl State {
    /// Buka lembar "Ubah jenis" (dipakai juga oleh pintasan pengembangan).
    pub fn buka_ubah_jenis(app: &mut MyDigitApp, jenis: JenisInstansi) {
        app.ui_profil.ubah = Some(UbahJenis::baru(app, jenis));
    }
}

/// Satu baris "Yang akan berubah" pada lembar Ubah jenis.
pub struct Perubahan {
    label: &'static str,
    dari: String,
    ke: String,
    centang: bool,
    terapkan: fn(&mut ProfilLembaga, &mut PengaturanInstansi, &JenisInstansi),
}

pub struct UbahJenis {
    jenis: JenisInstansi,
    perubahan: Vec<Perubahan>,
    tambah_kategori: bool,
    rencana: Vec<Kategori>,
}

impl UbahJenis {
    fn baru(app: &MyDigitApp, jenis: JenisInstansi) -> Self {
        let (p, i) = (&app.profil, &app.instansi);
        let lama = i.jenis.templat();
        let t = jenis.templat();
        let ya_tidak = |b: bool| if b { "Aktif" } else { "Nonaktif" }.to_string();
        let kandidat: Vec<Perubahan> = vec![
            Perubahan {
                label: "Jabatan pimpinan",
                dari: p.jabatan_pimpinan.clone(),
                ke: t.pimpinan.into(),
                centang: p.jabatan_pimpinan == lama.pimpinan,
                terapkan: |p, _, j| p.jabatan_pimpinan = j.templat().pimpinan.into(),
            },
            Perubahan {
                label: "Jabatan bendahara",
                dari: p.jabatan_bendahara.clone(),
                ke: t.bendahara.into(),
                centang: p.jabatan_bendahara == lama.bendahara,
                terapkan: |p, _, j| p.jabatan_bendahara = j.templat().bendahara.into(),
            },
            Perubahan {
                label: "Label nomor induk pimpinan",
                dari: p.label_nomor_pimpinan.clone(),
                ke: t.label_nomor.into(),
                centang: p.label_nomor_pimpinan == lama.label_nomor,
                terapkan: |p, _, j| p.label_nomor_pimpinan = j.templat().label_nomor.into(),
            },
            Perubahan {
                label: "Label nomor induk bendahara",
                dari: p.label_nomor_bendahara.clone(),
                ke: t.label_nomor.into(),
                centang: p.label_nomor_bendahara == lama.label_nomor,
                terapkan: |p, _, j| p.label_nomor_bendahara = j.templat().label_nomor.into(),
            },
            Perubahan {
                label: "Label identitas 1",
                dari: p.id1_label.clone(),
                ke: t.id1.into(),
                centang: p.id1_label == lama.id1 && p.id1_nomor.trim().is_empty(),
                terapkan: |p, _, j| p.id1_label = j.templat().id1.into(),
            },
            Perubahan {
                label: "Label identitas 2",
                dari: p.id2_label.clone(),
                ke: t.id2.into(),
                centang: p.id2_label == lama.id2 && p.id2_nomor.trim().is_empty(),
                terapkan: |p, _, j| p.id2_label = j.templat().id2.into(),
            },
            Perubahan {
                label: "Fitur pajak",
                dari: ya_tidak(i.pajak),
                ke: ya_tidak(t.pajak),
                centang: i.pajak == lama.pajak,
                terapkan: |_, i, j| i.pajak = j.templat().pajak,
            },
            Perubahan {
                label: "Istilah anggaran",
                dari: i.istilah_anggaran.clone(),
                ke: t.istilah.into(),
                centang: i.istilah_anggaran == lama.istilah,
                terapkan: |_, i, j| i.istilah_anggaran = j.templat().istilah.into(),
            },
            Perubahan {
                label: "Sumber dana",
                dari: i.sumber_dana.join(", "),
                ke: t.sumber_dana.join(", "),
                centang: i
                    .sumber_dana
                    .iter()
                    .map(String::as_str)
                    .eq(lama.sumber_dana.iter().copied()),
                terapkan: |_, i, j| {
                    i.sumber_dana = j
                        .templat()
                        .sumber_dana
                        .iter()
                        .map(|s| s.to_string())
                        .collect()
                },
            },
        ];
        let tampil = |s: &str| {
            if s.is_empty() {
                "(kosong)".to_string()
            } else {
                s.to_string()
            }
        };
        Self {
            jenis,
            perubahan: kandidat
                .into_iter()
                .filter(|c| c.dari != c.ke)
                .map(|c| Perubahan {
                    dari: tampil(&c.dari),
                    ke: tampil(&c.ke),
                    ..c
                })
                .collect(),
            tambah_kategori: true,
            rencana: app.db().rencana_kategori(jenis).unwrap_or_default(),
        }
    }
}

pub fn tampil(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let p = palet(ui);
    let mut d = app
        .ui_profil
        .draf
        .take()
        .unwrap_or_else(|| app.profil.clone());
    let mut di = app
        .ui_profil
        .draf_instansi
        .take()
        .unwrap_or_else(|| app.instansi.clone());
    let berubah = d != app.profil || di != app.instansi;
    let awalan_ok = [&di.awalan_masuk, &di.awalan_keluar, &di.awalan_transfer]
        .iter()
        .all(|a| awalan_valid(a));
    let jenis = app.instansi.jenis;
    let t = jenis.templat();

    tema::baris_kepala(ui, |ui| {
        ui.vertical(|ui| {
            tema::judul(
                ui,
                &app.judul_profil(),
                "Dipakai untuk kop, tanda tangan laporan, dan kwitansi.",
            )
        });
        tema::kanan(ui, |ui| {
            let simpan = ui
                .add_enabled_ui(berubah && awalan_ok, |ui| {
                    tema::tombol_utama(ui, "", "Simpan")
                })
                .inner;
            if simpan.clicked() {
                let (sp, si) = (d.clone(), di.clone());
                // Draf hanya direset bila tersimpan; bila gagal, isian tetap ada.
                if app.ubah(&ctx, "Profil tersimpan.", |db| {
                    db.terapkan_jenis(&si, &sp, &[]).map(|_| ())
                }) {
                    d = app.profil.clone();
                    di = app.instansi.clone();
                }
            }
            if berubah && tema::tombol(ui, "", "Batalkan").clicked() {
                d = app.profil.clone();
                di = app.instansi.clone();
            }
        });
    });

    // ---------- Masuk sebagai ----------
    tema::kartu(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(ikon::rt(jenis.ikon()).size(28.0).color(p.aksen));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(RichText::new(format!("Masuk sebagai {}", jenis.label())).strong());
                let sub = if app.instansi.subjenis.is_empty() {
                    jenis.keterangan().to_string()
                } else {
                    format!("{} · {}", app.instansi.subjenis, jenis.keterangan())
                };
                ui.add(egui::Label::new(RichText::new(sub).size(12.0).color(p.teks2)).truncate());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let r = ui
                    .add_enabled_ui(!berubah, |ui| tema::tombol(ui, ikon::SWAP, "Ubah…"))
                    .inner;
                let r = r.on_disabled_hover_text("Simpan atau batalkan perubahan terlebih dahulu.");
                if r.clicked() {
                    State::buka_ubah_jenis(app, jenis);
                }
            });
        });
    });
    ui.add_space(12.0);

    // ---------- Identitas ----------
    tema::kartu(ui, |ui| {
        tema::subjudul(ui, "Identitas");
        ui.add_space(4.0);
        tema::baris_form(ui, &format!("Nama {}", jenis.kata()), |ui| {
            tema::isian(ui, &mut d.nama, t.contoh_nama)
        });
        tema::baris_form(ui, "Jenis", |ui| {
            tema::isian_saran(ui, &mut di.subjenis, "opsional", 160.0, t.subjenis)
        });
        tema::baris_form(ui, "Instansi induk", |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut d.induk)
                    .hint_text(t.contoh_induk)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            )
        });
        catatan(
            ui,
            "Tampil di atas nama pada kop, huruf kapital. Maksimal 2 baris.",
        );
        tema::baris_form(ui, "Alamat", |ui| {
            tema::isian(ui, &mut d.alamat, "Jalan, kelurahan, kecamatan")
        });
        tema::baris_form(ui, "Kota / kabupaten", |ui| {
            tema::isian(ui, &mut d.kota, "mis. Dumai")
        });
        for (n, (label, nomor)) in [
            (&mut d.id1_label, &mut d.id1_nomor),
            (&mut d.id2_label, &mut d.id2_nomor),
        ]
        .into_iter()
        .enumerate()
        {
            tema::baris_form(ui, &format!("Nomor identitas {}", n + 1), |ui| {
                isian_label_nomor(ui, ("id", n), label, nomor, t.pilihan_id, 150.0);
            });
            if let Some(w) = periksa_nomor(label, nomor) {
                catatan_oranye(ui, w);
            }
        }
        tema::baris_form(ui, "Telepon", |ui| tema::isian(ui, &mut d.telepon, ""));
        tema::baris_form(ui, "Email", |ui| tema::isian(ui, &mut d.email, ""));
        if bukan_latin1(&d.nama) || bukan_latin1(&d.induk) || bukan_latin1(&d.alamat) {
            catatan_oranye(
                ui,
                "Ada huruf yang tidak dapat dicetak di PDF (akan tampil sebagai \"?\").",
            );
        }
    });
    ui.add_space(12.0);

    // ---------- Penanda tangan ----------
    tema::kartu(ui, |ui| {
        tema::subjudul(ui, "Penanda Tangan");
        ui.add_space(4.0);
        tema::baris_form(ui, "Jabatan pimpinan", |ui| {
            tema::isian_saran(
                ui,
                &mut d.jabatan_pimpinan,
                t.pimpinan,
                200.0,
                t.pilihan_pimpinan,
            )
        });
        tema::baris_form(ui, "Nama pimpinan", |ui| {
            tema::isian(ui, &mut d.nama_pimpinan, "")
        });
        tema::baris_form(ui, "Nomor induk pimpinan", |ui| {
            isian_label_nomor(
                ui,
                ("nomor", 0),
                &mut d.label_nomor_pimpinan,
                &mut d.nip_pimpinan,
                t.pilihan_label_nomor,
                90.0,
            );
        });
        tema::baris_form(ui, "Jabatan bendahara", |ui| {
            tema::isian_saran(
                ui,
                &mut d.jabatan_bendahara,
                t.bendahara,
                200.0,
                t.pilihan_bendahara,
            )
        });
        tema::baris_form(ui, "Nama bendahara", |ui| {
            tema::isian(ui, &mut d.nama_bendahara, "")
        });
        tema::baris_form(ui, "Nomor induk bendahara", |ui| {
            isian_label_nomor(
                ui,
                ("nomor", 1),
                &mut d.label_nomor_bendahara,
                &mut d.nip_bendahara,
                t.pilihan_label_nomor,
                90.0,
            );
        });
        ui.add_space(8.0);
        tema::label_redup(ui, "Pratinjau tanda tangan pada laporan:");
        ui.columns(2, |c| {
            let kota = if d.kota.is_empty() {
                String::new()
            } else {
                format!("{}, ", d.kota)
            };
            let blok = [
                [
                    "Mengetahui,".to_string(),
                    d.jabatan_pimpinan.clone(),
                    d.nama_pimpinan.clone(),
                    ProfilLembaga::baris_nomor(&d.label_nomor_pimpinan, &d.nip_pimpinan),
                ],
                [
                    format!("{kota}{}", tanggal_panjang(hari_ini())),
                    d.jabatan_bendahara.clone(),
                    d.nama_bendahara.clone(),
                    ProfilLembaga::baris_nomor(&d.label_nomor_bendahara, &d.nip_bendahara),
                ],
            ];
            for (ui, [atas, jabatan, nama, nomor]) in c.iter_mut().zip(blok) {
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(RichText::new(atas).size(12.0));
                    ui.label(RichText::new(jabatan).size(12.0));
                    ui.add_space(14.0);
                    let nama = if nama.is_empty() {
                        "(...............................)".to_string()
                    } else {
                        nama
                    };
                    ui.label(RichText::new(nama).size(12.0).strong().underline());
                    if !nomor.is_empty() {
                        ui.label(RichText::new(nomor).size(11.0).color(p.teks2));
                    }
                });
            }
        });
    });
    ui.add_space(12.0);

    // ---------- Pembukuan ----------
    tema::kartu(ui, |ui| {
        tema::subjudul(ui, "Pembukuan");
        ui.add_space(4.0);
        ui.checkbox(&mut di.pajak, "Gunakan fitur pajak (PPN, PPh 21/22/23)");
        if !di.pajak && app.ada_data_pajak() {
            catatan(
                ui,
                "Bagian pajak tetap tampil karena sudah ada transaksi berpajak.",
            );
        }
        tema::baris_form(ui, "Istilah anggaran", |ui| {
            tema::isian_saran(
                ui,
                &mut di.istilah_anggaran,
                t.istilah,
                160.0,
                t.pilihan_istilah,
            )
        });
        tema::baris_form(ui, "Sumber dana", |ui| {
            ui.horizontal_wrapped(|ui| {
                let mut hapus = None;
                for (i, s) in di.sumber_dana.iter().enumerate() {
                    let r = ui.add(
                        egui::Button::new((
                            RichText::new(s),
                            ikon::rt(ikon::X).size(11.0).color(p.teks2),
                        ))
                        .small(),
                    );
                    if r.on_hover_text("Hapus").clicked() {
                        hapus = Some(i);
                    }
                }
                if let Some(i) = hapus {
                    di.sumber_dana.remove(i);
                }
                let baru = &mut app.ui_profil.sumber_baru;
                let r = ui.add(
                    egui::TextEdit::singleline(baru)
                        .hint_text("tambah…")
                        .desired_width(110.0),
                );
                let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.small_button("+ Tambah").clicked() || enter) && !baru.trim().is_empty() {
                    let s = baru.trim().replace('|', "");
                    if !di.sumber_dana.contains(&s) {
                        di.sumber_dana.push(s);
                    }
                    baru.clear();
                }
            });
        });
        catatan(
            ui,
            "Tombol cepat pada isian sumber dana di formulir transaksi.",
        );
        tema::baris_form(ui, "Awalan nomor bukti", |ui| {
            ui.horizontal(|ui| {
                for (label, a) in [
                    ("Pemasukan", &mut di.awalan_masuk),
                    ("Pengeluaran", &mut di.awalan_keluar),
                    ("Transfer", &mut di.awalan_transfer),
                ] {
                    tema::label_redup(ui, label);
                    let r = ui.add(
                        egui::TextEdit::singleline(a)
                            .desired_width(64.0)
                            .char_limit(8),
                    );
                    if r.changed() {
                        *a = a.to_uppercase();
                    }
                    if !awalan_valid(a) {
                        ui.label(ikon::rt(ikon::WARNING_CIRCLE).color(p.oranye))
                            .on_hover_text(
                                "1–8 huruf besar atau angka, tanpa spasi atau tanda baca.",
                            );
                    }
                }
            });
        });
        let contoh = format!(
            "Contoh: {}-{}-0001",
            di.awalan(JenisTransaksi::Masuk),
            hari_ini().year()
        );
        catatan(
            ui,
            &format!(
                "{contoh}. Berlaku untuk transaksi baru; nomor yang sudah ada tidak diubah. \
                 Awalan yang sama berarti satu urutan nomor."
            ),
        );
    });
    ui.add_space(12.0);

    // ---------- Logo ----------
    tema::kartu(ui, |ui| {
        tema::subjudul(ui, &format!("Logo {}", jenis.label()));
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
                        jenis.ikon(),
                        ikon::font(34.0),
                        p.teks2,
                    );
                }
            }
            ui.vertical(|ui| {
                if tema::tombol(ui, ikon::IMAGE, "Pilih Gambar…").clicked() {
                    pilih_logo(app, &mut d, &ctx);
                }
                if d.logo.is_some() && tema::tombol_bahaya(ui, ikon::TRASH, "Hapus Logo").clicked()
                {
                    d.logo = None;
                }
            });
        });
    });

    app.ui_profil.draf = Some(d);
    app.ui_profil.draf_instansi = Some(di);
    lembar_ubah_jenis(app, &ctx);
}

/// Lembar "Masuk sebagai…" untuk mengubah jenis tanpa menghapus data.
fn lembar_ubah_jenis(app: &mut MyDigitApp, ctx: &egui::Context) {
    let Some(mut u) = app.ui_profil.ubah.take() else {
        return;
    };
    let mut tutup = false;
    let mut terapkan = false;
    let mut pilih_lain = None;
    let r = egui::Modal::new(egui::Id::unique("ubah-jenis")).show(ctx, |ui| {
        let p = palet(ui);
        ui.set_width(tema::lebar_modal(ui, 520.0));
        tema::subjudul(ui, "Masuk sebagai…");
        ui.add_space(4.0);
        let kolom = if tema::ruang_kurang(ui, 470.0) { 2 } else { 4 };
        for baris in JenisInstansi::SEMUA.chunks(kolom) {
            ui.columns(kolom, |c| {
                for (ui, j) in c.iter_mut().zip(baris.iter().copied()) {
                    let aktif = u.jenis == j;
                    let r = egui::Frame::new()
                        .fill(if aktif { p.pilih } else { p.kartu })
                        .stroke(egui::Stroke::new(
                            if aktif { 1.5 } else { 1.0 },
                            if aktif { p.aksen } else { p.garis },
                        ))
                        .corner_radius(10)
                        .inner_margin(egui::Margin::symmetric(6, 10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.vertical_centered(|ui| {
                                ui.label(ikon::rt(j.ikon()).size(22.0).color(p.aksen));
                                ui.label(RichText::new(j.label()).strong());
                            });
                        })
                        .response
                        .interact(egui::Sense::click())
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if r.clicked() && !aktif {
                        pilih_lain = Some(j);
                    }
                }
            });
        }
        ui.add_space(8.0);
        if u.jenis == app.instansi.jenis {
            tema::label_redup(
                ui,
                format!(
                    "Saat ini masuk sebagai {}. Pilih jenis lain.",
                    u.jenis.label()
                ),
            );
        } else {
            if !u.perubahan.is_empty() {
                ui.label(RichText::new("Yang akan berubah").strong());
                for c in &mut u.perubahan {
                    ui.checkbox(
                        &mut c.centang,
                        format!("{}: {} → {}", c.label, c.dari, c.ke),
                    );
                }
                ui.add_space(4.0);
            }
            let n = u.rencana.len();
            if n > 0 {
                ui.checkbox(
                    &mut u.tambah_kategori,
                    format!("Tambahkan {n} kategori {} yang belum ada", u.jenis.label()),
                );
                egui::CollapsingHeader::new("Lihat kategori")
                    .icon(tema::ikon_lipat)
                    .show(ui, |ui| {
                        egui::Grid::new("rencana-kategori")
                            .num_columns(2)
                            .spacing([12.0, 2.0])
                            .show(ui, |ui| {
                                for k in &u.rencana {
                                    ui.label(RichText::new(&k.kode).monospace().color(p.teks2));
                                    ui.label(&k.nama);
                                    ui.end_row();
                                }
                            });
                    });
            } else {
                tema::label_redup(ui, format!("Semua kategori {} sudah ada.", u.jenis.label()));
            }
            ui.add_space(4.0);
            tema::label_redup(
                ui,
                "Kategori, akun, dan transaksi yang ada tidak diubah atau dihapus.",
            );
        }
        ui.add_space(8.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let bisa = u.jenis != app.instansi.jenis;
            if ui
                .add_enabled_ui(bisa, |ui| tema::tombol_utama(ui, "", "Terapkan"))
                .inner
                .clicked()
            {
                terapkan = true;
            }
            if tema::tombol(ui, "", "Batal").clicked() {
                tutup = true;
            }
        });
    });
    if r.should_close() {
        tutup = true;
    }
    if let Some(j) = pilih_lain {
        u = UbahJenis::baru(app, j);
    }
    if terapkan {
        let (mut p, mut i) = (app.profil.clone(), app.instansi.clone());
        for c in u.perubahan.iter().filter(|c| c.centang) {
            (c.terapkan)(&mut p, &mut i, &u.jenis);
        }
        i.jenis = u.jenis;
        let kategori = if u.tambah_kategori {
            u.rencana.clone()
        } else {
            Vec::new()
        };
        let n = kategori.len();
        let pesan = if n > 0 {
            format!(
                "Jenis diubah menjadi {}. {n} kategori ditambahkan.",
                u.jenis.label()
            )
        } else {
            format!("Jenis diubah menjadi {}.", u.jenis.label())
        };
        if app.ubah(ctx, &pesan, |db| {
            db.terapkan_jenis(&i, &p, &kategori).map(|_| ())
        }) {
            app.reset_ui();
        } else {
            app.ui_profil.ubah = Some(u);
        }
        return;
    }
    if !tutup {
        app.ui_profil.ubah = Some(u);
    }
}

/// Isian label + nomor, dengan menu saran label (termasuk "Tanpa label").
fn isian_label_nomor(
    ui: &mut egui::Ui,
    id: (&str, usize),
    label: &mut String,
    nomor: &mut String,
    saran: &[&str],
    lebar_label: f32,
) {
    ui.horizontal(|ui| {
        ui.push_id(id, |ui| {
            ui.add(
                egui::TextEdit::singleline(label)
                    .hint_text("Label")
                    .desired_width(lebar_label),
            );
            ui.menu_button(ikon::rt(ikon::CARET_UP_DOWN), |ui| {
                for s in saran {
                    if ui.button(*s).clicked() {
                        *label = s.to_string();
                        ui.close();
                    }
                }
                if ui.button("Tanpa label").clicked() {
                    label.clear();
                    ui.close();
                }
            });
            ui.add(
                egui::TextEdit::singleline(nomor)
                    .hint_text("Nomor")
                    .desired_width(f32::INFINITY),
            );
        });
    });
}

fn catatan(ui: &mut egui::Ui, s: &str) {
    ui.horizontal(|ui| {
        if !tema::form_bertumpuk(ui) {
            ui.add_space(158.0);
        }
        ui.add(egui::Label::new(RichText::new(s).size(11.0).color(palet(ui).teks2)).wrap());
    });
}

fn catatan_oranye(ui: &mut egui::Ui, s: &str) {
    ui.horizontal(|ui| {
        if !tema::form_bertumpuk(ui) {
            ui.add_space(158.0);
        }
        let p = palet(ui);
        ui.label(ikon::rt(ikon::WARNING_CIRCLE).color(p.oranye));
        ui.add(egui::Label::new(RichText::new(s).size(11.0).color(p.oranye)).wrap());
    });
}

/// Huruf yang tidak bisa dicetak dengan font standar PDF (di luar Latin-1
/// dan tanda baca yang dipetakan di `pdf::win_ansi`).
fn bukan_latin1(s: &str) -> bool {
    s.chars()
        .any(|c| (c as u32) > 0xFF && !"—–−‘’“”•·€…".contains(c))
}

/// Salin gambar pilihan ke folder data, lalu langsung simpan ke profil.
fn pilih_logo(app: &mut MyDigitApp, d: &mut ProfilLembaga, ctx: &egui::Context) {
    if !crate::berkas::BISA_MEMILIH {
        app.galat("Memilih gambar belum tersedia di Android.");
        return;
    }
    let Some(src) = crate::berkas::pilih_buka("Gambar", &["png", "jpg", "jpeg"]) else {
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
    app.ubah(ctx, "Logo diperbarui.", |db| db.simpan_profil(&simpan));
    // Pertahankan isian lain yang belum disimpan.
    *d = draf;
}
