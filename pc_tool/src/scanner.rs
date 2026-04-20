//!
//! pico の 接続ポートを探す
//!
//! 引数 :
//! 返値 : pico の接続されている ポート
//!
//!

// 方針：PID と VID でさがす。
//      複数台の接続は考慮しない  → 1台のみを想定

use serialport::{SerialPortType, UsbPortInfo, available_ports};

use crate::errors::{CommError, DigimaticError};

// 下記は lsusb で確認したうえで修正
const PICO_VID: u16 = 0x2E8A; // Raspberry PI
const PICO_PID: u16 = 0x0005; // MicroPython

pub fn find_pico_port() -> Result<String, DigimaticError> {
    let ports_list = available_ports().map_err(CommError::from)?;

    for p in ports_list {
        match &p.port_type {
            SerialPortType::UsbPort(UsbPortInfo { vid, pid, .. })
                if *vid == PICO_VID && *pid == PICO_PID =>
            {
                return Ok(p.port_name.clone());
            }
            _ => continue,
        }
    }
    eprint!("Pico port not found");
    Err(DigimaticError::Comm(CommError::ConnectionClosed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actual_pico_connection() {
        let result = find_pico_port();

        match result {
            Ok(port) => {
                println!("\n✅ Pico detected at : {}", port);
                assert!(!port.is_empty());
            }
            Err(e) => {
                panic!(
                    "\n✖ Pico not found! Check connectino or VDI/PID. Error {} ",
                    e
                );
            }
        }
    }
}
