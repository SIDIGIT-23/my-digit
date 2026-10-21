//! Jenis pengguna ("Masuk sebagai") dan templat bawaannya.
//!
//! Empat jenis tingkat atas: Sekolah, Organisasi, Instansi, Lembaga. Jenis
//! menentukan bawaan (kategori, jabatan, label nomor, istilah anggaran,
//! sumber dana, fitur pajak) saat penyiapan atau saat pengguna mengubah jenis;
//! data yang sudah ada tidak pernah dihapus karenanya.

use crate::model::Arus;
#[cfg(test)]
use crate::model::{PengaturanInstansi, ProfilLembaga};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JenisInstansi {
    Sekolah,
    Organisasi,
    Instansi,
    Lembaga,
}

impl JenisInstansi {
    pub const SEMUA: [Self; 4] = [
        Self::Sekolah,
        Self::Organisasi,
        Self::Instansi,
        Self::Lembaga,
    ];

    pub fn kode(self) -> &'static str {
        match self {
            Self::Sekolah => "sekolah",
            Self::Organisasi => "organisasi",
            Self::Instansi => "instansi",
            Self::Lembaga => "lembaga",
        }
    }

    /// Kode tak dikenal dianggap Sekolah (jenis aplikasi sebelum v0.3).
    pub fn dari_kode(s: &str) -> Self {
        Self::SEMUA
            .into_iter()
            .find(|j| j.kode() == s)
            .unwrap_or(Self::Sekolah)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sekolah => "Sekolah",
            Self::Organisasi => "Organisasi",
            Self::Instansi => "Instansi",
            Self::Lembaga => "Lembaga",
        }
    }

    /// Kata dalam kalimat, mis. "Nama sekolah".
    pub fn kata(self) -> &'static str {
        match self {
            Self::Sekolah => "sekolah",
            Self::Organisasi => "organisasi",
            Self::Instansi => "instansi",
            Self::Lembaga => "lembaga",
        }
    }

    /// Kode glyph Phosphor (lihat `ui::ikon`).
    pub fn ikon(self) -> &'static str {
        match self {
            Self::Sekolah => "\u{E62C}",    // graduation-cap
            Self::Organisasi => "\u{E68E}", // users-three
            Self::Instansi => "\u{E0FF}",   // building-office
            Self::Lembaga => "\u{E810}",    // hand-heart
        }
    }

    pub fn keterangan(self) -> &'static str {
        match self {
            Self::Sekolah => {
                "SD, SMP, SMA/SMK, madrasah — negeri atau swasta. Dana BOS, komite, RKAS."
            }
            Self::Organisasi => {
                "OSIS, karang taruna, komunitas, ormas, paguyuban. Iuran anggota dan kegiatan."
            }
            Self::Instansi => {
                "Kantor desa/kelurahan, dinas, puskesmas, UPT. APBD/APBN, DPA, pajak."
            }
            Self::Lembaga => {
                "Yayasan, masjid/musala, LSM, panti asuhan, TPQ. Donasi, infak, hibah."
            }
        }
    }

    pub fn templat(self) -> &'static Templat {
        match self {
            Self::Sekolah => &SEKOLAH,
            Self::Organisasi => &ORGANISASI,
            Self::Instansi => &INSTANSI,
            Self::Lembaga => &LEMBAGA,
        }
    }
}

/// Bawaan satu jenis. Daftar `pilihan_*` dipakai sebagai tombol saran.
pub struct Templat {
    pub contoh_nama: &'static str,
    pub contoh_induk: &'static str,
    pub subjenis: &'static [&'static str],
    pub pimpinan: &'static str,
    pub pilihan_pimpinan: &'static [&'static str],
    pub bendahara: &'static str,
    pub pilihan_bendahara: &'static [&'static str],
    pub label_nomor: &'static str,
    pub pilihan_label_nomor: &'static [&'static str],
    pub id1: &'static str,
    pub id2: &'static str,
    pub pilihan_id: &'static [&'static str],
    pub istilah: &'static str,
    pub pilihan_istilah: &'static [&'static str],
    pub sumber_dana: &'static [&'static str],
    pub pajak: bool,
    pub kategori: &'static [(&'static str, &'static str, Arus)],
}

