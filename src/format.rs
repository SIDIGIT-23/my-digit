//! Format angka, tanggal, dan terbilang dalam bahasa Indonesia.

use jiff::civil::Date;

pub const BULAN: [&str; 12] = [
    "Januari",
    "Februari",
    "Maret",
    "April",
    "Mei",
    "Juni",
    "Juli",
    "Agustus",
    "September",
    "Oktober",
    "November",
    "Desember",
];

/// `1250000` → `"1.250.000"`.
pub fn ribuan(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if n < 0 {
        out.push('-');
    }
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(ch);
    }
    out
}

/// `1250000` → `"Rp 1.250.000"`.
pub fn rupiah(n: i64) -> String {
    if n < 0 {
        format!("-Rp {}", ribuan(-n))
    } else {
        format!("Rp {}", ribuan(n))
    }
}

/// Baca nominal yang diketik pengguna: titik, spasi, dan "Rp" diabaikan.
pub fn parse_rupiah(s: &str) -> Option<i64> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return if s.trim().is_empty() { Some(0) } else { None };
    }
    digits.parse().ok()
}

/// `2026-10-21` → `"21 Oktober 2026"`.
pub fn tanggal_panjang(d: Date) -> String {
    format!("{} {} {}", d.day(), BULAN[d.month() as usize - 1], d.year())
}

/// `2026-10-21` → `"21/10/2026"`.
pub fn tanggal_pendek(d: Date) -> String {
    format!("{:02}/{:02}/{}", d.day(), d.month(), d.year())
}

pub fn hari_ini() -> Date {
    jiff::Zoned::now().date()
}

/// Ejaan nominal untuk kwitansi: `1500` → `"seribu lima ratus"`.
pub fn terbilang(n: i64) -> String {
    if n == 0 {
        return "nol".into();
    }
    let mut s = eja(n.unsigned_abs());
    if n < 0 {
        s = format!("minus {s}");
    }
    s
}

fn eja(n: u64) -> String {
    const SATUAN: [&str; 12] = [
        "", "satu", "dua", "tiga", "empat", "lima", "enam", "tujuh", "delapan", "sembilan",
        "sepuluh", "sebelas",
    ];
    let gabung = |a: String, b: String| {
        if b.is_empty() { a } else { format!("{a} {b}") }
    };
    match n {
        0..=11 => SATUAN[n as usize].into(),
        12..=19 => format!("{} belas", SATUAN[(n - 10) as usize]),
        20..=99 => gabung(format!("{} puluh", SATUAN[(n / 10) as usize]), eja(n % 10)),
        100..=199 => gabung("seratus".into(), eja(n - 100)),
        200..=999 => gabung(
            format!("{} ratus", SATUAN[(n / 100) as usize]),
            eja(n % 100),
        ),
        1_000..=1_999 => gabung("seribu".into(), eja(n - 1_000)),
        2_000..=999_999 => gabung(format!("{} ribu", eja(n / 1_000)), eja(n % 1_000)),
        1_000_000..=999_999_999 => {
            gabung(format!("{} juta", eja(n / 1_000_000)), eja(n % 1_000_000))
        }
        1_000_000_000..=999_999_999_999 => gabung(
            format!("{} miliar", eja(n / 1_000_000_000)),
            eja(n % 1_000_000_000),
        ),
        _ => gabung(
            format!("{} triliun", eja(n / 1_000_000_000_000)),
            eja(n % 1_000_000_000_000),
        ),
    }
}

/// Huruf kapital di awal setiap kata.
pub fn judul_kata(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ribuan() {
        assert_eq!(ribuan(0), "0");
        assert_eq!(ribuan(999), "999");
        assert_eq!(ribuan(1_000), "1.000");
        assert_eq!(ribuan(1_250_000), "1.250.000");
        assert_eq!(rupiah(-5_000), "-Rp 5.000");
    }

    #[test]
    fn baca_nominal() {
        assert_eq!(parse_rupiah("Rp 1.250.000"), Some(1_250_000));
        assert_eq!(parse_rupiah(""), Some(0));
        assert_eq!(parse_rupiah("abc"), None);
    }

    #[test]
    fn ejaan() {
        assert_eq!(terbilang(11), "sebelas");
        assert_eq!(terbilang(15), "lima belas");
        assert_eq!(terbilang(100), "seratus");
        assert_eq!(terbilang(1_500), "seribu lima ratus");
        assert_eq!(terbilang(2_750_000), "dua juta tujuh ratus lima puluh ribu");
        assert_eq!(terbilang(1_001_000), "satu juta seribu");
        assert_eq!(terbilang(101_000), "seratus satu ribu");
    }
}
