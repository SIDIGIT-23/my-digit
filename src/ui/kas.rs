//! Halaman Pemeriksaan Kas (cash opname).

use eframe::egui::{self, RichText};
use egui_extras::DatePickerButton;
use jiff::civil::Date;

use super::{MyDigitApp, tema};
use crate::format::{hari_ini, ribuan, rupiah};
use crate::model::JenisAkun;
use crate::pdf::{self, DataOpname, PECAHAN};
use tema::palet;

pub struct State {
    tanggal: Date,
    akun_id: i64,
    jumlah: [u32; PECAHAN.len()],
    keterangan: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tanggal: hari_ini(),
            akun_id: 0,
            jumlah: [0; PECAHAN.len()],
            keterangan: String::new(),
        }
    }
}

pub fn tampil(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let p = palet(ui);
    tema::judul(
        ui,
        "Pemeriksaan Kas",
        "Hitung uang fisik di brankas, bandingkan dengan saldo buku, lalu cetak berita acaranya.",
    );

    let tunai: Vec<(i64, String)> = app
        .akun
        .iter()
        .filter(|a| a.jenis == JenisAkun::Tunai)
        .map(|a| (a.id, a.nama.clone()))
        .collect();
    if tunai.is_empty() {
        tema::kartu(ui, |ui| {
            tema::label_redup(
                ui,
                "Belum ada akun kas tunai. Tambahkan di menu Akun Kas & Bank.",
            )
        });
        return;
    }
    let st = &mut app.ui_kas;
    if !tunai.iter().any(|t| t.0 == st.akun_id) {
        st.akun_id = tunai[0].0;
    }

    tema::kartu(ui, |ui| {
        tema::baris_form(ui, "Tanggal pemeriksaan", |ui| {
            ui.add(
                DatePickerButton::new(&mut st.tanggal)
                    .id_salt("tgl-opname")
                    .format("%d/%m/%Y"),
            );
        });
        tema::baris_form(ui, "Akun kas", |ui| {
            let nama = tunai
                .iter()
                .find(|t| t.0 == st.akun_id)
                .map(|t| t.1.clone())
                .unwrap_or_default();
            egui::ComboBox::from_id_salt("akun-opname")
                .selected_text(nama)
                .show_ui(ui, |ui| {
                    for (id, n) in &tunai {
                        ui.selectable_value(&mut st.akun_id, *id, n);
                    }
                });
        });
    });
    ui.add_space(12.0);

    let (tanggal, akun_id) = (st.tanggal, st.akun_id);
    let saldo_buku = app
        .sumber()
        .saldo_sebelum(&[akun_id], tanggal.tomorrow().unwrap_or(tanggal));
    let st = &mut app.ui_kas;

    let fisik: i64 = PECAHAN
        .iter()
        .zip(&st.jumlah)
        .map(|(p, n)| p * *n as i64)
        .sum();
    let selisih = fisik - saldo_buku;
    let lebar = ui.available_width();
    let isi_pecahan = |ui: &mut egui::Ui, st: &mut State| {
        tema::kartu(ui, |ui| {
            ui.horizontal(|ui| {
                tema::subjudul(ui, "Rincian Uang Fisik");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Kosongkan").clicked() {
                        st.jumlah = [0; PECAHAN.len()];
                    }
                });
            });
            egui::Grid::new("pecahan")
                .num_columns(3)
                .spacing([18.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    for h in ["Pecahan", "Lembar / keping", "Nilai"] {
                        ui.label(RichText::new(h).strong().color(p.teks2));
                    }
                    ui.end_row();
                    for (i, nilai) in PECAHAN.iter().enumerate() {
                        ui.label(format!(
                            "{} {}",
                            rupiah(*nilai),
                            if *nilai >= 1_000 { "" } else { "(logam)" }
                        ));
                        ui.add(
                            egui::DragValue::new(&mut st.jumlah[i])
                                .range(0..=100_000)
                                .speed(0.2),
                        );
                        ui.label(ribuan(nilai * st.jumlah[i] as i64));
                        ui.end_row();
                    }
                });
        });
    };
    let hasil = |app: &mut MyDigitApp, ui: &mut egui::Ui| {
        tema::kartu(ui, |ui| {
            tema::subjudul(ui, "Hasil Pemeriksaan");
            ui.add_space(4.0);
            for (label, n, warna) in [
                ("Uang fisik (A)", fisik, p.teks),
                ("Saldo buku per tanggal (B)", saldo_buku, p.teks),
                (
                    "Selisih (A − B)",
                    selisih,
                    if selisih == 0 { p.hijau } else { p.merah },
                ),
            ] {
                ui.horizontal(|ui| {
                    tema::label_redup(ui, label);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(rupiah(n)).size(17.0).strong().color(warna));
                    });
                });
            }
            ui.add_space(4.0);
            let status = match selisih {
                0 => "✔  Kas sesuai dengan buku.".to_string(),
                s if s > 0 => format!("Kelebihan kas {}", rupiah(s)),
                s => format!("Kekurangan kas {}", rupiah(-s)),
            };
            ui.label(RichText::new(status).color(if selisih == 0 { p.hijau } else { p.oranye }));
            ui.add_space(8.0);
            tema::label_redup(ui, "Keterangan (opsional)");
            ui.add(
                egui::TextEdit::multiline(&mut app.ui_kas.keterangan)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY)
                    .hint_text("mis. selisih karena uang kembalian belum dicatat"),
            );
            ui.add_space(8.0);
            if tema::tombol_utama(ui, "🖨  Cetak Berita Acara (PDF)").clicked() {
                let st = &app.ui_kas;
                let nama_akun = app.nama_akun(st.akun_id).to_string();
                let data = DataOpname {
                    tanggal: st.tanggal,
                    akun: &nama_akun,
                    jumlah: &st.jumlah,
                    saldo_buku,
                    keterangan: &st.keterangan,
                };
                let bytes = pdf::berita_acara_kas(&app.profil, app.gambar_pdf().as_ref(), &data);
                let nama = format!("Berita Acara Pemeriksaan Kas {}.pdf", st.tanggal);
                app.simpan_berkas(&nama, "pdf", &bytes);
            }
        });
    };
    if lebar > 760.0 {
        ui.columns(2, |c| {
            isi_pecahan(&mut c[0], &mut app.ui_kas);
            hasil(app, &mut c[1]);
        });
    } else {
        isi_pecahan(ui, st);
        ui.add_space(12.0);
        hasil(app, ui);
    }
}
