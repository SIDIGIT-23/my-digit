//! Layar pertama "Masuk sebagai": pilih jenis (Sekolah, Organisasi,
//! Instansi, Lembaga), lalu isi data inti. Tidak ada yang disimpan sampai
//! "Mulai Gunakan My Digit" ditekan.

use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke, Vec2};

use super::{MyDigitApp, ikon, tema};
use crate::branding;
use crate::instansi::{JenisInstansi, Templat};
use crate::model::{Arus, PengaturanInstansi, ProfilLembaga};
use tema::palet;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Langkah {
    #[default]
    Jenis,
    Data,
}

#[derive(Default)]
pub struct State {
    pub langkah: Langkah,
    jenis: Option<JenisInstansi>,
    profil: ProfilLembaga,
    instansi: PengaturanInstansi,
    galat: Option<String>,
}

impl State {
    /// Mulai dari profil dan pengaturan yang sudah tersimpan, agar isian
    /// lama (alamat, NIP, logo, dll.) tidak tertimpa saat penyiapan selesai.
    pub fn dari(profil: ProfilLembaga, instansi: PengaturanInstansi) -> Self {
        Self {
            profil,
            instansi,
            ..Default::default()
        }
    }

    /// Pilih jenis dan isi bawaan templatnya. Isian yang sudah diubah
    /// pengguna (bukan kosong dan bukan bawaan jenis sebelumnya) dibiarkan.
    /// Sebelum ada pilihan, "bawaan sebelumnya" adalah bawaan v0.2 (= Sekolah).
    pub fn pilih(&mut self, jenis: JenisInstansi) {
        if self.jenis == Some(jenis) {
            return;
        }
        let lama: &Templat = self.jenis.unwrap_or(JenisInstansi::Sekolah).templat();
        let baru = jenis.templat();
        let ganti = |isian: &mut String, bawaan_lama: &str, bawaan_baru: &str| {
            if isian.trim().is_empty() || isian == bawaan_lama {
                *isian = bawaan_baru.to_string();
            }
        };
        let p = &mut self.profil;
        ganti(&mut p.jabatan_pimpinan, lama.pimpinan, baru.pimpinan);
        ganti(&mut p.jabatan_bendahara, lama.bendahara, baru.bendahara);
        ganti(
            &mut p.label_nomor_pimpinan,
            lama.label_nomor,
            baru.label_nomor,
        );
        ganti(
            &mut p.label_nomor_bendahara,
            lama.label_nomor,
            baru.label_nomor,
        );
        if p.id1_nomor.trim().is_empty() {
            ganti(&mut p.id1_label, lama.id1, baru.id1);
        }
        if p.id2_nomor.trim().is_empty() {
            ganti(&mut p.id2_label, lama.id2, baru.id2);
        }
        let i = &mut self.instansi;
        if lama.subjenis.contains(&i.subjenis.as_str()) {
            i.subjenis = String::new();
        }
        i.jenis = jenis;
        i.pajak = baru.pajak;
        i.istilah_anggaran = baru.istilah.into();
        i.sumber_dana = baru.sumber_dana.iter().map(|s| s.to_string()).collect();
        self.jenis = Some(jenis);
    }
}

pub fn tampil(app: &mut MyDigitApp, ui: &mut egui::Ui) {
    let Some(mut st) = app.ui_penyiapan.take() else {
        return;
    };
    let ctx = ui.ctx().clone();
    let p = palet(ui);
    let mut selesai = false;
    let mut pulihkan = false;

    let frame = egui::Frame::new().fill(p.latar).inner_margin(Margin {
        left: 24,
        right: 24,
        top: if cfg!(target_os = "macos") { 40 } else { 20 },
        bottom: 16,
    });
    egui::CentralPanel::default().frame(frame).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.set_max_width(640.0);
                    match st.langkah {
                        Langkah::Jenis => langkah_jenis(app, ui, &mut st, &mut pulihkan),
                        Langkah::Data => selesai = langkah_data(ui, &mut st),
                    }
                });
            });
    });

    if pulihkan {
        app.ui_penyiapan = Some(st);
        app.mulai_pulihkan();
        return;
    }
    if selesai {
        let jenis = st.jenis.unwrap_or(JenisInstansi::Sekolah);
        match app
            .db()
            .selesaikan_penyiapan(jenis, &st.profil, &st.instansi)
        {
            Ok(()) => {
                app.reset_ui();
                app.muat_ulang(&ctx);
                app.halaman = super::Halaman::Ringkasan;
                app.info(format!(
                    "Selamat datang! Data {} siap digunakan.",
                    jenis.kata()
                ));
                return;
            }
            Err(e) => st.galat = Some(e),
        }
    }
    app.ui_penyiapan = Some(st);
}

