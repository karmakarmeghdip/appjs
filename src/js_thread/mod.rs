// JS Thread Module
// Handles the Deno JavaScript runtime execution

use std::sync::mpsc::TryRecvError;

use crate::ipc::{JsCommand, JsCommandSender, JsThreadChannels, LogLevel, UiEvent, UiEventReceiver};

/// Configuration for the JS runtime
pub struct JsRuntimeConfig {
    /// Path to the main JavaScript/TypeScript module to execute
    pub main_module_path: String,
    /// Whether to allow all permissions (for development)
    pub allow_all_permissions: bool,
}

impl Default for JsRuntimeConfig {
    fn default() -> Self {
        Self {
            main_module_path: "./main.js".to_string(),
            allow_all_permissions: true,
        }
    }
}

/// The JS runtime runner
pub struct JsRuntime {
    /// Channel to receive UI events
    event_receiver: UiEventReceiver,
    /// Channel to send commands to UI thread
    command_sender: JsCommandSender,
    /// Runtime configuration
    #[allow(dead_code)]
    config: JsRuntimeConfig,
}

impl JsRuntime {
    /// Create a new JS runtime with the given channels and configuration
    pub fn new(channels: JsThreadChannels, config: JsRuntimeConfig) -> Self {
        Self {
            event_receiver: channels.event_receiver,
            command_sender: channels.command_sender,
            config,
        }
    }

    /// Send a command to the UI thread
    pub fn send_command(&self, command: JsCommand) {
        if let Err(e) = self.command_sender.send(command) {
            eprintln!("[JS] Failed to send command: {}", e);
        }
    }

    /// Log a message via the UI thread
    pub fn log(&self, level: LogLevel, message: impl Into<String>) {
        self.send_command(JsCommand::Log {
            level,
            message: message.into(),
        });
    }

    /// Process any pending UI events (non-blocking)
    pub fn process_events(&self) -> Vec<UiEvent> {
        let mut events = Vec::new();
        loop {
            match self.event_receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    eprintln!("[JS] UI thread disconnected");
                    break;
                }
            }
        }
        events
    }

    /// Run the JS runtime
    ///
    /// This function is async and should be run within a tokio runtime
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log(LogLevel::Info, "Starting JS runtime thread...");

        // For now, run in standby mode - just process events
        // Full Deno runtime integration will be added later
        self.run_standby_loop().await
    }

    /// Run in standby mode - process events from UI thread
    async fn run_standby_loop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log(LogLevel::Info, "Running in standby mode (Deno runtime not yet initialized)");

        // Simple loop that processes events
        loop {
            let events = self.process_events();

            for event in events {
                match event {
                    UiEvent::AppExit => {
                        self.log(LogLevel::Info, "App exit requested, shutting down");
                        return Ok(());
                    }
                    UiEvent::WindowCloseRequested => {
                        self.log(LogLevel::Info, "Window close requested");
                        return Ok(());
                    }
                    UiEvent::WidgetAction { widget_id, action } => {
                        self.log(
                            LogLevel::Debug,
                            format!("Widget action: {} - {:?}", widget_id, action),
                        );
                        // Echo back a command to demonstrate IPC
                        self.send_command(JsCommand::SetTitle(format!(
                            "AppJS - Button clicked!"
                        )));
                    }
                    _ => {
                        // Process other events
                    }
                }
            }

            // Small sleep to avoid busy-waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
        }
    }
}

/// Run the JS runtime on a background thread
///
/// This function creates a new tokio runtime and runs the JS event loop.
/// It should be called from `std::thread::spawn`.
pub fn run_js_thread(channels: JsThreadChannels, config: JsRuntimeConfig) {
    // Create a new tokio runtime for this thread
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    let js_runtime = JsRuntime::new(channels, config);

    // Run the JS runtime
    runtime.block_on(async {
        if let Err(e) = js_runtime.run().await {
            eprintln!("[JS] Runtime error: {:?}", e);
        }
    });
}