impl Templat {
    /// Isi field profil/instansi dari templat (data uji dan demo).
    /// Nama, nomor, dan identitas tidak disentuh.
    #[cfg(test)]
    pub fn isi(&self, p: &mut ProfilLembaga, i: &mut PengaturanInstansi) {
        p.jabatan_pimpinan = self.pimpinan.into();
        p.jabatan_bendahara = self.bendahara.into();
        p.label_nomor_pimpinan = self.label_nomor.into();
        p.label_nomor_bendahara = self.label_nomor.into();
        p.id1_label = self.id1.into();
        p.id2_label = self.id2.into();
        i.pajak = self.pajak;
        i.istilah_anggaran = self.istilah.into();
        i.sumber_dana = self.sumber_dana.iter().map(|s| s.to_string()).collect();
    }
}

/// Kategori bawaan Sekolah — sama persis dengan kategori awal My Digit v0.2.
pub const KATEGORI_SEKOLAH: &[(&str, &str, Arus)] = &[
    ("4.1", "Dana BOS", Arus::Masuk),
    ("4.2", "Iuran / Dana Komite", Arus::Masuk),
    ("4.3", "Donasi & Sumbangan", Arus::Masuk),
    ("4.4", "Jasa Giro / Bunga Bank", Arus::Masuk),
    ("4.9", "Penerimaan Lain-lain", Arus::Masuk),
    ("5.1", "Honor & Belanja Pegawai", Arus::Keluar),
    ("5.2", "Alat Tulis & Bahan Habis Pakai", Arus::Keluar),
    ("5.3", "Kegiatan Pembelajaran", Arus::Keluar),
    ("5.4", "Pemeliharaan Sarana Prasarana", Arus::Keluar),
    ("5.5", "Langganan Daya & Jasa", Arus::Keluar),
    ("5.6", "Belanja Modal / Peralatan", Arus::Keluar),
    ("5.7", "Biaya Administrasi Bank", Arus::Keluar),
    ("5.8", "Setoran Pajak", Arus::Keluar),
    ("5.9", "Pengeluaran Lain-lain", Arus::Keluar),
];

const KATEGORI_ORGANISASI: &[(&str, &str, Arus)] = &[
    ("4.1", "Iuran Anggota", Arus::Masuk),
    ("4.2", "Iuran / Kontribusi Kegiatan", Arus::Masuk),
    ("4.3", "Donasi & Sponsor", Arus::Masuk),
    ("4.4", "Hibah / Bantuan", Arus::Masuk),
    ("4.5", "Hasil Usaha & Dana Usaha", Arus::Masuk),
    ("4.6", "Jasa Giro / Bunga Bank", Arus::Masuk),
    ("4.9", "Penerimaan Lain-lain", Arus::Masuk),
    ("5.1", "Program Kerja & Kegiatan", Arus::Keluar),
    ("5.2", "Konsumsi Rapat & Pertemuan", Arus::Keluar),
    ("5.3", "Dana Sosial & Santunan", Arus::Keluar),
    ("5.4", "ATK, Cetak & Administrasi", Arus::Keluar),
    ("5.5", "Transportasi & Akomodasi", Arus::Keluar),
    ("5.6", "Perlengkapan & Inventaris", Arus::Keluar),
    ("5.7", "Honor / Uang Saku Petugas", Arus::Keluar),
    ("5.8", "Biaya Administrasi Bank", Arus::Keluar),
    ("5.9", "Pengeluaran Lain-lain", Arus::Keluar),
];

const KATEGORI_INSTANSI: &[(&str, &str, Arus)] = &[
    ("4.1", "Uang Persediaan / Ganti Uang (UP/GU)", Arus::Masuk),
    ("4.2", "Tambahan Uang (TU)", Arus::Masuk),
    ("4.3", "Pembayaran Langsung (LS)", Arus::Masuk),
    ("4.4", "Dana Transfer (Dana Desa, ADD, DAK)", Arus::Masuk),
    (
        "4.5",
        "Pendapatan Asli / Retribusi / Jasa Layanan",
        Arus::Masuk,
    ),
    ("4.6", "Hibah / Bantuan Keuangan", Arus::Masuk),
    ("4.7", "Jasa Giro / Bunga Bank", Arus::Masuk),
    ("4.9", "Penerimaan Lain-lain", Arus::Masuk),
    ("5.1", "Belanja Pegawai / Honorarium", Arus::Keluar),
    ("5.2", "Belanja Barang Pakai Habis & ATK", Arus::Keluar),
    (
        "5.3",
        "Belanja Jasa (Listrik, Air, Telepon, Internet)",
        Arus::Keluar,
    ),
    ("5.4", "Belanja Perjalanan Dinas", Arus::Keluar),
    ("5.5", "Belanja Pemeliharaan", Arus::Keluar),
    ("5.6", "Belanja Modal", Arus::Keluar),
    ("5.7", "Belanja Hibah / Bantuan Sosial", Arus::Keluar),
    ("5.8", "Setoran Pajak", Arus::Keluar),
    (
        "5.9",
        "Setoran Sisa Kas / Pengeluaran Lain-lain",
        Arus::Keluar,
    ),
];

