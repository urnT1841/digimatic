# GUI Architecture

## Overview

The application provides two presentation paths:

* Native GUI using eframe/egui
* Browser-based GUI using WebSocket communication

The GUI layer is intentionally separated from the measurement processing pipeline.

The GUI consumes processed Measurement data but does not participate in parsing or protocol handling.

---

# Presentation Flow

The general pipeline is:

```
Input
 |
 v
Parser
 |
 v
Measurement
 |
 v
Presentation Layer
 |
 +-- Native GUI
 |
 +-- Browser GUI
```

The GUI is a consumer of domain data.

---

# Native GUI (eframe)

Windows and macOS use the native eframe application.

Reason:

* Native window support
* Same rendering framework across platforms
* No browser dependency
* Better desktop application experience

Flow:

```
Measurement
    |
    v
Channel
    |
    v
eframe Application
    |
    v
egui Rendering
```

---

# Browser GUI (Linux)

Linux currently uses a browser-based presentation.

Architecture:

```
Measurement
    |
    v
Rust Web Server
    |
    v
WebSocket
    |
    v
Browser
```

Reason:

* Easier deployment on Linux environments
* Flexible frontend development
* Avoiding platform-specific GUI issues

---

# GUI and Core Separation

The GUI must not affect the Core design.

The Core should not know:

* How data is displayed
* Whether a browser or native window is used
* How graphs are rendered

The GUI receives already-validated Measurement data.

---

# Real-Time Graph Design

The trend graph visualizes continuous measurement changes.

Design goals:

* High visibility
* Stable rendering
* Small displacement visualization

Features:

* Dynamic Y-axis scaling
* Line graph rendering
* Measurement point markers
* Limited history buffer

The graph is intended for observing measurement behavior rather than replacing precision measurement software.

---

# GUI as Reference Application

The application layer demonstrates practical usage of the Core library.

It provides examples of:

* Real-time measurement processing
* Logging
* Visualization
* Simulation

Future Core users can refer to the application as an integration example.

---

# Future Direction

Possible improvements:

* Unified GUI/CLI presentation architecture
* Additional platform support
* Improved frontend separation
* Independent visualization components

The GUI remains an application feature and is intentionally kept outside the Core library.