fn langkah_jenis(app: &MyDigitApp, ui: &mut egui::Ui, st: &mut State, pulihkan: &mut bool) {
    let p = palet(ui);
    ui.add_space(8.0);
    ui.add(egui::Image::new(&app.logo_app).fit_to_exact_size(Vec2::splat(64.0)));
    ui.add_space(8.0);
    ui.label(
        RichText::new("Selamat datang di My Digit")
            .size(26.0)
            .strong(),
    );
    ui.label(RichText::new(branding::APP_TAGLINE).color(p.teks2));
    ui.add_space(18.0);
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.label(RichText::new("Masuk sebagai").size(15.0).strong());
        tema::label_redup(
            ui,
            "Pilihan ini menyiapkan kategori, istilah, dan isian yang sesuai. Bisa diubah nanti.",
        );
        ui.add_space(6.0);
        let mut lanjut = false;
        for baris in JenisInstansi::SEMUA.chunks(2) {
            ui.columns(2, |c| {
                for (ui, jenis) in c.iter_mut().zip(baris) {
                    let r = kartu_jenis(ui, *jenis, st.jenis == Some(*jenis));
                    if r.clicked() {
                        st.pilih(*jenis);
                    }
                    if r.double_clicked() {
                        st.pilih(*jenis);
                        lanjut = true;
                    }
                }
            });
            ui.add_space(4.0);
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.link("Pulihkan dari Cadangan…").clicked() {
                *pulihkan = true;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let bisa = st.jenis.is_some();
                let tombol = ui
                    .add_enabled_ui(bisa, |ui| tema::tombol_utama(ui, "", "Lanjutkan"))
                    .inner;
                let enter = bisa && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if tombol.clicked() || enter {
                    lanjut = true;
                }
            });
        });
        if lanjut && st.jenis.is_some() {
            st.langkah = Langkah::Data;
        }
    });
}