const KATEGORI_LEMBAGA: &[(&str, &str, Arus)] = &[
    ("4.1", "Donatur Tetap", Arus::Masuk),
    ("4.2", "Infak & Sedekah", Arus::Masuk),
    ("4.3", "Donasi Tidak Tetap / Kotak Amal", Arus::Masuk),
    ("4.4", "Zakat, Fidyah & Dana Titipan", Arus::Masuk),
    ("4.5", "Hibah / Bantuan Pemerintah", Arus::Masuk),
    ("4.6", "Hibah / CSR Lembaga Lain", Arus::Masuk),
    ("4.7", "Hasil Usaha & Sewa Fasilitas", Arus::Masuk),
    ("4.8", "Bagi Hasil / Jasa Giro Bank", Arus::Masuk),
    ("4.9", "Penerimaan Lain-lain", Arus::Masuk),
    ("5.1", "Honor Pengurus, Pengajar & Petugas", Arus::Keluar),
    ("5.2", "Program & Kegiatan", Arus::Keluar),
    ("5.3", "Santunan & Penyaluran Bantuan", Arus::Keluar),
    ("5.4", "Penyaluran Zakat & Dana Titipan", Arus::Keluar),
    ("5.5", "Operasional Kantor & ATK", Arus::Keluar),
    ("5.6", "Listrik, Air & Langganan", Arus::Keluar),
    ("5.7", "Pemeliharaan & Pembangunan", Arus::Keluar),
    ("5.8", "Biaya Administrasi Bank", Arus::Keluar),
    ("5.9", "Pengeluaran Lain-lain", Arus::Keluar),
];

static SEKOLAH: Templat = Templat {
    contoh_nama: "mis. SMP Negeri 23 Dumai",
    contoh_induk: "PEMERINTAH KOTA DUMAI\nDINAS PENDIDIKAN",
    subjenis: &[
        "SD Negeri",
        "SMP Negeri",
        "SMA Negeri",
        "SMK Negeri",
        "Sekolah Swasta",
        "Madrasah",
    ],
    pimpinan: "Kepala Sekolah",
    pilihan_pimpinan: &[
        "Kepala Sekolah",
        "Kepala Madrasah",
        "Plt. Kepala Sekolah",
        "Ketua Yayasan",
    ],
    bendahara: "Bendahara",
    pilihan_bendahara: &["Bendahara", "Bendahara BOS", "Bendahara Komite"],
    label_nomor: "NIP",
    pilihan_label_nomor: &["NIP", "NIY", "NUPTK"],
    id1: "NPSN",
    id2: "",
    pilihan_id: &["NPSN", "NSM", "NPWP"],
    istilah: "RKAS",
    pilihan_istilah: &["RKAS", "RKAM", "RAPBS"],
    sumber_dana: &["BOS", "Komite", "Swadaya"],
    pajak: true,
    kategori: KATEGORI_SEKOLAH,
};

