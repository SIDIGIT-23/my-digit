//! Halaman Transaksi: daftar, formulir, kwitansi.

use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};
use jiff::civil::Date;

use super::ikon;
use super::{Bulan, MyDigitApp, pilih_bulan, tema};
use crate::format::{hari_ini, parse_rupiah, ribuan, rupiah, tanggal_pendek};
use crate::model::*;
use crate::pdf;
use tema::palet;

pub struct State {
    filter: Option<JenisTransaksi>,
    bulan: Bulan,
    cari: String,
    terpilih: Option<i64>,
    form: Option<Form>,
    hapus: Option<i64>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            filter: None,
            bulan: Bulan::sekarang(),
            cari: String::new(),
            terpilih: None,
            form: None,
            hapus: None,
        }
    }
}

struct Form {
    id: i64,
    jenis: JenisTransaksi,
    tanggal: Date,
    no_bukti: String,
    /// Nomor terakhir yang dibuat otomatis; bila belum diubah pengguna,
    /// nomor diperbarui saat jenis atau tahun berganti.
    bukti_otomatis: String,
    akun_id: i64,
    tujuan_id: i64,
    kategori_id: Option<i64>,
    uraian: String,
    pihak: String,
    sumber_dana: String,
    jumlah: String,
    pajak: [String; 4],
    setor: Option<JenisPajak>,
    galat: Option<String>,
}

pub fn buka_baru(app: &mut MyDigitApp, jenis: JenisTransaksi) {
    let tanggal = hari_ini();
    let no = app
        .db()
        .nomor_bukti_berikutnya(app.instansi.awalan(jenis), tanggal.year())
        .unwrap_or_default();
    let tunai = app
        .akun
        .iter()
        .find(|a| a.jenis == JenisAkun::Tunai)
        .or(app.akun.first())
        .map(|a| a.id)
        .unwrap_or(0);
    let bank = app
        .akun
        .iter()
        .find(|a| a.jenis == JenisAkun::Bank && a.id != tunai)
        .map(|a| a.id)
        .unwrap_or(0);
    let (akun_id, tujuan_id) = if jenis == JenisTransaksi::Transfer {
        (bank, tunai)
    } else {
        (tunai, bank)
    };
    app.ui_transaksi.form = Some(Form {
        id: 0,
        jenis,
        tanggal,
        bukti_otomatis: no.clone(),
        no_bukti: no,
        akun_id,
        tujuan_id,
        kategori_id: None,
        uraian: String::new(),
        pihak: String::new(),
        sumber_dana: String::new(),
        jumlah: String::new(),
        pajak: Default::default(),
        setor: None,
        galat: None,
    });
}

pub fn buka_edit(app: &mut MyDigitApp, t: &Transaksi) {
    let isi = |n: i64| if n == 0 { String::new() } else { ribuan(n) };
    app.ui_transaksi.form = Some(Form {
        id: t.id,
        jenis: t.jenis,
        tanggal: t.tanggal,
        no_bukti: t.no_bukti.clone(),
        bukti_otomatis: String::new(),
        akun_id: t.akun_id,
        tujuan_id: t.akun_tujuan_id.unwrap_or(0),
        kategori_id: t.kategori_id,
        uraian: t.uraian.clone(),
        pihak: t.pihak.clone(),
        sumber_dana: t.sumber_dana.clone(),
        jumlah: ribuan(t.jumlah),
        pajak: JenisPajak::SEMUA.map(|p| isi(t.pungutan.get(p))),
        setor: t.setor_pajak,
        galat: None,
    });
}

pub fn cetak_kwitansi(app: &mut MyDigitApp, t: &Transaksi) {
    let bytes = pdf::kwitansi(
        &app.profil,
        app.gambar_pdf().as_ref(),
        t,
        app.kategori(t.kategori_id),
    );
    app.simpan_berkas(&format!("Kwitansi {}.pdf", t.no_bukti), "pdf", &bytes);
}

