// JS Thread Module
// Handles the JavaScript runtime execution using deno_core

mod console_ops;
pub mod event_serializer;
pub mod ipc_ops;
pub mod style_parser;
mod telemetry_stub;
mod web_bootstrap;

use std::sync::{Arc, Mutex};
use std::rc::Rc;

use deno_runtime::deno_core::{JsRuntime, RuntimeOptions};
use deno_runtime::deno_permissions::{PermissionsContainer, RuntimePermissionDescriptorParser};

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
    let fs = Arc::new(deno_runtime::deno_fs::RealFs);

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
    let permissions = PermissionsContainer::allow_all(Arc::new(
        RuntimePermissionDescriptorParser::new(sys_traits::impls::RealSys),
    ));

    let mut ipc_ext = ipc_ops::appjs_ipc::init();
    ipc_ext.op_state_fn = Some(Box::new(move |state| {
        state.put(sender_for_state);
        state.put(shared_receiver);
        state.put::<PermissionsContainer>(permissions);
    }));

    // Create the runtime with AppJS extensions and selected Deno Web APIs.
    // App dev setup is expected to provide a pre-bundled JavaScript file.
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![
            deno_runtime::deno_webidl::deno_webidl::init(),
            deno_runtime::deno_web::deno_web::init(
                Default::default(),
                Default::default(),
                deno_runtime::deno_web::InMemoryBroadcastChannel::default(),
            ),
            telemetry_stub::appjs_telemetry_stub::init(),
            deno_runtime::deno_net::deno_net::init(None, None),
            deno_runtime::deno_tls::deno_tls::init(),
            deno_runtime::deno_fetch::deno_fetch::init(Default::default()),
            deno_runtime::deno_cache::deno_cache::init(None),
            deno_runtime::deno_websocket::deno_websocket::init(),
            deno_runtime::deno_webstorage::deno_webstorage::init(None),
            deno_runtime::deno_crypto::deno_crypto::init(None),
            deno_runtime::deno_ffi::deno_ffi::init(None),
            deno_runtime::deno_napi::deno_napi::init(None),
            deno_runtime::deno_http::deno_http::init(Default::default()),
            deno_runtime::ops::tty::deno_tty::init(),
            deno_runtime::deno_io::deno_io::init(Some(Default::default())),
            deno_runtime::deno_fs::deno_fs::init(fs.clone()),
            deno_runtime::deno_os::deno_os::init(Default::default()),
            deno_runtime::deno_process::deno_process::init(Default::default()),
            deno_node_crypto::deno_node_crypto::init(),
            deno_node_sqlite::deno_node_sqlite::init(),
            deno_runtime::deno_node::deno_node::init::<
                deno_resolver::npm::DenoInNpmPackageChecker,
                deno_resolver::npm::NpmResolver<sys_traits::impls::RealSys>,
                sys_traits::impls::RealSys,
            >(None, fs.clone()),
            deno_runtime::ops::runtime::deno_runtime::init(script_specifier.clone()),
            deno_bundle_runtime::deno_bundle_runtime::init(None),
            deno_runtime::shared::runtime::init(),
            ipc_ext,
            web_bootstrap::appjs_web_bootstrap::init(),
        ],
        extension_transpiler: Some(Rc::new(|specifier, source| {
            deno_runtime::transpile::maybe_transpile_source(specifier, source)
        })),
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