static ORGANISASI: Templat = Templat {
    contoh_nama: "mis. OSIS SMP Negeri 23 Dumai",
    contoh_induk: "SMP NEGERI 23 DUMAI",
    subjenis: &[
        "OSIS",
        "Karang Taruna",
        "Komunitas",
        "Ormas",
        "Paguyuban",
        "RT/RW",
    ],
    pimpinan: "Ketua",
    pilihan_pimpinan: &[
        "Ketua",
        "Ketua Umum",
        "Ketua OSIS",
        "Pembina",
        "Ketua RT",
        "Ketua RW",
    ],
    bendahara: "Bendahara",
    pilihan_bendahara: &["Bendahara", "Bendahara Umum", "Bendahara OSIS"],
    label_nomor: "",
    pilihan_label_nomor: &["NIP", "NIY", "NRA"],
    id1: "No. SK",
    id2: "",
    pilihan_id: &["No. SK", "No. SK Pengurus", "No. SKT", "No. AHU", "NPWP"],
    istilah: "Rencana Anggaran",
    pilihan_istilah: &["Rencana Anggaran", "RAB", "Program Kerja"],
    sumber_dana: &["Kas", "Iuran", "Donasi", "Sponsor"],
    pajak: false,
    kategori: KATEGORI_ORGANISASI,
};

static INSTANSI: Templat = Templat {
    contoh_nama: "mis. Kantor Kelurahan Bukit Datuk",
    contoh_induk: "PEMERINTAH KOTA DUMAI\nKECAMATAN DUMAI TIMUR",
    subjenis: &[
        "Kantor Desa",
        "Kelurahan",
        "Dinas/Badan",
        "Puskesmas",
        "UPT",
    ],
    pimpinan: "Kepala Kantor",
    pilihan_pimpinan: &[
        "Kepala Kantor",
        "Kepala Dinas",
        "Kepala Desa",
        "Lurah",
        "Camat",
        "Kepala Puskesmas",
        "Kepala UPT",
        "Pengguna Anggaran",
        "Kuasa Pengguna Anggaran",
    ],
    bendahara: "Bendahara Pengeluaran",
    pilihan_bendahara: &[
        "Bendahara Pengeluaran",
        "Bendahara Penerimaan",
        "Kaur Keuangan",
        "Bendahara",
    ],
    label_nomor: "NIP",
    pilihan_label_nomor: &["NIP", "NRP"],
    id1: "Kode Satker",
    id2: "NPWP",
    pilihan_id: &[
        "Kode Satker",
        "Kode SKPD",
        "Kode Desa",
        "Kode Puskesmas",
        "NPWP",
    ],
    istilah: "DPA",
    pilihan_istilah: &["DPA", "DIPA", "APBDes", "RBA"],
    sumber_dana: &["APBD", "APBN", "Dana Desa", "BLUD"],
    pajak: true,
    kategori: KATEGORI_INSTANSI,
};

static LEMBAGA: Templat = Templat {
    contoh_nama: "mis. Masjid Al-Ikhlas",
    contoh_induk: "YAYASAN …",
    subjenis: &["Yayasan", "Masjid/Musala", "LSM", "Panti Asuhan", "TPQ"],
    pimpinan: "Ketua",
    pilihan_pimpinan: &[
        "Ketua",
        "Ketua Yayasan",
        "Ketua Pengurus",
        "Ketua DKM",
        "Ketua Takmir",
        "Direktur",
        "Kepala TPQ",
        "Pimpinan Panti",
    ],
    bendahara: "Bendahara",
    pilihan_bendahara: &["Bendahara", "Bendahara Yayasan", "Bendahara Masjid"],
    label_nomor: "",
    pilihan_label_nomor: &["NIY"],
    id1: "No. SK Kemenkumham",
    id2: "NPWP",
    pilihan_id: &[
        "No. SK Kemenkumham",
        "No. Akta Pendirian",
        "ID Masjid",
        "No. Izin Operasional",
        "NPWP",
    ],
    istilah: "Rencana Anggaran",
    pilihan_istilah: &["Rencana Anggaran", "RKAT", "RAB"],
    sumber_dana: &["Donatur", "Infak", "Hibah", "Zakat"],
    pajak: false,
    kategori: KATEGORI_LEMBAGA,
};

/// Peringatan non-blokir untuk nomor identitas yang panjangnya tidak lazim:
/// NPSN 8 digit, NSM 12 digit, NPWP 15/16 digit (titik, strip, dan spasi diabaikan).
pub fn periksa_nomor(label: &str, nomor: &str) -> Option<&'static str> {
    let angka: String = nomor
        .chars()
        .filter(|c| !matches!(c, '.' | '-' | ' '))
        .collect();
    if angka.is_empty() {
        return None;
    }
    let label = label.to_uppercase();
    let semua_angka = angka.chars().all(|c| c.is_ascii_digit());
    if label.contains("NPSN") && (!semua_angka || angka.len() != 8) {
        Some("NPSN biasanya 8 digit angka.")
    } else if label.contains("NSM") && (!semua_angka || angka.len() != 12) {
        Some("NSM biasanya 12 digit angka.")
    } else if label.contains("NPWP") && (!semua_angka || !matches!(angka.len(), 15 | 16)) {
        Some("NPWP biasanya 15 atau 16 digit angka.")
    } else {
        None
    }
}

