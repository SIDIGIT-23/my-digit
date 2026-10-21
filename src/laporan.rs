//! Penyusunan laporan: BKU, buku pembantu, rekap, dan realisasi anggaran.
//!
//! Setiap laporan menghasilkan [`Tabel`] yang sama, sehingga tampilan layar,
//! ekspor PDF, dan ekspor CSV memakai satu sumber data.

use std::collections::HashMap;

use jiff::civil::Date;

use crate::format::{ribuan, tanggal_panjang, tanggal_pendek};
use crate::model::*;

#[derive(Debug, Clone)]
pub struct Kolom {
    pub nama: &'static str,
    /// Lebar relatif terhadap kolom lain.
    pub lebar: f32,
    pub kanan: bool,
}

const fn kol(nama: &'static str, lebar: f32, kanan: bool) -> Kolom {
    Kolom { nama, lebar, kanan }
}

#[derive(Debug, Clone)]
pub struct Tabel {
    pub judul: String,
    pub subjudul: String,
    pub kolom: Vec<Kolom>,
    pub baris: Vec<Vec<String>>,
    pub total: Option<Vec<String>>,
    pub mendatar: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JenisLaporan {
    Bku,
    PembantuKas,
    PembantuBank,
    PembantuPajak,
    RekapKategori,
    RealisasiAnggaran,
}

impl JenisLaporan {
    pub const SEMUA: [JenisLaporan; 6] = [
        Self::Bku,
        Self::PembantuKas,
        Self::PembantuBank,
        Self::PembantuPajak,
        Self::RekapKategori,
        Self::RealisasiAnggaran,
    ];

    pub fn judul(self) -> &'static str {
        match self {
            Self::Bku => "Buku Kas Umum",
            Self::PembantuKas => "Buku Pembantu Kas Tunai",
            Self::PembantuBank => "Buku Pembantu Bank",
            Self::PembantuPajak => "Buku Pembantu Pajak",
            Self::RekapKategori => "Rekap per Kategori",
            Self::RealisasiAnggaran => "Realisasi Anggaran",
        }
    }

    pub fn keterangan(self) -> &'static str {
        match self {
            Self::Bku => "Semua penerimaan dan pengeluaran beserta saldo berjalan.",
            Self::PembantuKas => "Transaksi pada akun kas tunai.",
            Self::PembantuBank => "Transaksi pada satu rekening bank.",
            Self::PembantuPajak => "Pungutan dan setoran PPN, PPh 21, PPh 22, PPh 23.",
            Self::RekapKategori => "Total penerimaan dan pengeluaran tiap kategori.",
            Self::RealisasiAnggaran => "Perbandingan anggaran tahunan dengan realisasinya.",
        }
    }
}

/// Rentang tanggal laporan (inklusif).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Periode {
    pub mulai: Date,
    pub sampai: Date,
}

impl Periode {
    pub fn bulan(tahun: i16, bulan: i8) -> Self {
        let mulai = Date::new(tahun, bulan, 1).expect("bulan valid");
        Self {
            mulai,
            sampai: mulai.last_of_month(),
        }
    }
    pub fn tahun(tahun: i16) -> Self {
        Self {
            mulai: Date::new(tahun, 1, 1).unwrap(),
            sampai: Date::new(tahun, 12, 31).unwrap(),
        }
    }
    pub fn memuat(&self, d: Date) -> bool {
        d >= self.mulai && d <= self.sampai
    }
    pub fn label(&self) -> String {
        format!(
            "Periode {} s.d. {}",
            tanggal_panjang(self.mulai),
            tanggal_panjang(self.sampai)
        )
    }
}

/// Satu baris mutasi pada buku kas.
#[derive(Debug, Clone)]
pub struct Mutasi {
    pub tanggal: Date,
    pub no_bukti: String,
    pub kode: String,
    pub uraian: String,
    pub masuk: i64,
    pub keluar: i64,
}

/// Data yang dibutuhkan semua laporan.
pub struct Sumber<'a> {
    pub akun: &'a [Akun],
    pub kategori: &'a [Kategori],
    pub transaksi: &'a [Transaksi],
}

