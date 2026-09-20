Core化の目的
境界設計
Measurement設計思想
Simulatorの位置付け
公開API方針
OSS化時の考え方



# Core Architecture and Library Design

## Overview

This document describes the architectural direction for separating the Digimatic parser system into a reusable Core library and an application layer.

The project originally started as a Rust learning project for communicating with Mitutoyo digital calipers. Through iterative refactoring, the system evolved into a layered architecture where protocol handling, domain modeling, simulation, and presentation responsibilities are separated.

The goal of Core extraction is not only code separation, but establishing a stable protocol-processing library that can be reused independently from the current application.

---

# Design Goal

The Core library should provide:

* Digimatic frame parsing
* Protocol-level validation
* Measurement domain representation
* Unit-aware value access

The Core library should not depend on:

* GUI frameworks
* Logging systems
* Operating system APIs
* Serial communication implementations
* Simulation logic

The dependency direction should be:

```
Application
    |
    v
Core Library
```

The Core must remain independent from the application layer.

---

# Core Responsibilities

The Core layer contains concepts that represent the measurement domain.

Expected components:

```
core/

├── frame
│   └── Digimatic frame representation
│
├── parser
│   └── Frame decoding and validation
│
├── measurement
│   └── Parsed measurement domain object
│
├── measurement_value
│   └── Public API value representation
│
└── unit
    └── Measurement unit definition
```

---

# Measurement Design Philosophy

`Measurement` is the central domain object.

A Measurement represents:

> A measurement value that has already passed through the parsing process.

It is not intended to be a general-purpose data container.

Therefore:

* Fields are private.
* Construction is intentionally restricted.
* External code should not freely create Measurement instances.

The design principle is:

```
Raw Frame
    |
    v
Parser
    |
    v
Measurement
```

This ensures that every Measurement has passed through the same validation pipeline.

Allowing arbitrary construction would weaken the meaning of the type itself.

---

# Measurement Construction Policy

Measurement construction is intentionally limited.

Allowed paths:

```
TryFrom<DigimaticFrame>
        |
        v
Measurement


dummy()
        |
        v
Measurement
```

The parser path represents real measurement flow.

The dummy path exists for:

* GUI development
* Application testing
* Simulation support

It is not intended as a normal construction mechanism.

---

# MeasurementValue Public API

`MeasurementValue` exists as an external-facing representation.

Concept:

> A measuring instrument provides a value together with its unit, similar to reading a caliper display.

Example:

```
MeasurementValue {
    value: 12.34,
    unit: mm
}
```

The purpose is to provide a stable API boundary.

Internal application code should normally use:

```
Measurement methods
```

because they preserve domain meaning.

External users of the library may prefer:

```
MeasurementValue
```

because it provides a simple value + unit representation.

---

# Simulator Position

The simulator is not part of Core.

The simulator is a validation and development tool.

Its responsibility is:

* Generate realistic measurement streams
* Create Digimatic frames
* Test parser behavior
* Test error handling

Architecture:

```
Simulator

Base Signal Generator
        |
        v
Frame Generator
        |
        v
Effector
        |
        +-- Noise
        +-- Fault Injection
        |
        v
Parser
        |
        v
Measurement
```

---

# Fault Injection

Fault Injection belongs to the simulator layer.

Its purpose is to intentionally generate invalid input:

Examples:

* Bit corruption
* Missing data
* Invalid characters
* Short packets
* Communication interruption simulation

The purpose is to verify that the Core parser behaves correctly.

It should not exist inside Core because Core consumes data; it does not create test failures.

---

# Application Layer Responsibilities

The application layer contains:

```
app/

├── GUI
├── CLI
├── Logger
├── Serial communication
└── Simulator
```

The application demonstrates how the Core library can be used.

It is an example implementation, not part of the protocol definition.

---

# OSS Release Philosophy

The project is niche, but the value is not only the number of users.

The important goal is:

* Provide a clean Digimatic parsing implementation
* Document protocol handling
* Provide a practical example application

The Core library should be designed so that someone interested in Digimatic communication can discover:

* How frames are decoded
* How measurements are represented
* How the pipeline is structured

---

# Future Direction

Possible future improvements:

* Extract Core into a separate crate
* English documentation for Core APIs
* Public API stabilization
* Protocol extension support
* Binary frame support

Core extraction should happen after the current application structure is stabilized.

The current application provides the reference implementation and validation environment for the future library.
