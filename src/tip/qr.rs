//! QR code as one SVG path (a few KB of DOM instead of ~1,000 `<rect>`s), drawn with the page's own CSS.

use qrcode::{Color, EcLevel, QrCode};

/// Light border around the code, in modules (the QR spec asks for 4).
pub const QUIET_ZONE: usize = 4;

/// A QR code ready to draw: `view_box` side (modules plus both quiet zones) and the dark-module path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QrPath {
    pub size: usize,
    pub d: String,
}

/// Low error correction: a lit screen has no print damage to recover from, and the smaller code (a tip URL is
/// ~220 bytes: version 9, 53 modules) gets bigger modules at the same size, which is what a far phone camera needs.
pub fn qr_path(data: &str) -> Option<QrPath> {
    let code = QrCode::with_error_correction_level(data.as_bytes(), EcLevel::L).ok()?;
    let width = code.width();
    let d = code
        .to_colors()
        .iter()
        .enumerate()
        .filter(|(_, color)| **color == Color::Dark)
        .map(|(i, _)| {
            let (x, y) = (i % width + QUIET_ZONE, i / width + QUIET_ZONE);
            format!("M{x} {y}h1v1h-1z")
        })
        .collect();
    Some(QrPath { size: width + 2 * QUIET_ZONE, d })
}
