//! presentation.rs
//! measurement構造体の見せ方定義

use crate::frame::{Measurement, Unit};

pub fn format_measurement_value_with_unit(m: &Measurement) -> String {
    let unit = match m.unit {
        Unit::Mm => "mm",
        Unit::Inch => "inch",
    };

    format!("{:.2} {}", m.to_f64(), unit)
}
