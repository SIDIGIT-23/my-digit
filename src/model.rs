//! Struktur data keuangan My Digit.
//!
//! Semua nominal disimpan dalam rupiah utuh (`i64`), tanpa pecahan sen.

use jiff::civil::Date;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JenisAkun {
    Tunai,
    Bank,
}

impl JenisAkun {
    pub fn kode(self) -> &'static str {
        match self {
            Self::Tunai => "tunai",
            Self::Bank => "bank",
        }
    }
    pub fn dari_kode(s: &str) -> Self {
        if s == "bank" { Self::Bank } else { Self::Tunai }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Tunai => "Kas Tunai",
            Self::Bank => "Bank",
        }
    }
}

/// Akun penyimpanan uang: kas tunai atau rekening bank.
#[derive(Debug, Clone)]
pub struct Akun {
    pub id: i64,
    pub nama: String,
    pub jenis: JenisAkun,
    pub no_rekening: String,
    pub saldo_awal: i64,
}

/// Arah arus kas sebuah kategori.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arus {
    Masuk,
    Keluar,
}

impl Arus {
    pub fn kode(self) -> &'static str {
        match self {
            Self::Masuk => "masuk",
            Self::Keluar => "keluar",
        }
    }
    pub fn dari_kode(s: &str) -> Self {
        if s == "keluar" {
            Self::Keluar
        } else {
            Self::Masuk
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Masuk => "Penerimaan",
            Self::Keluar => "Pengeluaran",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Kategori {
    pub id: i64,
    pub kode: String,
    pub nama: String,
    pub arus: Arus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JenisTransaksi {
    Masuk,
    Keluar,
    Transfer,
}

impl JenisTransaksi {
    pub fn kode(self) -> &'static str {
        match self {
            Self::Masuk => "masuk",
            Self::Keluar => "keluar",
            Self::Transfer => "transfer",
        }
    }
    pub fn dari_kode(s: &str) -> Self {
        match s {
            "keluar" => Self::Keluar,
            "transfer" => Self::Transfer,
            _ => Self::Masuk,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Masuk => "Pemasukan",
            Self::Keluar => "Pengeluaran",
            Self::Transfer => "Transfer",
        }
    }
    /// Awalan nomor bukti: Bukti Kas Masuk, Bukti Kas Keluar, Transfer.
    pub fn awalan_bukti(self) -> &'static str {
        match self {
            Self::Masuk => "BKM",
            Self::Keluar => "BKK",
            Self::Transfer => "TRF",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JenisPajak {
    Ppn,
    Pph21,
    Pph22,
    Pph23,
}

impl JenisPajak {
    pub const SEMUA: [JenisPajak; 4] = [Self::Ppn, Self::Pph21, Self::Pph22, Self::Pph23];

    pub fn kode(self) -> &'static str {
        match self {
            Self::Ppn => "ppn",
            Self::Pph21 => "pph21",
            Self::Pph22 => "pph22",
            Self::Pph23 => "pph23",
        }
    }
    pub fn dari_kode(s: &str) -> Option<Self> {
        Self::SEMUA.into_iter().find(|p| p.kode() == s)
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Ppn => "PPN",
            Self::Pph21 => "PPh 21",
            Self::Pph22 => "PPh 22",
            Self::Pph23 => "PPh 23",
        }
    }
}

/// Pajak yang dipungut bendahara saat membayar sebuah belanja.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pungutan {
    pub ppn: i64,
    pub pph21: i64,
    pub pph22: i64,
    pub pph23: i64,
}

impl Pungutan {
    pub fn get(&self, p: JenisPajak) -> i64 {
        match p {
            JenisPajak::Ppn => self.ppn,
            JenisPajak::Pph21 => self.pph21,
            JenisPajak::Pph22 => self.pph22,
            JenisPajak::Pph23 => self.pph23,
        }
    }
    pub fn get_mut(&mut self, p: JenisPajak) -> &mut i64 {
        match p {
            JenisPajak::Ppn => &mut self.ppn,
            JenisPajak::Pph21 => &mut self.pph21,
            JenisPajak::Pph22 => &mut self.pph22,
            JenisPajak::Pph23 => &mut self.pph23,
        }
    }
    pub fn total(&self) -> i64 {
        self.ppn + self.pph21 + self.pph22 + self.pph23
    }
}

#[derive(Debug, Clone)]
pub struct Transaksi {
    pub id: i64,
    pub tanggal: Date,
    pub no_bukti: String,
    pub jenis: JenisTransaksi,
    /// Akun sumber (keluar/transfer) atau tujuan (masuk).
    pub akun_id: i64,
    /// Akun tujuan, hanya untuk transfer.
    pub akun_tujuan_id: Option<i64>,
    pub kategori_id: Option<i64>,
    pub uraian: String,
    /// Nominal bruto.
    pub jumlah: i64,
    /// Pihak penyetor (pemasukan) atau penerima (pengeluaran).
    pub pihak: String,
    pub sumber_dana: String,
    pub pungutan: Pungutan,
    /// Bila diisi, transaksi ini adalah penyetoran pajak tersebut ke kas negara.
    pub setor_pajak: Option<JenisPajak>,
}

impl Transaksi {
    /// Perubahan saldo `akun` akibat transaksi ini.
    pub fn efek_saldo(&self, akun: i64) -> i64 {
        match self.jenis {
            JenisTransaksi::Masuk if self.akun_id == akun => self.jumlah,
            JenisTransaksi::Keluar if self.akun_id == akun => -self.jumlah + self.pungutan.total(),
            JenisTransaksi::Transfer => {
                let mut d = 0;
                if self.akun_id == akun {
                    d -= self.jumlah;
                }
                if self.akun_tujuan_id == Some(akun) {
                    d += self.jumlah;
                }
                d
            }
            _ => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Anggaran {
    pub kategori_id: i64,
    pub jumlah: i64,
}

/// Identitas lembaga pengguna, dipakai untuk kop dan tanda tangan laporan.
///
/// `logo` adalah logo milik lembaga (mis. logo sekolah), terpisah dari logo
/// aplikasi My Digit di `assets/logo.png`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProfilLembaga {
    pub nama: String,
    pub alamat: String,
    pub kota: String,
    pub telepon: String,
    pub email: String,
    pub jabatan_pimpinan: String,
    pub nama_pimpinan: String,
    pub nip_pimpinan: String,
    pub nama_bendahara: String,
    pub nip_bendahara: String,
    /// Nama berkas logo lembaga di folder data aplikasi.
    pub logo: Option<String>,
}
