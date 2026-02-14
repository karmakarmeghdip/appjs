// AppJS - JavaScript Desktop Runtime
//
// This application implements a dual-threaded architecture:
// - Main Thread (UI): Owns the window and widget tree via masonry_winit
// - Background Thread (JS): Runs the Deno JavaScript runtime
//
// Communication between threads is handled via std::sync::mpsc channels.

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

mod ipc;
mod js_thread;
mod ui_thread;

use std::thread;

use ipc::IpcChannels;
use js_thread::{JsRuntimeConfig, run_js_thread};
use ui_thread::run_ui;

fn main() {
    // Initialize logging/tracing if needed
    // tracing_subscriber::fmt::init();

    println!("AppJS Starting...");

    // Create IPC channels for communication between threads
    let channels = IpcChannels::new();

    // Extract the channel endpoints for each thread
    let ui_channels = channels.ui_thread;
    let js_channels = channels.js_thread;

    // Configure the JS runtime
    let js_config = JsRuntimeConfig {
        main_module_path: "./main.js".to_string(),
        allow_all_permissions: true,
    };

    // Spawn the JS runtime thread
    // This thread will run the Deno runtime and process JS code
    let js_thread_handle = thread::Builder::new()
        .name("js-runtime".to_string())
        .spawn(move || {
            println!("[Main] JS thread started");
            run_js_thread(js_channels, js_config);
            println!("[Main] JS thread finished");
        })
        .expect("Failed to spawn JS runtime thread");

    // Run the UI on the main thread
    // This blocks until the window is closed
    // The main thread MUST run the UI due to platform requirements (macOS, etc.)
    println!("[Main] Starting UI on main thread");
    run_ui(ui_channels.event_sender, ui_channels.command_receiver);

    // Wait for the JS thread to finish
    // This happens after the UI closes
    println!("[Main] UI closed, waiting for JS thread to finish...");
    if let Err(e) = js_thread_handle.join() {
        eprintln!("[Main] JS thread panicked: {:?}", e);
    }

    println!("[Main] AppJS shutdown complete");
}