pub fn tampil(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let p = palet(ui);
    tema::baris_kepala(ui, |ui| {
        ui.vertical(|ui| {
            tema::judul(
                ui,
                "Transaksi",
                "Catat setiap penerimaan dan pengeluaran segera setelah terjadi.",
            )
        });
        tema::kanan(ui, |ui| {
            if tema::tombol_utama(ui, ikon::PLUS, "Transaksi Baru").clicked() {
                buka_baru(app, JenisTransaksi::Masuk);
            }
        });
    });

    let st = &mut app.ui_transaksi;
    ui.horizontal_wrapped(|ui| {
        tema::segmen(
            ui,
            &mut st.filter,
            &[
                (None, "Semua"),
                (Some(JenisTransaksi::Masuk), "Pemasukan"),
                (Some(JenisTransaksi::Keluar), "Pengeluaran"),
                (Some(JenisTransaksi::Transfer), "Transfer"),
            ],
        );
        ui.add_space(8.0);
        pilih_bulan(ui, "transaksi-bulan", &mut st.bulan, true);
        ui.add_space(8.0);
        ui.add(
            egui::TextEdit::singleline(&mut st.cari)
                .hint_text(tema::petunjuk_ikon(
                    ui,
                    ikon::MAGNIFYING_GLASS,
                    "Cari uraian, nomor bukti, pihak…",
                ))
                .desired_width(240.0),
        );
    });
    ui.add_space(8.0);

    let periode = st.bulan.periode();
    let cari = st.cari.to_lowercase();
    let filter = st.filter;
    let daftar: Vec<Transaksi> = app
        .transaksi
        .iter()
        .rev()
        .filter(|t| periode.memuat(t.tanggal))
        .filter(|t| filter.is_none_or(|f| f == t.jenis))
        .filter(|t| {
            cari.is_empty()
                || t.uraian.to_lowercase().contains(&cari)
                || t.no_bukti.to_lowercase().contains(&cari)
                || t.pihak.to_lowercase().contains(&cari)
        })
        .cloned()
        .collect();

    let terpilih = app
        .ui_transaksi
        .terpilih
        .and_then(|id| daftar.iter().find(|t| t.id == id).cloned());
    let ponsel = tema::sempit(ui);
    ui.horizontal_wrapped(|ui| {
        let (m, k) = daftar.iter().fold((0, 0), |(m, k), t| match t.jenis {
            JenisTransaksi::Masuk => (m + t.jumlah, k),
            JenisTransaksi::Keluar => (m, k + t.jumlah),
            JenisTransaksi::Transfer => (m, k),
        });
        tema::label_redup(ui, format!("{} transaksi", daftar.len()));
        ui.label(RichText::new(format!("Masuk {}", rupiah(m))).color(p.hijau));
        ui.label(RichText::new(format!("Keluar {}", rupiah(k))).color(p.merah));
        // Di ponsel, ketuk baris untuk mengubah; kwitansi & hapus ada di formulir.
        if ponsel {
            return;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled_ui(terpilih.is_some(), |ui| {
                if tema::tombol_bahaya(ui, ikon::TRASH, "Hapus").clicked() {
                    app.ui_transaksi.hapus = terpilih.as_ref().map(|t| t.id);
                }
                let bisa_kwitansi = terpilih
                    .as_ref()
                    .is_some_and(|t| t.jenis != JenisTransaksi::Transfer);
                if ui
                    .add_enabled(
                        bisa_kwitansi,
                        egui::Button::new((ikon::rt(ikon::PRINTER), "Kwitansi"))
                            .min_size(egui::vec2(0.0, 28.0)),
                    )
                    .clicked()
                    && let Some(t) = &terpilih
                {
                    cetak_kwitansi(app, t);
                }
                if tema::tombol(ui, ikon::PENCIL_SIMPLE, "Ubah").clicked()
                    && let Some(t) = &terpilih
                {
                    buka_edit(app, t);
                }
            });
        });
    });
    ui.add_space(4.0);

    tema::kartu(ui, |ui| {
        if daftar.is_empty() {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.label(ikon::rt(ikon::RECEIPT).size(36.0).color(p.teks2));
                tema::label_redup(ui, "Tidak ada transaksi pada periode ini.");
            });
            ui.add_space(30.0);
            return;
        }
        if ponsel {
            for t in &daftar {
                if super::ringkasan::baris_transaksi(app, ui, t).clicked() {
                    app.ui_transaksi.terpilih = Some(t.id);
                    buka_edit(app, t);
                }
            }
            return;
        }
        let tinggi = (ui.ctx().content_rect().height() - 330.0).max(240.0);
        TableBuilder::new(ui)
            .id_salt("tabel-transaksi")
            .striped(true)
            .sense(egui::Sense::click())
            .max_scroll_height(tinggi)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(82.0))
            .column(Column::exact(118.0))
            .column(Column::remainder().at_least(180.0).clip(true))
            .column(Column::exact(120.0).clip(true))
            .column(Column::exact(118.0))
            .column(Column::exact(118.0))
            .header(26.0, |mut h| {
                for (judul, kanan) in [
                    ("Tanggal", false),
                    ("No. Bukti", false),
                    ("Uraian", false),
                    ("Akun", false),
                    ("Penerimaan", true),
                    ("Pengeluaran", true),
                ] {
                    h.col(|ui| {
                        let teks = RichText::new(judul).strong().color(p.teks2);
                        if kanan {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| ui.label(teks),
                            );
                        } else {
                            ui.label(teks);
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(40.0, daftar.len(), |mut row| {
                    let t = &daftar[row.index()];
                    row.set_selected(app.ui_transaksi.terpilih == Some(t.id));
                    row.col(|ui| {
                        ui.label(tanggal_pendek(t.tanggal));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(&t.no_bukti).monospace().size(12.0));
                    });
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.add(egui::Label::new(&t.uraian).truncate());
                            let mut info = app
                                .kategori(t.kategori_id)
                                .map(|k| k.nama.clone())
                                .unwrap_or_else(|| t.jenis.label().into());
                            if !t.pihak.is_empty() {
                                info += &format!(" · {}", t.pihak);
                            }
                            if t.pungutan.total() > 0 {
                                info += &format!(" · pajak {}", rupiah(t.pungutan.total()));
                            }
                            ui.add(
                                egui::Label::new(RichText::new(info).size(11.0).color(p.teks2))
                                    .truncate(),
                            );
                        });
                    });
                    row.col(|ui| {
                        let akun = match t.jenis {
                            JenisTransaksi::Transfer => format!(
                                "{} → {}",
                                app.nama_akun(t.akun_id),
                                app.nama_akun(t.akun_tujuan_id.unwrap_or(0))
                            ),
                            _ => app.nama_akun(t.akun_id).to_string(),
                        };
                        ui.add(egui::Label::new(akun).truncate());
                    });
                    let kanan = |ui: &mut egui::Ui, n: i64, warna| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if n != 0 {
                                ui.label(RichText::new(ribuan(n)).color(warna));
                            }
                        });
                    };
                    let (m, k) = match t.jenis {
                        JenisTransaksi::Masuk => (t.jumlah, 0),
                        JenisTransaksi::Keluar => (0, t.jumlah),
                        JenisTransaksi::Transfer => (t.jumlah, t.jumlah),
                    };
                    row.col(|ui| kanan(ui, m, p.hijau));
                    row.col(|ui| kanan(ui, k, p.merah));
                    let r = row.response();
                    if r.clicked() {
                        app.ui_transaksi.terpilih = Some(t.id);
                    }
                    if r.double_clicked() {
                        app.ui_transaksi.terpilih = Some(t.id);
                        buka_edit(app, t);
                    }
                });
            });
    });
}

