# About This Project
*[日本語はこちら](README.ja.md)*

This is a tool (currently in development) designed to receive data output from a Mitutoyo digital caliper and send it to a PC.
The signals from the caliper are intercepted by a Raspberry Pi Pico (specifically, a Seeed XIAO RP2040) and forwarded to the PC.

**Signal Flow:**
Caliper
  → Level Shifter (SN74LXC8T245PWR on a DIP adapter)
  → RP Pico (XIAO RP2040)
  → PC (Linux / Windows)

## Current Status
At the current stage:
- **Runs as a Simulator** (Linux only, utilizing `socat`):
    - Generates mock caliper measurement data.
    - Generates frame strings compliant with Mitutoyo's data output specifications.
    - Creates virtual serial ports for transmission and reception.
    - Transmits and receives data through these virtual ports.
    - Decodes the received frames to extract measurement values.

- **Hardware Integration**:
    - Successfully verified data reception from the Pico to the PC via USB-CDC.
    - The electronic circuitry to connect the actual caliper is currently under construction (Waiting for parts as of Feb 19, 2026).

## ToDo
Build the hardware interface for the actual digital caliper (Electronics):
- **Caliper connection cable:** Mitutoyo genuine flat straight cable (905338)
- **Cable output receptacle:** 10-pin Box Header, PCB mount (Marutsu Online 217010SE)
- **Data receiver MCU:** Seeed Studio XIAO RP2040
- **Signal level shifter (1.5V to 3.3V):** SN74LXC8T245PWR
- **TSSOP24 to DIP adapter board:** DA-TSSOP24-P65
- **Power supply for level shifter:** LDO AP2112