/// Kunci urut kode kategori secara angka: `"4.10"` → `[(4, ""), (10, "")]`;
/// segmen bukan angka → `(u32::MAX, teks)`.
pub fn kunci_kode(kode: &str) -> Vec<(u32, String)> {
    kode.split('.')
        .map(|s| {
            let s = s.trim();
            match s.parse::<u32>() {
                Ok(n) => (n, String::new()),
                Err(_) => (u32::MAX, s.to_string()),
            }
        })
        .collect()
}

/// Awalan nomor bukti: 1–8 karakter A–Z atau 0–9.
pub fn awalan_valid(s: &str) -> bool {
    (1..=8).contains(&s.len())
        && s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templat_valid() {
        for j in JenisInstansi::SEMUA {
            let k = j.templat().kategori;
            let mut kode: Vec<&str> = k.iter().map(|(kode, _, _)| *kode).collect();
            kode.sort();
            kode.dedup();
            assert_eq!(kode.len(), k.len(), "{}: kode unik", j.label());
            for (kode, _, arus) in k {
                let (a, b) = kode.split_once('.').unwrap();
                let b: u32 = b.parse().unwrap();
                assert!((1..=9).contains(&b), "{kode}");
                assert_eq!(
                    a == "4",
                    *arus == Arus::Masuk,
                    "{kode}: prefiks 4 ⇔ penerimaan"
                );
                assert!(a == "4" || a == "5", "{kode}");
            }
            assert_eq!(JenisInstansi::dari_kode(j.kode()), j);
        }
        // Kategori Sekolah harus identik dengan kategori awal v0.2.
        let v02: &[(&str, &str)] = &[
            ("4.1", "Dana BOS"),
            ("4.2", "Iuran / Dana Komite"),
            ("4.3", "Donasi & Sumbangan"),
            ("4.4", "Jasa Giro / Bunga Bank"),
            ("4.9", "Penerimaan Lain-lain"),
            ("5.1", "Honor & Belanja Pegawai"),
            ("5.2", "Alat Tulis & Bahan Habis Pakai"),
            ("5.3", "Kegiatan Pembelajaran"),
            ("5.4", "Pemeliharaan Sarana Prasarana"),
            ("5.5", "Langganan Daya & Jasa"),
            ("5.6", "Belanja Modal / Peralatan"),
            ("5.7", "Biaya Administrasi Bank"),
            ("5.8", "Setoran Pajak"),
            ("5.9", "Pengeluaran Lain-lain"),
        ];
        let sekarang: Vec<(&str, &str)> =
            KATEGORI_SEKOLAH.iter().map(|(k, n, _)| (*k, *n)).collect();
        assert_eq!(sekarang, v02);
        assert_eq!(
            JenisInstansi::dari_kode("tidak-dikenal"),
            JenisInstansi::Sekolah
        );
    }

    #[test]
    fn urutan_kode_numerik() {
        let mut kode = vec!["4.1", "4.10", "4.2", "5.2.1", "5.2"];
        kode.sort_by_key(|k| kunci_kode(k));
        assert_eq!(kode, ["4.1", "4.2", "4.10", "5.2", "5.2.1"]);
    }

    #[test]
    fn nomor_dan_awalan() {
        assert_eq!(periksa_nomor("NPSN", "10404123"), None);
        assert!(periksa_nomor("NPSN", "1040412").is_some());
        assert_eq!(periksa_nomor("NPWP", "01.234.567.8-901.000"), None);
        assert!(periksa_nomor("NSM", "12").is_some());
        assert_eq!(periksa_nomor("No. SK", "apa saja"), None);
        assert!(awalan_valid("BKM") && awalan_valid("KM2"));
        assert!(
            !awalan_valid("")
                && !awalan_valid("bk")
                && !awalan_valid("A%")
                && !awalan_valid("TERLALUPANJANG")
        );
    }
}