/// Modal formulir transaksi dan konfirmasi hapus (digambar di atas halaman apa pun).
pub fn formulir(app: &mut MyDigitApp, ctx: &egui::Context) {
    if let Some(id) = app.ui_transaksi.hapus {
        let mut selesai = false;
        let t = app.transaksi.iter().find(|t| t.id == id).cloned();
        egui::Modal::new(egui::Id::unique("hapus-transaksi")).show(ctx, |ui| {
            ui.set_width(tema::lebar_modal(ui, 360.0));
            tema::subjudul(ui, "Hapus transaksi?");
            if let Some(t) = &t {
                ui.label(format!(
                    "{} — {} ({})",
                    t.no_bukti,
                    t.uraian,
                    rupiah(t.jumlah)
                ));
            }
            tema::label_redup(ui, "Tindakan ini tidak dapat dibatalkan.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if tema::tombol_bahaya(ui, "", "Hapus").clicked() {
                    app.ubah(ctx, "Transaksi dihapus.", |db| db.hapus_transaksi(id));
                    app.ui_transaksi.terpilih = None;
                    selesai = true;
                }
                if tema::tombol(ui, "", "Batal").clicked() {
                    selesai = true;
                }
            });
        });
        if selesai {
            app.ui_transaksi.hapus = None;
        }
    }

    let Some(mut f) = app.ui_transaksi.form.take() else {
        return;
    };
    let mut tutup = false;
    let mut simpan = false;
    let mut minta_hapus = false;
    let mut minta_kwitansi = false;
    let instansi = app.instansi.clone();
    let pajak_tampil = app.pajak_tampil();
    let p = egui::Modal::new(egui::Id::unique("form-transaksi"))
        .show(ctx, |ui| {
            let p = palet(ui);
            ui.set_width(tema::lebar_modal(ui, 520.0));
            tema::baris_kepala(ui, |ui| {
                tema::subjudul(ui, if f.id == 0 { "Transaksi Baru" } else { "Ubah Transaksi" });
                tema::kanan(ui, |ui| {
                    let mut j = f.jenis;
                    tema::segmen(
                        ui,
                        &mut j,
                        &[
                            (JenisTransaksi::Masuk, "Pemasukan"),
                            (JenisTransaksi::Keluar, "Pengeluaran"),
                            (JenisTransaksi::Transfer, "Transfer"),
                        ],
                    );
                    if j != f.jenis {
                        f.jenis = j;
                        f.kategori_id = None;
                    }
                });
            });
            ui.add_space(6.0);

            tema::baris_form(ui, "Tanggal", |ui| {
                tema::pilih_tanggal(ui, "tgl-transaksi", &mut f.tanggal);
            });
            // Perbarui nomor bukti otomatis bila jenis atau tahun berubah.
            if f.id == 0 && f.no_bukti == f.bukti_otomatis {
                let baru = app
                    .db()
                    .nomor_bukti_berikutnya(instansi.awalan(f.jenis), f.tanggal.year())
                    .unwrap_or_default();
                if baru != f.bukti_otomatis {
                    f.no_bukti = baru.clone();
                    f.bukti_otomatis = baru;
                }
            }
            let contoh_bukti = format!("{}-{}-0001", instansi.awalan(f.jenis), f.tanggal.year());
            tema::baris_form(ui, "No. bukti", |ui| tema::isian(ui, &mut f.no_bukti, &contoh_bukti));

            let label_akun = match f.jenis {
                JenisTransaksi::Masuk => "Masuk ke akun",
                JenisTransaksi::Keluar => "Dibayar dari akun",
                JenisTransaksi::Transfer => "Dari akun",
            };
            tema::baris_form(ui, label_akun, |ui| pilih_akun(ui, "akun", &app.akun, &mut f.akun_id));
            if f.jenis == JenisTransaksi::Transfer {
                tema::baris_form(ui, "Ke akun", |ui| pilih_akun(ui, "tujuan", &app.akun, &mut f.tujuan_id));
            } else {
                let arus = if f.jenis == JenisTransaksi::Masuk { Arus::Masuk } else { Arus::Keluar };
                tema::baris_form(ui, "Kategori", |ui| {
                    let teks = app
                        .kategori(f.kategori_id)
                        .map(|k| format!("{} {}", k.kode, k.nama))
                        .unwrap_or_else(|| "Pilih kategori…".into());
                    egui::ComboBox::from_id_salt("kategori").icon(tema::ikon_dropdown).selected_text(teks).width(ui.available_width().min(340.0)).show_ui(ui, |ui| {
                        for k in app.kategori.iter().filter(|k| k.arus == arus) {
                            ui.selectable_value(&mut f.kategori_id, Some(k.id), format!("{}  {}", k.kode, k.nama));
                        }
                    });
                });
            }
            tema::baris_form(ui, "Uraian", |ui| tema::isian(ui, &mut f.uraian, "mis. Pembelian kertas HVS untuk ujian"));
            if f.jenis != JenisTransaksi::Transfer {
                let label = if f.jenis == JenisTransaksi::Masuk { "Diterima dari" } else { "Dibayarkan kepada" };
                tema::baris_form(ui, label, |ui| tema::isian(ui, &mut f.pihak, "Nama orang / toko / instansi"));
                tema::baris_form(ui, "Sumber dana", |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let petunjuk = instansi.sumber_dana.first().map(|s| format!("mis. {s}")).unwrap_or_default();
                        ui.add(egui::TextEdit::singleline(&mut f.sumber_dana).hint_text(petunjuk).desired_width(ui.available_width().min(200.0)));
                        for s in &instansi.sumber_dana {
                            if ui.small_button(s).clicked() {
                                f.sumber_dana = s.clone();
                            }
                        }
                    });
                });
            }
            tema::baris_form(ui, "Jumlah (Rp)", |ui| tema::isian_rupiah(ui, &mut f.jumlah, 200.0));

            let ada_pajak = f.setor.is_some() || f.pajak.iter().any(|s| !s.is_empty());
            if f.jenis == JenisTransaksi::Keluar && (pajak_tampil || ada_pajak) {
                ui.add_space(4.0);
                egui::CollapsingHeader::new("Pajak")
                    .icon(tema::ikon_lipat)
                    .default_open(f.setor.is_some() || f.pajak.iter().any(|s| !s.is_empty()))
                    .show(ui, |ui| {
                        let mut setor = f.setor.is_some();
                        ui.checkbox(&mut setor, "Transaksi ini adalah setoran pajak ke kas negara");
                        if setor {
                            let mut jenis = f.setor.unwrap_or(JenisPajak::Ppn);
                            tema::baris_form(ui, "Jenis pajak disetor", |ui| {
                                egui::ComboBox::from_id_salt("setor").icon(tema::ikon_dropdown).selected_text(jenis.label()).show_ui(ui, |ui| {
                                    for pj in JenisPajak::SEMUA {
                                        ui.selectable_value(&mut jenis, pj, pj.label());
                                    }
                                });
                            });
                            f.setor = Some(jenis);
                        } else {
                            f.setor = None;
                            tema::label_redup(ui, "Pajak yang dipungut dari pembayaran ini (dicatat di Buku Pembantu Pajak):");
                            egui::Grid::new("pajak").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
                                for (i, pj) in JenisPajak::SEMUA.iter().enumerate() {
                                    ui.label(RichText::new(pj.label()).color(p.teks2));
                                    tema::isian_rupiah(ui, &mut f.pajak[i], 130.0);
                                    if i % 2 == 1 {
                                        ui.end_row();
                                    }
                                }
                            });
                        }
                    });
            }

            if let Some(g) = &f.galat {
                ui.label(RichText::new(g).color(p.merah));
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if f.id != 0 {
                    if tema::tombol_ikon(ui, ikon::TRASH, "Hapus transaksi", p.merah).clicked() {
                        minta_hapus = true;
                    }
                    if f.jenis != JenisTransaksi::Transfer
                        && tema::tombol_ikon(ui, ikon::PRINTER, "Cetak kwitansi", p.aksen).clicked()
                    {
                        minta_kwitansi = true;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if tema::tombol_utama(ui, "", "Simpan").clicked() {
                        simpan = true;
                    }
                    if tema::tombol(ui, "", "Batal").clicked() {
                        tutup = true;
                    }
                });
            });
            if ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command) {
                simpan = true;
            }
            p
        });
    if minta_hapus {
        app.ui_transaksi.hapus = Some(f.id);
        tutup = true;
    }
    if minta_kwitansi && let Some(t) = app.transaksi.iter().find(|t| t.id == f.id).cloned() {
        cetak_kwitansi(app, &t);
    }
    if p.should_close() {
        tutup = true;
    }

    if simpan {
        match validasi(&f, app) {
            Ok(t) => {
                let pesan = if f.id == 0 {
                    "Transaksi tersimpan."
                } else {
                    "Perubahan tersimpan."
                };
                let mut id = 0;
                if app.ubah(ctx, pesan, |db| db.simpan_transaksi(&t).map(|i| id = i)) {
                    app.ui_transaksi.terpilih = Some(id);
                    return;
                }
            }
            Err(e) => f.galat = Some(e),
        }
    }
    if !tutup {
        app.ui_transaksi.form = Some(f);
    }
}