impl Sumber<'_> {
    fn kategori(&self, id: Option<i64>) -> Option<&Kategori> {
        id.and_then(|id| self.kategori.iter().find(|k| k.id == id))
    }

    fn nama_akun(&self, id: i64) -> &str {
        self.akun
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.nama.as_str())
            .unwrap_or("?")
    }

    /// Mutasi transaksi `t` yang menyentuh akun dalam `filter`
    /// (semua akun bila `None`).
    pub fn mutasi(&self, t: &Transaksi, filter: Option<&[i64]>) -> Vec<Mutasi> {
        let kena = |id: i64| filter.is_none_or(|f| f.contains(&id));
        let kode = self
            .kategori(t.kategori_id)
            .map(|k| k.kode.clone())
            .unwrap_or_default();
        let baris = |uraian: String, masuk, keluar| Mutasi {
            tanggal: t.tanggal,
            no_bukti: t.no_bukti.clone(),
            kode: kode.clone(),
            uraian,
            masuk,
            keluar,
        };
        let mut out = Vec::new();
        match t.jenis {
            JenisTransaksi::Masuk if kena(t.akun_id) => {
                out.push(baris(t.uraian.clone(), t.jumlah, 0))
            }
            JenisTransaksi::Keluar if kena(t.akun_id) => {
                out.push(baris(t.uraian.clone(), 0, t.jumlah));
                for p in JenisPajak::SEMUA {
                    let n = t.pungutan.get(p);
                    if n > 0 {
                        out.push(baris(
                            format!("Pungutan {} — {}", p.label(), t.uraian),
                            n,
                            0,
                        ));
                    }
                }
            }
            JenisTransaksi::Transfer => {
                let tujuan = t.akun_tujuan_id.unwrap_or(0);
                if kena(t.akun_id) {
                    out.push(baris(
                        format!("Transfer ke {} — {}", self.nama_akun(tujuan), t.uraian),
                        0,
                        t.jumlah,
                    ));
                }
                if kena(tujuan) {
                    out.push(baris(
                        format!("Transfer dari {} — {}", self.nama_akun(t.akun_id), t.uraian),
                        t.jumlah,
                        0,
                    ));
                }
            }
            _ => {}
        }
        out
    }

    /// Saldo akun-akun `filter` sebelum tanggal `sebelum`.
    pub fn saldo_sebelum(&self, filter: &[i64], sebelum: Date) -> i64 {
        let awal: i64 = self
            .akun
            .iter()
            .filter(|a| filter.contains(&a.id))
            .map(|a| a.saldo_awal)
            .sum();
        let mutasi: i64 = self
            .transaksi
            .iter()
            .filter(|t| t.tanggal < sebelum)
            .map(|t| filter.iter().map(|&a| t.efek_saldo(a)).sum::<i64>())
            .sum();
        awal + mutasi
    }

    /// Saldo akhir akun `id` saat ini.
    pub fn saldo_akun(&self, id: i64) -> i64 {
        let awal = self
            .akun
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.saldo_awal)
            .unwrap_or(0);
        awal + self.transaksi.iter().map(|t| t.efek_saldo(id)).sum::<i64>()
    }

    pub fn saldo_total(&self) -> i64 {
        self.akun.iter().map(|a| self.saldo_akun(a.id)).sum()
    }

    /// Total pemasukan dan pengeluaran (tanpa transfer internal) dalam periode.
    pub fn arus(&self, periode: Periode) -> (i64, i64) {
        self.transaksi
            .iter()
            .filter(|t| periode.memuat(t.tanggal))
            .fold((0, 0), |(m, k), t| match t.jenis {
                JenisTransaksi::Masuk => (m + t.jumlah, k),
                JenisTransaksi::Keluar => (m, k + t.jumlah - t.pungutan.total()),
                JenisTransaksi::Transfer => (m, k),
            })
    }

    // ---------- Laporan ----------

    pub fn buku_kas(&self, judul: &str, filter: &[i64], periode: Periode) -> Tabel {
        let mut saldo = self.saldo_sebelum(filter, periode.mulai);
        let mut baris = vec![vec![
            tanggal_pendek(periode.mulai),
            String::new(),
            String::new(),
            "Saldo awal".into(),
            String::new(),
            String::new(),
            ribuan(saldo),
        ]];
        let (mut tot_m, mut tot_k) = (0, 0);
        for t in self.transaksi.iter().filter(|t| periode.memuat(t.tanggal)) {
            for m in self.mutasi(t, Some(filter)) {
                saldo += m.masuk - m.keluar;
                tot_m += m.masuk;
                tot_k += m.keluar;
                baris.push(vec![
                    tanggal_pendek(m.tanggal),
                    m.no_bukti,
                    m.kode,
                    m.uraian,
                    nol_kosong(m.masuk),
                    nol_kosong(m.keluar),
                    ribuan(saldo),
                ]);
            }
        }
        Tabel {
            judul: judul.into(),
            subjudul: periode.label(),
            kolom: vec![
                kol("Tanggal", 1.1, false),
                kol("No. Bukti", 1.5, false),
                kol("Kode", 0.6, false),
                kol("Uraian", 4.0, false),
                kol("Penerimaan", 1.4, true),
                kol("Pengeluaran", 1.4, true),
                kol("Saldo", 1.5, true),
            ],
            baris,
            total: Some(vec![
                String::new(),
                String::new(),
                String::new(),
                "Jumlah / saldo akhir".into(),
                ribuan(tot_m),
                ribuan(tot_k),
                ribuan(saldo),
            ]),
            mendatar: true,
        }
    }

    pub fn bku(&self, periode: Periode) -> Tabel {
        let semua: Vec<i64> = self.akun.iter().map(|a| a.id).collect();
        self.buku_kas(JenisLaporan::Bku.judul(), &semua, periode)
    }

    pub fn pembantu_kas(&self, periode: Periode) -> Tabel {
        let tunai: Vec<i64> = self
            .akun
            .iter()
            .filter(|a| a.jenis == JenisAkun::Tunai)
            .map(|a| a.id)
            .collect();
        self.buku_kas(JenisLaporan::PembantuKas.judul(), &tunai, periode)
    }

    pub fn pembantu_bank(&self, akun_id: i64, periode: Periode) -> Tabel {
        let mut t = self.buku_kas(JenisLaporan::PembantuBank.judul(), &[akun_id], periode);
        if let Some(a) = self.akun.iter().find(|a| a.id == akun_id) {
            let rek = if a.no_rekening.is_empty() {
                String::new()
            } else {
                format!(" ({})", a.no_rekening)
            };
            t.subjudul = format!("{}{} · {}", a.nama, rek, t.subjudul);
        }
        t
    }

    pub fn pembantu_pajak(&self, periode: Periode) -> Tabel {
        // Saldo pajak = pungutan yang belum disetor.
        let efek = |t: &Transaksi| -> i64 {
            match (t.jenis, t.setor_pajak) {
                (JenisTransaksi::Keluar, Some(_)) => -t.jumlah,
                (JenisTransaksi::Keluar, None) => t.pungutan.total(),
                _ => 0,
            }
        };
        let mut saldo: i64 = self
            .transaksi
            .iter()
            .filter(|t| t.tanggal < periode.mulai)
            .map(efek)
            .sum();
        let mut baris = vec![vec![
            tanggal_pendek(periode.mulai),
            String::new(),
            "Saldo awal".into(),
            String::new(),
            String::new(),
            String::new(),
            ribuan(saldo),
        ]];
        let (mut tot_m, mut tot_k) = (0, 0);
        for t in self
            .transaksi
            .iter()
            .filter(|t| periode.memuat(t.tanggal) && t.jenis == JenisTransaksi::Keluar)
        {
            let mut tambah = |jenis: JenisPajak, uraian: String, masuk: i64, keluar: i64| {
                saldo += masuk - keluar;
                tot_m += masuk;
                tot_k += keluar;
                baris.push(vec![
                    tanggal_pendek(t.tanggal),
                    t.no_bukti.clone(),
                    uraian,
                    jenis.label().into(),
                    nol_kosong(masuk),
                    nol_kosong(keluar),
                    ribuan(saldo),
                ]);
            };
            if let Some(jenis) = t.setor_pajak {
                tambah(jenis, format!("Setoran — {}", t.uraian), 0, t.jumlah);
            } else {
                for p in JenisPajak::SEMUA {
                    let n = t.pungutan.get(p);
                    if n > 0 {
                        tambah(p, format!("Pungutan — {}", t.uraian), n, 0);
                    }
                }
            }
        }
        Tabel {
            judul: JenisLaporan::PembantuPajak.judul().into(),
            subjudul: periode.label(),
            kolom: vec![
                kol("Tanggal", 1.1, false),
                kol("No. Bukti", 1.5, false),
                kol("Uraian", 4.0, false),
                kol("Jenis", 0.9, false),
                kol("Pungutan", 1.4, true),
                kol("Setoran", 1.4, true),
                kol("Saldo", 1.4, true),
            ],
            baris,
            total: Some(vec![
                String::new(),
                String::new(),
                "Jumlah / saldo akhir".into(),
                String::new(),
                ribuan(tot_m),
                ribuan(tot_k),
                ribuan(saldo),
            ]),
            mendatar: true,
        }
    }

    /// Total per kategori dalam periode (pengeluaran dihitung bruto).
    pub fn total_per_kategori(&self, periode: Periode) -> HashMap<i64, i64> {
        let mut map = HashMap::new();
        for t in self.transaksi.iter().filter(|t| periode.memuat(t.tanggal)) {
            if let (Some(k), JenisTransaksi::Masuk | JenisTransaksi::Keluar) =
                (t.kategori_id, t.jenis)
            {
                *map.entry(k).or_insert(0) += t.jumlah;
            }
        }
        map
    }

    pub fn rekap_kategori(&self, periode: Periode) -> Tabel {
        let total = self.total_per_kategori(periode);
        let mut baris = Vec::new();
        let mut jumlah = [0i64; 2];
        for arus in [Arus::Masuk, Arus::Keluar] {
            baris.push(vec![
                String::new(),
                arus.label().to_uppercase(),
                String::new(),
            ]);
            let mut sub = 0;
            for k in self.kategori.iter().filter(|k| k.arus == arus) {
                let n = total.get(&k.id).copied().unwrap_or(0);
                sub += n;
                baris.push(vec![k.kode.clone(), k.nama.clone(), ribuan(n)]);
            }
            baris.push(vec![
                String::new(),
                format!("Jumlah {}", arus.label().to_lowercase()),
                ribuan(sub),
            ]);
            jumlah[(arus == Arus::Keluar) as usize] = sub;
        }
        Tabel {
            judul: JenisLaporan::RekapKategori.judul().into(),
            subjudul: periode.label(),
            kolom: vec![
                kol("Kode", 0.8, false),
                kol("Kategori", 5.0, false),
                kol("Jumlah", 2.0, true),
            ],
            baris,
            total: Some(vec![
                String::new(),
                "Selisih penerimaan − pengeluaran".into(),
                ribuan(jumlah[0] - jumlah[1]),
            ]),
            mendatar: false,
        }
    }

    pub fn realisasi_anggaran(&self, tahun: i16, anggaran: &[Anggaran]) -> Tabel {
        let realisasi = self.total_per_kategori(Periode::tahun(tahun));
        let mut baris = Vec::new();
        let (mut ta, mut tr) = (0, 0);
        for arus in [Arus::Masuk, Arus::Keluar] {
            baris.push(vec![
                String::new(),
                arus.label().to_uppercase(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ]);
            for k in self.kategori.iter().filter(|k| k.arus == arus) {
                let a = anggaran
                    .iter()
                    .find(|a| a.kategori_id == k.id)
                    .map(|a| a.jumlah)
                    .unwrap_or(0);
                let r = realisasi.get(&k.id).copied().unwrap_or(0);
                if a == 0 && r == 0 {
                    continue;
                }
                if arus == Arus::Keluar {
                    ta += a;
                    tr += r;
                }
                baris.push(vec![
                    k.kode.clone(),
                    k.nama.clone(),
                    ribuan(a),
                    ribuan(r),
                    ribuan(a - r),
                    persen(r, a),
                ]);
            }
        }
        Tabel {
            judul: JenisLaporan::RealisasiAnggaran.judul().into(),
            subjudul: format!("Tahun Anggaran {tahun}"),
            kolom: vec![
                kol("Kode", 0.7, false),
                kol("Kategori", 3.6, false),
                kol("Anggaran", 1.6, true),
                kol("Realisasi", 1.6, true),
                kol("Sisa", 1.6, true),
                kol("%", 0.8, true),
            ],
            baris,
            total: Some(vec![
                String::new(),
                "Jumlah belanja".into(),
                ribuan(ta),
                ribuan(tr),
                ribuan(ta - tr),
                persen(tr, ta),
            ]),
            mendatar: true,
        }
    }
}

