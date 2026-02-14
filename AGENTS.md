# AGENTS.md

This file establishes the context, architectural decisions, and development guidelines for AI agents and developers working on the `appjs` repository.

## 1. Project Overview

`appjs` is a high-performance JavaScript/TypeScript Runtime designed for building Desktop Applications. It leverages the power of Rust for native capabilities and Deno for a secure, modern JavaScript environment.

### Core Philosophy
*   **Native Performance**: Critical UI and system operations run in Rust.
*   **Web Flexibility**: Application logic and UI definitions are driven by JS/TS.
*   **Thread Safety**: Strict separation of concerns via a dual-threaded model.

### Architecture

The application implements a strict **Dual-Threaded Architecture**:

1.  **Main Thread (UI Thread)**
    *   **Responsibility**: Owns the `winit` Window and Event Loop. Manages the `masonry` Widget tree and rendering.
    *   **Constraints**: NEVER perform blocking I/O or heavy computation here. Freezing this thread freezes the application window.
    *   **Libraries**: `winit`, `masonry`, `masonry_winit`.

2.  **Background Thread (JS Runtime Thread)**
    *   **Responsibility**: hosts the `deno_runtime` instance. Executes all JavaScript code, manages application state, and handles business logic.
    *   **Constraints**: Cannot access UI objects directly. Must send messages to the Main Thread to mutate the UI.
    *   **Libraries**: `deno_runtime`.
    *   **Note**: `deno_runtime` uses `tokio` internally, but it is **not** exposed. Do not attempt to use `tokio` directly unless you explicitly add it to `Cargo.toml`.

3.  **Communication Bridge**
    *   Communication is asynchronous and message-based.
    *   **Mechanism**: `std::sync::mpsc` channels are the primary transport.
    *   **Direction 1 (UI -> JS)**: User interactions (clicks, scroll, type) are captured by `winit`, converted to `UiEvent`s, and sent to the JS thread.
    *   **Direction 2 (JS -> UI)**: JS logic produces `DomMutation` or `AppCommand` messages, sending them to the UI thread to update the visual state.

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
*Note: This will launch the application window.*

### Testing
Run the full test suite:
```bash
cargo test
```

**Run a Single Test**:
To run a specific test case (e.g., `test_channel_communication`):
```bash
cargo test test_channel_communication -- --nocapture
```
*   `--nocapture`: Displays `println!` output, essential for debugging async channel tests.

### Linting & Formatting
Ensure code quality before submitting changes.

**Linting**:
```bash
cargo clippy -- -D warnings
```
*   Fix all warnings. Clippy captures idiomatic Rust issues that the compiler might miss.

**Formatting**:
```bash
cargo fmt
```
*   Standard Rust formatting is mandatory.

## 3. Code Style & Guidelines

### Rust Conventions

*   **Style**: Follow standard Rust naming conventions.
    *   Types (`struct`, `enum`, `trait`): `PascalCase`
    *   Functions, Methods, Variables, Modules: `snake_case`
    *   Constants/Statics: `SCREAMING_SNAKE_CASE`
*   **Imports**: Organize imports logically.
    ```rust
    // 1. Std lib
    use std::sync::mpsc;
    use std::thread;

    // 2. External crates
    use deno_runtime::deno_core;
    use winit::event::Event;

    // 3. Local modules
    use crate::bridge::UiEvent;
    ```
*   **Error Handling**:
    *   Use `Result<T, E>` for recoverable errors.
    *   Avoid `unwrap()` in production code. Use `expect("Reason")` if you are 100% sure, or better yet, handle the `Err` case.
    *   Propagate errors using the `?` operator.

### Architectural Patterns

#### The Event Loop & Channels
When implementing features, always consider the flow of data across the thread boundary.

**1. Defining Events**
Create strong types for messages. Do not send raw strings.
```rust
// src/events.rs (Example)
pub enum UiEvent {
    WindowResized { width: u32, height: u32 },
    MouseClick { x: f64, y: f64 },
}

pub enum JsCommand {
    SetTitle(String),
    CreateWidget { id: String, kind: String },
}
```

**2. The UI Loop (Main)**
Inside the `winit` event loop:
*   **Poll**: Check the receiver channel for `JsCommand` messages non-blocking (e.g., `try_recv`).
*   **Dispatch**: Apply valid commands to the `masonry` widget tree.
*   **Send**: Convert `winit` events to `UiEvent` and send to the JS thread.

**3. The JS Loop (Background)**
*   Run the Deno event loop.
*   Periodically (or via async await) check for incoming `UiEvent`s.
*   Expose Rust functions to V8 that allow JS to push `JsCommand`s into the sender channel.

### Deno Runtime Specifics
*   **Extensions**: Use `deno_core::Extension` to inject Rust functions into the JS global scope.
*   **Ops**: Use `#[op2]` (if available in the version) or standard ops for high-performance bindings.
*   **Snapshots**: If startup time becomes an issue, consider using a V8 snapshot.

## 4. Feature Implementation Checklist

When tasked with adding a new feature (e.g., "Add a button that logs to console"):

1.  **Plan the Message**: Add `ButtonClicked` to `UiEvent`.
2.  **Update UI (Rust)**: Add the Button widget in `masonry`.
3.  **Wire Event (Rust)**: In the UI thread, catch the button click and send `UiEvent::ButtonClicked`.
4.  **Handle in JS (Rust/JS)**: Ensure the JS runtime receives this event and triggers a callback.
5.  **Verify**: Run `cargo test` and `cargo run` to interact with the feature.

## 5. Environment & Tooling rules

*   **Cursor/Copilot**:
    *   When generating code, always prioritize type safety.
    *   If you generate a `match` statement, ensure all arms are covered.
    *   Do not hallucinate external crate features. Check `Cargo.toml` versions.
    *   When editing `main.rs`, preserve the thread setup boilerplate unless explicitly asked to refactor the core architecture.

## 7. External Resources & Reference

*   **Masonry Examples**: Masonry is part of the Xilem project. Search for usage patterns in the `linebender/xilem` repository, specifically under `masonry/examples`.
*   **Deno Runtime**: Refer to `denoland/deno` for advanced runtime configuration, though strict adherence to `deno_runtime` crate docs is preferred.
