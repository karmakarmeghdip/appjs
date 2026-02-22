# Vellum UI: Codebase Architecture & Data Flow Guide

Welcome to Vellum UI! This document is an extensive contributor guide explaining the entire codebase file by file, function by function. It tracks how cross-process messages map from user input, through the Rust UI layer, onto the underlying JS runtime, and back.

---

## High-Level Execution Flow

Vellum UI functions on a strict **dual-process architecture**:
1. **The Native UI Process (Rust)**: Runs the Window loop, GPU rendering (`wgpu`/`vello`), and the `masonry` widget tree.
2. **The Client Runtime Process (Bun/JS)**: Runs user scripts, component states (e.g. SolidJS), and dispatches IPC messages to update the native UI layout.

### Sub-Thread Breakdown (Rust Base)

Inside the Rust binary, there are two primary threads running concurrently:
- **`Main UI Thread`**: Exclusively blocked by the `winit` event loop handling the physical window.
- **`IPC Server Thread`**: A background Rust thread spawned by `main.rs` whose sole purpose is to bind to a Unix Domain Socket (or Named Pipe on Windows) and communicate with the client process.

---

## The Bootstrapping Sequence (Step-by-Step)

When a developer runs `Vellum UI`, the initialization cascade looks like this:

### 1. Bootstrapping the JS Client (`packages/core/src/bun_bridge.ts`)

The user runs their application with `bun run <script.ts>`.
- When the developer's script hits `import { window } from "@vellum/core"`, the `bun_bridge.ts` bootstrap module evaluates immediately.
- **Function**: `bun_bridge.ts` generates a temporary Unix Domain Socket (UDS) path (e.g., `/tmp/Vellum_<uuid>.sock`).
- **Function**: `spawn()` is called to physically launch the `vellum` Rust binary as a subprocess, passing the socket path via the `VELLUM_SOCKET` environment variable.

### 2. Rust Application Starts (`src/main.rs`)
- The Rust application hits `main()`.
- **Function**: `std::sync::mpsc::channel()` is called to create two queues:
   - `UiEventReceiver`: A queue where the UI Thread sends physical events (Clicks, etc.) to the IPC Bridge.
   - `ClientCommandReceiver`: A queue via `EventLoopProxy` where the IPC Bridge sends remote commands (CreateWidget) to the UI Thread.
- **Function**: `std::thread::spawn(move || { crate::ipc::server::run_ipc_server(...) })`. The main thread spins up the independent IPC Bridge Thread and hands it the channels.
- **Function**: `masonry_winit::WindowExt::new()` creates the physical OS Window.
- **Function**: `crate::ui::driver::VellumDriver::new()` is initialized. This is the router trait that handles incoming `winit` physical events.
- **Function**: `EventLoop::run_app()` fires, handing complete control over to the OS. The Main Thread is now trapped in the UI loop.

### 3. The IPC Socket Server (`src/ipc/server.rs`)
While the physical UI initializes, the background Rust thread runs `run_ipc_server`:
- **Function**: `get_socket_path()` reads the `VELLUM_SOCKET` environment variable.
- **Function**: `bind_socket()` opens the UDS/Named Pipe and waits for the client to connect.
- **Function**: `listener.accept()` unblocks when the Bun process successfully connects to the socket.
- **Function**: Two infinite loops begin via threads/channels:
   - **Write Loop**: Checks the `mpsc` channel for `UiEvent`s and writes them directly to the active socket connection.
   - **Read Loop**: (`read_msgpack_frame` thread) Parses incoming MsgPack frames from the socket and decodes them via `rmp_serde` into `ClientMessage` / `ClientCommand`.

### 4. Dispatching a JS Command (`packages/core/src/ops.ts`)
When the JS script wants to do something—like change the window title:
- **JS Function**: `window.setTitle("Hello!")` is called.
- This invokes `ops.ts -> sendCommand("setTitle", "Hello!")`.
- **Function**: `bun_bridge.ts` invokes `msgpackr.pack({ type: "SetTitle", data: "Hello!" })`.
- **Function**: `socket.write()` sends the length-prefixed bytes directly over the UDS connection to Rust.

---