fn nol_kosong(n: i64) -> String {
    if n == 0 { String::new() } else { ribuan(n) }
}

pub fn persen(bagian: i64, total: i64) -> String {
    if total == 0 {
        "—".into()
    } else {
        format!("{:.1}%", bagian as f64 * 100.0 / total as f64).replace('.', ",")
    }
}

/// Tulis tabel sebagai CSV (pemisah `;`, UTF-8 dengan BOM) agar langsung
/// terbaca rapi oleh Excel berbahasa Indonesia.
pub fn ke_csv(t: &Tabel) -> String {
    let sel = |s: &str| {
        if s.contains([';', '"', '\n']) {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    };
    let baris = |r: &[String]| r.iter().map(|s| sel(s)).collect::<Vec<_>>().join(";");
    let mut out = String::from("\u{feff}");
    out += &format!("{}\n{}\n", sel(&t.judul), sel(&t.subjudul));
    out += &t
        .kolom
        .iter()
        .map(|k| sel(k.nama))
        .collect::<Vec<_>>()
        .join(";");
    out.push('\n');
    for r in &t.baris {
        out += &baris(r);
        out.push('\n');
    }
    if let Some(total) = &t.total {
        out += &baris(total);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> (Vec<Akun>, Vec<Kategori>, Vec<Transaksi>) {
        let akun = vec![
            Akun {
                id: 1,
                nama: "Kas Tunai".into(),
                jenis: JenisAkun::Tunai,
                no_rekening: String::new(),
                saldo_awal: 100_000,
            },
            Akun {
                id: 2,
                nama: "Bank".into(),
                jenis: JenisAkun::Bank,
                no_rekening: "123".into(),
                saldo_awal: 1_000_000,
            },
        ];
        let kategori = vec![
            Kategori {
                id: 1,
                kode: "4.1".into(),
                nama: "BOS".into(),
                arus: Arus::Masuk,
            },
            Kategori {
                id: 2,
                kode: "5.2".into(),
                nama: "ATK".into(),
                arus: Arus::Keluar,
            },
            Kategori {
                id: 3,
                kode: "5.8".into(),
                nama: "Setor pajak".into(),
                arus: Arus::Keluar,
            },
        ];
        let t = |id, tgl: Date, jenis, akun_id, tujuan, kat, jumlah, pungutan, setor| Transaksi {
            id,
            tanggal: tgl,
            no_bukti: format!("X-{id}"),
            jenis,
            akun_id,
            akun_tujuan_id: tujuan,
            kategori_id: kat,
            uraian: "uji".into(),
            jumlah,
            pihak: String::new(),
            sumber_dana: String::new(),
            pungutan,
            setor_pajak: setor,
        };
        let d = |m, d| Date::constant(2026, m, d);
        let transaksi = vec![
            t(
                1,
                d(9, 30),
                JenisTransaksi::Masuk,
                2,
                None,
                Some(1),
                500_000,
                Pungutan::default(),
                None,
            ),
            t(
                2,
                d(10, 2),
                JenisTransaksi::Transfer,
                2,
                Some(1),
                None,
                300_000,
                Pungutan::default(),
                None,
            ),
            t(
                3,
                d(10, 5),
                JenisTransaksi::Keluar,
                1,
                None,
                Some(2),
                110_000,
                Pungutan {
                    ppn: 10_000,
                    ..Default::default()
                },
                None,
            ),
            t(
                4,
                d(10, 9),
                JenisTransaksi::Keluar,
                1,
                None,
                Some(3),
                10_000,
                Pungutan::default(),
                Some(JenisPajak::Ppn),
            ),
        ];
        (akun, kategori, transaksi)
    }

    #[test]
    fn saldo_dan_bku() {
        let (akun, kategori, transaksi) = data();
        let s = Sumber {
            akun: &akun,
            kategori: &kategori,
            transaksi: &transaksi,
        };
        // Tunai: 100.000 + 300.000 − 110.000 + 10.000 − 10.000
        assert_eq!(s.saldo_akun(1), 290_000);
        // Bank: 1.000.000 + 500.000 − 300.000
        assert_eq!(s.saldo_akun(2), 1_200_000);

        let okt = Periode::bulan(2026, 10);
        let bku = s.bku(okt);
        assert_eq!(bku.baris[0][6], "1.600.000", "saldo awal Oktober");
        assert_eq!(bku.total.as_ref().unwrap()[6], ribuan(s.saldo_total()));

        let pajak = s.pembantu_pajak(okt);
        assert_eq!(pajak.total.as_ref().unwrap()[6], "0", "PPN sudah disetor");

        let kas = s.pembantu_kas(okt);
        assert_eq!(kas.total.as_ref().unwrap()[6], "290.000");
    }

    #[test]
    fn realisasi() {
        let (akun, kategori, transaksi) = data();
        let s = Sumber {
            akun: &akun,
            kategori: &kategori,
            transaksi: &transaksi,
        };
        let ang = [Anggaran {
            kategori_id: 2,
            jumlah: 220_000,
        }];
        let t = s.realisasi_anggaran(2026, &ang);
        let atk = t.baris.iter().find(|r| r[0] == "5.2").unwrap();
        assert_eq!(atk[3], "110.000");
        assert_eq!(atk[5], "50,0%");
    }

    #[test]
    fn csv_aman() {
        let t = Tabel {
            judul: "A;B".into(),
            subjudul: String::new(),
            kolom: vec![kol("X", 1.0, false)],
            baris: vec![vec!["\"q\"".into()]],
            total: None,
            mendatar: false,
        };
        let csv = ke_csv(&t);
        assert!(csv.contains("\"A;B\""));
        assert!(csv.contains("\"\"\"q\"\"\""));
    }
}
