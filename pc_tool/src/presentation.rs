//! presentation.rs
//! measurement構造体の見せ方定義

use crate::frame::{Measurement, Unit};

/// 共通整形ルール 見せる必要ないので pubにしない
fn format_logic(val: f64, unit: Unit, precision: usize) -> String {
    let unit_str = match unit {
        Unit::Mm => "mm",
        Unit::Inch => "inch",
    };
    format!("{:.*} {}", precision, val, unit_str)
}

/// GUI表示用ラッパー
pub fn format_with_display_unit(m: &Measurement, display_unit: Unit) -> String {
    const MM_PER_INCH: f64 = 25.4;
    let base_val = m.to_f64();

    // 変換ロジック
    let converted_val = match (m.unit, display_unit) {
        (Unit::Mm, Unit::Inch) => base_val / MM_PER_INCH,
        (Unit::Inch, Unit::Mm) => base_val * MM_PER_INCH,
        _ => base_val,
    };

    // 2. PointPositionシフト 計測値を欲しい単位(mm/Inch)で表示するときの桁数決定
    let precision = match (m.unit, display_unit) {
        (Unit::Mm, Unit::Inch) => (m.point as usize) + 2,
        (Unit::Inch, Unit::Mm) => (m.point as usize).saturating_sub(2).max(1),
        _ => m.point as usize,
    };

    format_logic(converted_val, display_unit, precision)
}
