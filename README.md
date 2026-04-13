# Mitutoyo Digital Caliper to PC Interface (SPC)

*[日本語はこちら](README.ja.md)*

Capture, decode, and log measurement data from Mitutoyo digital calipers using a Raspberry Pi Pico.

---

## ✨ Overview

This project provides a complete pipeline for receiving Digimatic data from Mitutoyo calipers and processing it on a PC.

- 📡 Capture raw signals via Raspberry Pi Pico (XIAO RP2040)  
- 🔄 Decode Digimatic frames into measurement values  
- 💾 Log both raw and processed data  
- 🧪 Test everything without hardware using a built-in simulator  

---

## 🎯 Who is this for?

This project is primarily designed for personal and hobby use, but also serves as a practical tool for:

- Logging measurements from digital calipers  
- Experimenting with Digimatic communication protocols  
- Learning embedded ↔ PC data communication  
- Debugging low-level signal handling  

It is especially useful if you want access to both **raw communication data** and **decoded measurement values**.

---

## 🔧 Tech Stack

- **PC Side:** Rust  
- **Hardware Interface:** MicroPython (Raspberry Pi Pico / XIAO RP2040)

---

## 🔁 Data Flow

`Caliper` → `Level Shifter (SN74LXC8T245PWR)` → `XIAO RP2040` → `PC (Linux / Windows)`

---

## 🚀 Features

- Real-time measurement data capture  
- Digimatic frame decoding  
- CSV logging and terminal display  
- CLI-based simulator (no hardware required)  
- Virtual serial communication for realistic testing  
- Built-in diagnostic mode on Pico  

---

## 🧪 CLI Simulator (No Hardware Required)

*Available on Linux/macOS (uses `socat`; not supported on Windows).*

Test the full data pipeline without connecting a physical caliper.

### Pico Simulation Mode
- Generates simulated caliper measurement data  
- Sends Digimatic frames as strings via CDC-USB (virtual serial port)  

### PC Simulation Mode
- Provides a socket-based environment for communication testing  

### Highlights
- Realistic virtual serial communication (not in-memory)  
- Frame generation based on Mitutoyo specifications  
- Frame decoding and validation  

---

## 🧰 Pico Diagnostic Mode

A built-in interactive diagnostic tool for GPIO and device behavior.

- Text-based menu interface  
- Real-time GPIO monitoring  
- Temporary device configuration  
- Pin toggle testing  

Type `Diag` in the terminal to enter this mode.

> The diagnostic mode is feature-rich enough to function as a standalone debugging tool.

---

## 🖥 GUI Display (Windows)

<a href="./pc_tool/assets/DisplayWindow(windows).png">
  <img src="./pc_tool/assets/DisplayWindow(windows).png" alt="GUI Mode Display" width="220">
</a>

- Currently available on **Windows** (Linux support TBD)

**Launch:**
```bash
cargo run --bin digimatic -- -gui -sim