# AGENTS.md

This file establishes the context, architectural decisions, and development
guidelines for AI agents and developers working on the `vellum` repository.

## 1. Project Overview

`vellum` is a high-performance cross-language GUI engine designed for
building Desktop Applications. It leverages Rust for native capabilities and
runs application logic (like JavaScript/TypeScript/SolidJS) in an external process via IPC.

### Core Philosophy

- **Native Performance**: Critical UI and system operations run in Rust.
- **Web Flexibility**: Application logic and UI definitions are driven by JS/TS.
- **Thread Safety**: Strict separation of concerns via a dual-threaded model.

### Architecture

The application implements a strict **Dual-Threaded Architecture**:

1. **Main Thread (UI Thread)**
   - **Responsibility**: Owns the `winit` Window and Event Loop. Manages the
     `masonry` Widget tree and rendering.
   - **Constraints**: NEVER perform blocking I/O or heavy computation here.
     Freezing this thread freezes the application window.
   - **Libraries**: `winit`, `masonry`, `masonry_winit`.

2. **Background Process (Client Runtime)**
   - **Responsibility**: Hosts the external client runtime (e.g., Bun). Executes all application
     code, manages state, and handles business logic.
   - **Constraints**: Cannot access UI objects directly. Must send IPC messages to
     the Main Thread to mutate the UI.
   - **Behavior**: The client runtime actually spawns the Rust `vellum` binary as a child process and connects to it over a Unix Domain Socket (or Named Pipe on Windows).

3. **Communication Bridge**
   - Communication is asynchronous and message-based.
   - **Mechanism**: `std::sync::mpsc` for in-process UI event capture + MsgPack
     frames over UDS/Named Pipes for cross-process communication.
   - **Direction 1 (UI -> Client)**: User interactions (clicks, scroll, type) are
     captured by `winit`, converted to `UiEvent`s, encoded as MsgPack, and
     streamed to the client via socket.
   - **Direction 2 (Client -> UI)**: Client logic sends MsgPack command messages that
     are mapped to `ClientCommand` and dispatched through `EventLoopProxy`.

## 2. Build, Test, and Run Commands

Standard Cargo workflows apply. Ensure you are in the project root.

### Build

Compile the project in debug mode:

```bash
cargo build
```

Compile for release (optimized):

```bash
cargo build --release
```

### Run

Run the application (Debug):

```bash
cargo run
```

_Note: This will launch the application window._

### Testing

Run the full test suite:

```bash
cargo test
```

**Run a Single Test**: To run a specific test case (e.g.,
`test_channel_communication`):

```bash
cargo test test_channel_communication -- --nocapture
```

- `--nocapture`: Displays `println!` output, essential for debugging async
  channel tests.

**CRITICAL RULE FOR TESTING CONVENTIONS**:
Every new core component or functional module (especially in `src/ui/` and `src/ipc/`) MUST have exhaustive inline unit tests placed at the bottom of the file inside a `#[cfg(test)]` module. This keeps test coverage closely aligned with the source implementation. Do not create separate `tests/` integration folders for unit-level behavior unless doing E2E. Ensure coverage for parsers, state managers, and serialization boundaries.

**CRITICAL RULE FOR AGENTS**: 
Whenever you modify Rust (`.rs`) files, you **MUST** run `cargo build` prior to testing or running any external JavaScript/SolidJS examples via `bun`. `cargo check` only verifies types; external scripts spawn the physical compiled executable, which will be stale if you only run `cargo check`. Overlooking `cargo build` will lead to debugging phantom errors on outdated binaries.

### Linting & Formatting

Ensure code quality before submitting changes.

**Linting**:

```bash
cargo clippy -- -D warnings
```

- Fix all warnings. Clippy captures idiomatic Rust issues that the compiler
  might miss.

**Formatting**:

```bash
cargo fmt
```

- Standard Rust formatting is mandatory.

## 3. Code Style & Guidelines

### Rust Conventions

- **Style**: Follow standard Rust naming conventions.
  - Types (`struct`, `enum`, `trait`): `PascalCase`
  - Functions, Methods, Variables, Modules: `snake_case`
  - Constants/Statics: `SCREAMING_SNAKE_CASE`
- **Imports**: Organize imports logically.
  ```rust
  // 1. Std lib
  use std::sync::mpsc;
  use std::thread;

  // 2. External crates
  use rmp_serde;
  use winit::event::Event;

  // 3. Local modules
  use crate::bridge::UiEvent;
  ```
- **Error Handling**:
  - Use `Result<T, E>` for recoverable errors.
  - Avoid `unwrap()` in production code. Use `expect("Reason")` if you are 100%
    sure, or better yet, handle the `Err` case.
  - Propagate errors using the `?` operator.

### Architectural Patterns

#### The Event Loop & Channels

When implementing features, always consider the flow of data across the thread
boundary.

**1. Defining Events** Create strong types for messages. Do not send raw
strings.

```rust
// src/events.rs (Example)
pub enum UiEvent {
    WindowResized { width: u32, height: u32 },
    MouseClick { x: f64, y: f64 },
}

pub enum ClientCommand {
    SetTitle(String),
    CreateWidget { id: String, kind: String },
}
```

**2. The UI Loop (Main)** Inside the `winit` event loop:

- **Poll**: Check the receiver channel for `ClientCommand` messages non-blocking
  (e.g., `try_recv`).
- **Dispatch**: Apply valid commands to the `masonry` widget tree.
- **Send**: Convert `winit` events to `UiEvent` and send to the client process.

**3. The Client Loop (Background)**

- The external runtime (e.g. Bun) generates a socket path, spawns the `vellum` Rust binary, and connects to the socket.
- Rust forwards `UiEvent`s to the runtime via MsgPack frames through the socket.
- Rust reads MsgPack command frames from the socket and converts them to `ClientCommand`.

### Binding Specifics

- Keep protocol messages in `src/ipc/msgpack.rs` synchronized with client wrappers (e.g.
  `packages/core/src/bun_bridge.ts`).
- Use length-prefixed MsgPack frames for robust streaming over the UDS connection.

## 4. Feature Implementation Checklist

When tasked with adding a new feature (e.g., "Add a button that logs to
console"):

1. **Plan the Message**: Add `ButtonClicked` to `UiEvent`.
2. **Update UI (Rust)**: Add the Button widget in `masonry`.
3. **Wire Event (Rust)**: In the UI thread, catch the button click and send
   `UiEvent::ButtonClicked`.
4. **Handle in JS (Rust/JS)**: Ensure the JS runtime receives this event and
   triggers a callback.
5. **Verify**: Run `cargo test` and `cargo run` to interact with the feature.

## 5. Environment & Tooling rules

- **Cursor/Copilot**:
  - When generating code, always prioritize type safety.
  - If you generate a `match` statement, ensure all arms are covered.
  - Do not hallucinate external crate features. Check `Cargo.toml` versions.
  - When editing `main.rs`, preserve the thread setup boilerplate unless
    explicitly asked to refactor the core architecture.

## 7. External Resources & Reference

- **Masonry Examples**: Masonry is part of the Xilem project. Search for usage
  patterns in the `linebender/xilem` repository, specifically under
  `masonry/examples`.
- **Bun Runtime**: Refer to `bun.sh/docs` for runtime behavior and Node-compat
  APIs.

## 8. Internal Documentation

For more detailed architectural overviews and refactoring logs, refer to the
`docs/` directory:

- **[Codebase Structure & Architecture](docs/architecture.md)**: A comprehensive
  guide to the folder structure, file responsibilities, and thread interactions.