## 5. The IPC Payload Ingestion (Rust Bridge)
- The Rust IPC Bridge (`src/ipc/server.rs` -> `read_msgpack_frame`) awakens because data appeared on the socket.
- **Function**: `rmp_serde::from_slice` successfully matches the packet to the `ClientMessage` enum payload defined in `src/ipc/msgpack.rs`.
- The `ClientMessage` strictly encapsulates a `ClientCommand` (e.g. `ClientCommand::SetTitle`).
- **Function**: The Bridge calls `proxy.send_event(ClientCommandAction(command))`. This signals the physical `winit` event loop on the Main Thread from the background thread.

---

## 6. Main Thread Action Dispatch (`src/ui/driver.rs` & `src/ui/handler.rs`)
- The `winit` event loop receives `winit::event::Event::UserEvent`.
- **Function**: The `EventLoop` kicks this to the trait implementation `VellumDriver::user_event` in `src/ui/driver.rs`.
- **Function**: `driver.rs` extracts the `ClientCommandAction` and calls `crate::ui::handler::handle_client_command(cmd, ...)`.

### Handling The Command (`src/ui/handler.rs`)
This is the core nervous system of the UI mutations.
- **Function**: `handle_client_command` matches on the `ClientCommand` enum.
- In our `SetTitle` example, it calls `render_root.emit_signal(RenderRootSignal::SetTitle(title))`, updating the native OS window.

### Handling Dynamic Widgets (`src/ui/widget_manager.rs`)
What if the command was `ClientCommand::CreateWidget`?
- **Function**: `handler.rs` routes this to `src/ui/creation.rs -> create_and_add_widget`.
- **Function**: Depending on the `WidgetKind` (e.g., `Button`), `creation.rs` maps a physical `masonry::widgets::Button`.
- **Crucial Flow**: Vellum tracks nested parents via string IDs (e.g. JS ID `btn_main`). Masonry uses abstract integer `WidgetId`s.
- **Function**: `WidgetManager::register_widget` is called. The `WidgetManager` inserts the JS ID mapping to Masonry's integer format and updates its `parent_to_children: HashMap<String, Vec<String>>` tracking topology in `O(1)` time.
- The `masonry` node is physically grafted into the mutable `render_root` tree via `Flex::add_child()`.

---

## 7. Closing the Loop: Dispatching Interactions Native -> JS

What happens when the user clicks the physical button on the screen?
- `masonry` handles the physical click and fires an `Action`.
- **Function**: `src/ui/driver.rs -> VellumDriver::on_action` catches the Masonry generic UI action.
- **Function**: `driver.rs` needs the string ID to tell JS. It calls `WidgetManager::find_client_id(masonry_widget_id)` to reverse look up the JS ID (e.g. returning `btn_main`).
- **Function**: It wraps the data into `UiEvent::WidgetAction { widget_id: "btn_main", action: Click }`.
- **Function**: `driver.rs` calls `ui_event_sender.send(event)` directly to the `mpsc` queue.
- The **IPC Bridge Thread**'s event loop unblocks, encodes the `UiEvent` into MsgPack `ServerMessage`, and writes it to the active socket connection.
- `bun_bridge.ts` socket data listener parses the frame, emits the event locally, and the user's `Vellum.events.on("widgetAction", ...)` callback fires.

---

## Module Reference Summary

| Location | Purpose |
|----------|---------|
| `src/main.rs` | Boots threads, creates event loop, channels setup. |
| `src/ipc/channels.rs` | Types for thread-safe cross-thread queues. |
| `src/ipc/msgpack.rs` | Defines exactly what MsgPack binaries traverse the socket connection. |
| `src/ipc/server.rs` | The background bridge routing raw bytes to structured channels. |
| `src/ui/driver.rs` | The handler that physically mutates the OS windows/event pipelines. |
| `src/ui/handler.rs` | Matches abstracted IPC Commands into Native Widget mutations. |
| `src/ui/widget_manager.rs` | O(1) Hierarchy lookup resolving JS UUIDs to Masonry Integers. |
| `packages/core/src/bun_bridge.ts` | The JS bootstrap that spawns Rust, connects to the socket, and mirrors `ipc/server.rs`. |  
