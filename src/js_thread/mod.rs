// JS Thread Module
// Handles the JavaScript runtime execution using deno_core

mod console_ops;
pub mod event_serializer;
pub mod ipc_ops;
pub mod style_parser;

use std::sync::{Arc, Mutex};

use deno_core::{JsRuntime, RuntimeOptions};

use crate::ipc::{JsCommand, JsThreadChannels, LogLevel};

/// Configuration for the JS runtime
pub struct JsRuntimeConfig {
    /// Path to the bundled JavaScript file to execute
    pub script_path: String,
}

impl Default for JsRuntimeConfig {
    fn default() -> Self {
        Self {
            script_path: "./main.js".to_string(),
        }
    }
}

/// Run the JS runtime on a background thread
///
/// This function creates a new tokio runtime and runs the JS event loop.
/// It should be called from `std::thread::spawn`.
pub fn run_js_thread(channels: JsThreadChannels, config: JsRuntimeConfig) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    rt.block_on(async move {
        if let Err(e) = run_js_runtime(channels, config).await {
            eprintln!("[JS] Runtime error: {:?}", e);
        }
    });
}

/// The async inner function that sets up and runs the JS runtime
async fn run_js_runtime(
    channels: JsThreadChannels,
    config: JsRuntimeConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command_sender = channels.command_sender;
    let event_receiver = channels.event_receiver;

    // Helper to log via IPC
    let log = |msg: &str| {
        let _ = command_sender.send(JsCommand::Log {
            level: LogLevel::Info,
            message: msg.to_string(),
        });
    };

    log("Initializing JS runtime...");

    let script_path = std::path::Path::new(&config.script_path);
    let script_specifier = deno_core::resolve_path(
        script_path.to_string_lossy().as_ref(),
        &std::env::current_dir()?,
    )
    .map_err(|e| format!("Invalid script path '{}': {}", config.script_path, e))?;
    let script_source = std::fs::read_to_string(script_path)
        .map_err(|e| format!("Failed to read script '{}': {}", config.script_path, e))?;

    log(&format!("Executing script: {}", script_specifier));

    // Prepare the IPC extension with state injected into OpState
    let shared_receiver = ipc_ops::SharedEventReceiver(Arc::new(Mutex::new(event_receiver)));
    let sender_for_state = command_sender.clone();

    let mut ipc_ext = ipc_ops::appjs_ipc::init();
    ipc_ext.op_state_fn = Some(Box::new(move |state| {
        state.put(sender_for_state);
        state.put(shared_receiver);
    }));

    // Create the deno_core JsRuntime with IPC extensions only.
    // App dev setup is expected to provide a pre-bundled JavaScript file.
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![console_ops::appjs_console::init(), ipc_ext],
        ..Default::default()
    });

    log("JS runtime initialized, executing script...");

    runtime
        .execute_script(script_specifier.to_string(), script_source)
        .map_err(|e| format!("Script execution error ({}): {}", config.script_path, e))?;

    // Run the event loop to process async ops
    // (including the event listener loop if the user registered any listeners via appjs.events.on())
    runtime
        .run_event_loop(Default::default())
        .await
        .map_err(|e| format!("Event loop error: {}", e))?;

    log("JS runtime finished");

    Ok(())
}