fn pilih_akun(ui: &mut egui::Ui, id: &str, akun: &[Akun], terpilih: &mut i64) {
    let teks = akun
        .iter()
        .find(|a| a.id == *terpilih)
        .map(|a| a.nama.clone())
        .unwrap_or_else(|| "Pilih akun…".into());
    egui::ComboBox::from_id_salt(id)
        .icon(tema::ikon_dropdown)
        .selected_text(teks)
        .width(ui.available_width().min(340.0))
        .show_ui(ui, |ui| {
            for a in akun {
                ui.selectable_value(
                    terpilih,
                    a.id,
                    format!("{}  ·  {}", a.nama, a.jenis.label()),
                );
            }
        });
}

fn validasi(f: &Form, app: &MyDigitApp) -> Result<Transaksi, String> {
    let jumlah = parse_rupiah(&f.jumlah).ok_or("Jumlah harus berupa angka.")?;
    if jumlah <= 0 {
        return Err("Jumlah harus lebih dari nol.".into());
    }
    if f.uraian.trim().is_empty() {
        return Err("Uraian wajib diisi.".into());
    }
    if f.no_bukti.trim().is_empty() {
        return Err("Nomor bukti wajib diisi.".into());
    }
    if !app.akun.iter().any(|a| a.id == f.akun_id) {
        return Err("Pilih akun kas atau bank.".into());
    }
    let mut pungutan = Pungutan::default();
    if f.jenis == JenisTransaksi::Keluar && f.setor.is_none() {
        for (i, p) in JenisPajak::SEMUA.iter().enumerate() {
            *pungutan.get_mut(*p) =
                parse_rupiah(&f.pajak[i]).ok_or("Nominal pajak harus berupa angka.")?;
        }
        if pungutan.total() > jumlah {
            return Err("Total pajak tidak boleh melebihi jumlah pembayaran.".into());
        }
    }
    let akun_tujuan_id = if f.jenis == JenisTransaksi::Transfer {
        if f.tujuan_id == f.akun_id || !app.akun.iter().any(|a| a.id == f.tujuan_id) {
            return Err("Pilih akun tujuan yang berbeda.".into());
        }
        Some(f.tujuan_id)
    } else {
        None
    };
    let duplikat = app
        .transaksi
        .iter()
        .any(|t| t.id != f.id && t.no_bukti == f.no_bukti.trim());
    if duplikat {
        return Err(format!("Nomor bukti {} sudah dipakai.", f.no_bukti.trim()));
    }
    Ok(Transaksi {
        id: f.id,
        tanggal: f.tanggal,
        no_bukti: f.no_bukti.trim().into(),
        jenis: f.jenis,
        akun_id: f.akun_id,
        akun_tujuan_id,
        kategori_id: if f.jenis == JenisTransaksi::Transfer {
            None
        } else {
            f.kategori_id
        },
        uraian: f.uraian.trim().into(),
        jumlah,
        pihak: f.pihak.trim().into(),
        sumber_dana: f.sumber_dana.trim().into(),
        pungutan,
        setor_pajak: if f.jenis == JenisTransaksi::Keluar {
            f.setor
        } else {
            None
        },
    })
}
