//!
//! pico の 接続ポートを探す
//! 
//! 引数 :
//! 返値 : pico の接続されている ポート
//! 
//! 

// 方針：PID と VID でさがす。
//      複数台の接続は考慮しない  → 1台のみを想定

use serialport::{available_ports, SerialPortType, UsbPortInfo};


// 下記は lsusb で確認したうえで修正
const PICO_VID: u16 = 0x2E8A;   // Raspberry PI
const PICO_PID: u16 = 0x0005;   // MicroPython


pub fn find_pico_port() -> Result<String, Box<dyn std::error::Error>> {

    // available_ports()で探す。返値は Vec. pico portはループで探す
    let ports_list = available_ports()?;

    for p in ports_list {
        match &p.port_type {
            //USB接続で pico の vdi/pidを持つもの
            SerialPortType::UsbPort(UsbPortInfo { 
                vid: PICO_VID, 
                pid: PICO_PID,
                ..
            }) => {
                // 条件に合致 (USBでPICO)
                return Ok(p.port_name.clone());
            }

            // それ以外（Bluetoothだったり、USBでもVIDが違うやつ）は全部無視
            _ => continue,
        }
    }
   Err("Pico port not found".into())
}