/// Kartu besar satu jenis di layar "Masuk sebagai".
fn kartu_jenis(ui: &mut egui::Ui, jenis: JenisInstansi, aktif: bool) -> egui::Response {
    let p = palet(ui);
    let id = ui.make_persistent_id(("kartu-jenis", jenis.kode()));
    let sorot = ui.ctx().read_response(id).is_some_and(|r| r.hovered());
    let garis = if aktif {
        Stroke::new(1.5, p.aksen)
    } else if sorot {
        Stroke::new(1.0, p.aksen.gamma_multiply(0.5))
    } else {
        Stroke::new(1.0, p.garis)
    };
    let r = egui::Frame::new()
        .fill(if aktif { p.pilih } else { p.kartu })
        .stroke(garis)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(112.0);
            // Kolom egui rata kiri-kanan; teks yang dibungkus jadi meregang
            // bila tidak diberi tata letak rata kiri sendiri.
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.horizontal(|ui| {
                    ui.label(ikon::rt(jenis.ikon()).size(30.0).color(p.aksen));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if aktif {
                            ui.label(ikon::rt(ikon::CHECK_CIRCLE).size(18.0).color(p.aksen));
                        }
                    });
                });
                ui.add_space(2.0);
                ui.label(RichText::new(jenis.label()).size(16.0).strong());
                ui.add(
                    egui::Label::new(RichText::new(jenis.keterangan()).size(12.0).color(p.teks2))
                        .wrap(),
                );
            });
        })
        .response;
    ui.interact(r.rect, id, egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Langkah 2: data inti. Mengembalikan `true` bila "Mulai" ditekan.
fn langkah_data(ui: &mut egui::Ui, st: &mut State) -> bool {
    let p = palet(ui);
    let jenis = st.jenis.unwrap_or(JenisInstansi::Sekolah);
    let t = jenis.templat();
    let mut mulai = false;
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.horizontal(|ui| {
            for i in 0..2 {
                let (r, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 4.0, if i == 1 { p.aksen } else { p.garis });
            }
            tema::label_redup(ui, "Langkah 2 dari 2");
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(ikon::rt(jenis.ikon()).size(26.0).color(p.aksen));
            ui.label(RichText::new(format!("Data {}", jenis.kata())).size(24.0).strong());
        });
        tema::label_redup(ui, "Dipakai untuk kop dan tanda tangan laporan.");
        ui.add_space(10.0);

        tema::kartu(ui, |ui| {
            let (pf, i) = (&mut st.profil, &mut st.instansi);
            tema::baris_form(ui, "Jenis", |ui| tema::isian_saran(ui, &mut i.subjenis, "opsional", 160.0, t.subjenis));
            tema::baris_form(ui, &format!("Nama {} *", jenis.kata()), |ui| {
                tema::isian(ui, &mut pf.nama, t.contoh_nama)
            });
            tema::baris_form(ui, "Kota / kabupaten", |ui| {
                tema::isian(ui, &mut pf.kota, "dipakai pada baris tanggal tanda tangan")
            });
            ui.add_space(4.0);
            tema::baris_form(ui, "Jabatan pimpinan", |ui| {
                tema::isian_saran(ui, &mut pf.jabatan_pimpinan, t.pimpinan, 180.0, t.pilihan_pimpinan)
            });
            tema::baris_form(ui, "Nama pimpinan", |ui| tema::isian(ui, &mut pf.nama_pimpinan, ""));
            tema::baris_form(ui, "Jabatan bendahara", |ui| {
                tema::isian_saran(ui, &mut pf.jabatan_bendahara, t.bendahara, 180.0, t.pilihan_bendahara)
            });
            tema::baris_form(ui, "Nama bendahara", |ui| tema::isian(ui, &mut pf.nama_bendahara, ""));
        });
        ui.add_space(8.0);

        let masuk = t.kategori.iter().filter(|k| k.2 == Arus::Masuk).count();
        let keluar = t.kategori.len() - masuk;
        egui::CollapsingHeader::new(RichText::new("Yang akan disiapkan").strong())
            .icon(tema::ikon_lipat)
            .show(ui, |ui| {
                ui.label(format!("Kategori: {masuk} penerimaan · {keluar} pengeluaran"));
                egui::Grid::new("pratinjau-kategori").num_columns(2).spacing([12.0, 2.0]).show(ui, |ui| {
                    for (kode, nama, _) in t.kategori {
                        ui.label(RichText::new(*kode).monospace().color(p.teks2));
                        ui.label(*nama);
                        ui.end_row();
                    }
                });
                ui.add_space(4.0);
                ui.label(format!("Fitur pajak: {}", if st.instansi.pajak { "Aktif" } else { "Nonaktif" }));
                ui.label(format!("Istilah anggaran: {}", st.instansi.istilah_anggaran));
            });
        ui.add_space(4.0);
        tema::label_redup(
            ui,
            format!(
                "Alamat, logo, nomor identitas, dan nomor induk dapat dilengkapi nanti di Profil {}.",
                jenis.label()
            ),
        );
        if let Some(g) = &st.galat {
            ui.label(RichText::new(g).color(p.merah));
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if tema::tombol(ui, ikon::CARET_LEFT, "Kembali").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                st.langkah = Langkah::Jenis;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let bisa = !st.profil.nama.trim().is_empty();
                let r = ui.add_enabled_ui(bisa, |ui| tema::tombol_utama(ui, "", "Mulai Gunakan My Digit")).inner;
                if r.clicked() {
                    mulai = true;
                }
                if !bisa {
                    tema::label_redup(ui, format!("Isi nama {} dulu.", jenis.kata()));
                }
            });
        });
    });
    mulai
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pilih_tidak_menimpa_isian_pengguna() {
        let lama = ProfilLembaga {
            alamat: "Jl. Lama".into(),
            nip_bendahara: "123".into(),
            logo: Some("logo-lembaga.png".into()),
            jabatan_pimpinan: "Kepala Madrasah".into(),
            jabatan_bendahara: "Bendahara".into(),
            label_nomor_pimpinan: "NIP".into(),
            label_nomor_bendahara: "NIP".into(),
            ..Default::default()
        };
        let mut st = State::dari(lama.clone(), PengaturanInstansi::default());
        st.pilih(JenisInstansi::Lembaga);
        assert_eq!(st.profil.alamat, "Jl. Lama");
        assert_eq!(st.profil.nip_bendahara, "123");
        assert_eq!(st.profil.logo, lama.logo);
        assert_eq!(
            st.profil.jabatan_pimpinan, "Kepala Madrasah",
            "isian pengguna dibiarkan"
        );
        assert_eq!(
            st.profil.label_nomor_pimpinan, "",
            "bawaan v0.2 diganti bawaan Lembaga"
        );
        assert_eq!(st.profil.id1_label, "No. SK Kemenkumham");

        st.instansi.subjenis = "Masjid/Musala".into();
        st.pilih(JenisInstansi::Lembaga);
        assert_eq!(
            st.instansi.subjenis, "Masjid/Musala",
            "memilih ulang jenis yang sama tidak mengubah apa pun"
        );
        st.pilih(JenisInstansi::Instansi);
        assert_eq!(
            st.instansi.subjenis, "",
            "sub-jenis saran jenis lama dikosongkan"
        );
        assert_eq!(st.profil.jabatan_bendahara, "Bendahara Pengeluaran");
    }
}
