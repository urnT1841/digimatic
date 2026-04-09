# About This Project
*[日本語はこちら](README.ja.md)*

# Mitutoyo Digital Caliper to PC Interface (SPC)

This tool enables PCs to receive data output from Mitutoyo digital calipers.
The signal from the caliper is captured by a Raspberry Pi Pico (specifically, a Seeed Studio XIAO RP2040) and transmitted to the PC.

**Tech Stack:**
- **PC Side:** Rust
- **Hardware Interface:** MicroPython (XIAO RP2040)

### Data Flow
`Caliper` → `Level Shifter (SN74LXC8T245PWR)` → `XIAO RP2040` → `PC (Linux / Windows)`

---

## Current Status

### 1. Simulator Functionality
*Currently for Linux only, utilizing `socat`.*
- Generates simulated caliper measurement data.
- Generates frame strings conforming to Mitutoyo specifications.
- Creates virtual ports for both transmission and reception.
- Transmits and receives data via virtual ports.
- Decodes frames to retrieve measurement values.

### 2. GUI Display
<a href="./pc_tool/assets/DisplayWindow(windows).png">
  <img src="./pc_tool/assets/DisplayWindow(windows).png" alt="GUI Mode Display" width="280">
</a>

- The GUI mode is currently available for **Windows**. (Linux support is TBD).
- **To Launch:** Run with `-gui -sim` flags.
  ```bash
  cargo run --bin digimatic -- -gui -sim

### 3. Hardware Implementation
Communication: Successfully verified on a breadboard.

Assembly: Currently migrating the circuit to a universal board (perfboard).

### 4. Software Progress
  - Pico (MicroPython): Receives bitstreams from the caliper and forwards them to the PC.
  - PC (Rust): Receives string frames and logs them to CSV or displays them in the terminal.

Implementation of the dedicated display window is in progress.

## Software Setup
RP Pico Firmware
Transfer the following files to the Pico. main.py will execute automatically upon reboot:
  - main.py
  - pin_definitions.py
  - led_switch.py
  - state_process.py
  - decoder.py
  - communicator.py

### Required Components
Details for the hardware interface (Electronic construction):

  - Connection Cable: Mitutoyo Genuine Flat Straight Cable (905338)
  - Connector: 10-pin Box Header, PCB Mount (e.g., Marutsu 217010SE)
  - Microcontroller: Seeed Studio XIAO RP2040
  - Level Shifter (1.5V -> 3.3V): SN74LXC8T245PWR
  - Adapter Board: TSSOP24 to DIP conversion board (DA-TSSOP24-P65)
  - LDO Regulator: AP2112 (for Level Shifter power supply)

## TODO
[ ] Integrate Serial Communication: Replace the generator with CdcReceiver to stream real-time data from the actual caliper to the DisplayApp.

[ ] Settings UI: Implement UI in egui to allow users to change font sizes and colors.

[ ] Unit Display: Add units (e.g., "mm") next to the numerical values.