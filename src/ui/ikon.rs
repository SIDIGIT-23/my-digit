//! Ikon garis [Phosphor Icons](https://phosphoricons.com) (lisensi MIT, lihat
//! `assets/fonts/LICENSE-Phosphor.txt`), pengganti emoji dengan gaya yang
//! mirip SF Symbols.
//!
//! Ikon punya keluarga huruf sendiri ([`keluarga`]) dan tidak dicampur ke huruf
//! teks: Phosphor memetakan a–z ke glyph kosong, dan huruf sistem (SF Pro,
//! Segoe UI) bisa punya glyph di area Private Use yang sama.

use eframe::egui::{FontFamily, FontId, RichText};

pub const FONT: &[u8] = include_bytes!("../../assets/fonts/Phosphor.ttf");
const NAMA: &str = "ikon";

pub fn keluarga() -> FontFamily {
    FontFamily::Name(NAMA.into())
}

pub fn font(ukuran: f32) -> FontId {
    FontId::new(ukuran, keluarga())
}

/// Ikon sebagai teks kaya, siap diberi `.size()` atau `.color()`.
pub fn rt(ikon: &str) -> RichText {
    RichText::new(ikon).family(keluarga())
}

/// Daftarkan font ikon, dengan huruf teks sebagai cadangan di belakangnya.
pub fn pasang(fonts: &mut eframe::egui::FontDefinitions) {
    fonts.font_data.insert(
        NAMA.into(),
        eframe::egui::FontData::from_static(FONT).into(),
    );
    let mut urutan = vec![NAMA.to_string()];
    urutan.extend(
        fonts
            .families
            .get(&FontFamily::Proportional)
            .cloned()
            .unwrap_or_default(),
    );
    fonts.families.insert(keluarga(), urutan);
}

// Kode glyph dari Phosphor Regular (nama mengikuti nama ikon resminya).
pub const ARCHIVE: &str = "\u{E00C}";
pub const ARROW_COUNTER_CLOCKWISE: &str = "\u{E038}";
pub const ARROW_DOWN_LEFT: &str = "\u{E040}";
pub const ARROW_UP_RIGHT: &str = "\u{E092}";
pub const ARROWS_LEFT_RIGHT: &str = "\u{E0A0}";
pub const BANK: &str = "\u{E0B4}";
pub const BUILDINGS: &str = "\u{E102}";
pub const CALENDAR_BLANK: &str = "\u{E10A}";
pub const CARET_DOWN: &str = "\u{E136}";
pub const CARET_LEFT: &str = "\u{E138}";
pub const CARET_RIGHT: &str = "\u{E13A}";
pub const CARET_UP_DOWN: &str = "\u{E140}";
pub const CHART_PIE_SLICE: &str = "\u{E15A}";
pub const CHECK_CIRCLE: &str = "\u{E184}";
pub const DOTS_THREE_CIRCLE: &str = "\u{E200}";
pub const ERASER: &str = "\u{E21E}";
pub const FILE_PDF: &str = "\u{E702}";
pub const FILE_TEXT: &str = "\u{E23A}";
pub const FILE_XLS: &str = "\u{EB22}";
pub const FOLDER_OPEN: &str = "\u{E256}";
pub const GEAR_SIX: &str = "\u{E272}";
pub const HOUSE: &str = "\u{E2C2}";
pub const IMAGE: &str = "\u{E2CA}";
pub const INFO: &str = "\u{E2CE}";
pub const MAGNIFYING_GLASS: &str = "\u{E30C}";
pub const MINUS: &str = "\u{E32A}";
pub const MONEY: &str = "\u{E588}";
pub const PENCIL_SIMPLE: &str = "\u{E3B4}";
pub const PLUS: &str = "\u{E3D4}";
pub const PRINTER: &str = "\u{E3DC}";
pub const RECEIPT: &str = "\u{E3EC}";
pub const SEAL_PERCENT: &str = "\u{E60A}";
pub const SWAP: &str = "\u{E83C}";
pub const TAG: &str = "\u{E478}";
pub const TRASH: &str = "\u{E4A6}";
pub const WALLET: &str = "\u{E68A}";
pub const WARNING_CIRCLE: &str = "\u{E4E2}";
pub const X: &str = "\u{E4F6}";
