# 🚀 Digimatic Data System v2.0.0

## Overview

Version 2.0.0 introduces a major architectural stabilization of the Digimatic data processing system.

This release focuses on **architecture cleanup, module boundary clarification, and legacy isolation**, rather than feature expansion.

The system is now structurally stable for future extensions such as binary frame support and OS-level virtual port abstraction.

---

## ✨ Highlights

- Clear separation of core logic and simulation layer
- Formalized pub / private API boundaries
- Legacy I/O systems isolated from active runtime
- Stable measurement pipeline established
- GUI and CLI presentation layer clarified

---

## 🧭 Architecture (v2)

Sim / Real Input
↓
Frame Parser (frame.rs / parser.rs)
↓
Measurement (core representation)
↓
Presentation / Logger / GUI

The system now enforces **Measurement as the single unified intermediate representation**.

---

## 🧱 Major Changes

### 1. Legacy migration

The following modules have been moved to legacy:

- `sim/sender.rs`
- `sim/port_prepare.rs`

#### Reason
- Replaced by mpsc-based internal communication
- Socat-based virtual port pipeline is no longer part of active runtime

---

### 2. Simulation layer (`sim`)

- Simulation module remains active
- Responsibility is strictly limited to data generation
- No longer responsible for transport or I/O

---

### 3. Frame builder

- Maintains `f64 → Digimatic frame array` conversion
- No structural changes in this release
- Future candidate for trait-based abstraction (v3+)

---

### 4. pub / private boundary stabilization

#### Core API (stable)

- `frame`
- `parser`
- `measurement`

#### Application layer

- `presentation`
- `logger`
- `config`

#### Internal / restricted

- `sim`
- helper functions (`pub(crate)` or private)

---

### 5. Console / logging

- Console utilities remain distributed
- No consolidation in v2
- Planned for future logging layer unification

---

### 6. Legacy directory policy

The `legacy/` directory is introduced as a project-level convention:

- Stores inactive but preserved implementations
- Not part of runtime build logic
- Used for historical I/O and alternative pipelines

---

## 🧪 Behavior changes

- No functional changes to measurement output
- No changes to parsing results
- No changes to GUI behavior
- System behavior is fully backward compatible

---

## 🧭 Design intent

This release is intentionally structural:

- Reduce architectural ambiguity
- Remove hidden I/O dependencies
- Prepare for future extensibility (binary frame, OS abstraction)
- Stabilize data flow model

---

## 🔮 Future direction (v3+)

- Binary frame generation abstraction (trait-based)
- Virtual port layer redesign (cross-platform I/O abstraction)
- Logging / console unification
- GUI / CLI presentation unification layer
- Potential re-evaluation of `frame.rs` structure granularity

---

## ✅ Conclusion

v2.0.0 represents a **stabilization milestone**:

- Architecture is now explicitly layered
- Legacy systems are isolated
- Core data flow is deterministic and unified

This version establishes the foundation for all future extensions.