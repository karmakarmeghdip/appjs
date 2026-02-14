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

    // Parse CLI arguments: expect a JS/TS file path as the first argument
    let args: Vec<String> = std::env::args().collect();
    let script_path = match args.get(1) {
        Some(path) => path.clone(),
        None => {
            eprintln!("Usage: appjs <script.js|script.ts>");
            eprintln!("  Example: appjs ./app.js");
            std::process::exit(1);
        }
    };

    // Resolve to absolute path
    let script_path = std::path::Path::new(&script_path);
    let absolute_path = match script_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "Error: Cannot resolve script path '{}': {}",
                script_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    println!("[Main] Running script: {}", absolute_path.display());

    // Create IPC channels for communication between threads
    let channels = IpcChannels::new();

    // Extract the channel endpoints for each thread
    let ui_channels = channels.ui_thread;
    let js_channels = channels.js_thread;

    // Configure the JS runtime
    let js_config = JsRuntimeConfig {
        main_module_path: absolute_path.to_string_lossy().to_string(),
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
