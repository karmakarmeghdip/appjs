// IPC ops for the JS runtime
// Provides the bridge between JavaScript and the UI thread via IPC channels
//
// Command ops (sync): send JsCommand variants through JsCommandSender in OpState
// Event listener op (async): blocks until a UiEvent arrives from the UI thread

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;

use crate::ipc::{JsCommand, JsCommandSender, LogLevel, UiEvent, UiEventReceiver, WidgetKind};

// ============================================================================
// Wrappers for storing IPC channels in OpState
// ============================================================================

/// Wrapper so we can store the UiEventReceiver in OpState (needs Arc<Mutex<>> for spawn_blocking)
pub struct SharedEventReceiver(pub Arc<Mutex<UiEventReceiver>>);

// ============================================================================
// Helper: send a command via OpState
// ============================================================================

fn send_command(state: &mut OpState, cmd: JsCommand) -> Result<(), JsErrorBox> {
    let sender = state.borrow::<JsCommandSender>();
    sender
        .send(cmd)
        .map_err(|e| JsErrorBox::generic(format!("IPC send failed: {}", e)))
}

// ============================================================================
// Command ops (synchronous)
// ============================================================================

/// Set the window title
#[op2(fast)]
pub fn op_set_title(state: &mut OpState, #[string] title: &str) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::SetTitle(title.to_string()))
}

/// Create a widget
#[op2]
pub fn op_create_widget(
    state: &mut OpState,
    #[string] id: &str,
    #[string] kind: &str,
    #[string] parent_id: Option<String>,
) -> Result<(), JsErrorBox> {
    let widget_kind = match kind {
        "Label" | "label" => WidgetKind::Label,
        "Button" | "button" => WidgetKind::Button,
        "TextInput" | "textInput" | "text_input" => WidgetKind::TextInput,
        "TextArea" | "textArea" | "text_area" => WidgetKind::TextArea,
        "Container" | "container" => WidgetKind::Container,
        "Flex" | "flex" => WidgetKind::Flex,
        other => WidgetKind::Custom(other.to_string()),
    };
    send_command(
        state,
        JsCommand::CreateWidget {
            id: id.to_string(),
            kind: widget_kind,
            parent_id,
        },
    )
}

/// Remove a widget
#[op2(fast)]
pub fn op_remove_widget(state: &mut OpState, #[string] id: &str) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::RemoveWidget { id: id.to_string() })
}

/// Set widget text content
#[op2(fast)]
pub fn op_set_widget_text(
    state: &mut OpState,
    #[string] id: &str,
    #[string] text: &str,
) -> Result<(), JsErrorBox> {
    send_command(
        state,
        JsCommand::SetWidgetText {
            id: id.to_string(),
            text: text.to_string(),
        },
    )
}

/// Set widget visibility
#[op2(fast)]
pub fn op_set_widget_visible(
    state: &mut OpState,
    #[string] id: &str,
    visible: bool,
) -> Result<(), JsErrorBox> {
    send_command(
        state,
        JsCommand::SetWidgetVisible {
            id: id.to_string(),
            visible,
        },
    )
}

/// Resize the window
#[op2(fast)]
pub fn op_resize_window(state: &mut OpState, width: u32, height: u32) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::ResizeWindow { width, height })
}

/// Close the window
#[op2(fast)]
pub fn op_close_window(state: &mut OpState) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::CloseWindow)
}

/// Exit the application
#[op2(fast)]
pub fn op_exit_app(state: &mut OpState) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::ExitApp)
}

/// Log a message at a given level
#[op2(fast)]
pub fn op_log(
    state: &mut OpState,
    #[string] level: &str,
    #[string] message: &str,
) -> Result<(), JsErrorBox> {
    let log_level = match level {
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
    };
    send_command(
        state,
        JsCommand::Log {
            level: log_level,
            message: message.to_string(),
        },
    )
}

// ============================================================================
// Event listener op (async)
// ============================================================================

/// Wait for the next UI event. Blocks until an event arrives.
/// Returns a JSON string representing the event, or null if the channel is disconnected.
#[op2]
#[string]
pub async fn op_wait_for_event(state: Rc<RefCell<OpState>>) -> Result<String, JsErrorBox> {
    // Clone the Arc<Mutex<Receiver>> so we can move it into spawn_blocking
    let receiver = {
        let state = state.borrow();
        let shared = state.borrow::<SharedEventReceiver>();
        shared.0.clone()
    };

    // Block on the receiver in a separate thread to avoid blocking the tokio runtime
    let event = tokio::task::spawn_blocking(move || {
        let rx = receiver.lock().unwrap();
        rx.recv()
    })
    .await
    .map_err(|e| JsErrorBox::generic(format!("spawn_blocking failed: {}", e)))?;

    match event {
        Ok(event) => Ok(serialize_event(&event)),
        Err(_) => {
            // Channel disconnected -- UI thread is gone
            Ok(r#"{"type":"disconnected"}"#.to_string())
        }
    }
}

/// Serialize a UiEvent to JSON string for JavaScript consumption
fn serialize_event(event: &UiEvent) -> String {
    match event {
        UiEvent::WindowResized { width, height } => {
            format!(
                r#"{{"type":"windowResized","width":{},"height":{}}}"#,
                width, height
            )
        }
        UiEvent::MouseClick { x, y } => {
            format!(r#"{{"type":"mouseClick","x":{},"y":{}}}"#, x, y)
        }
        UiEvent::MouseMove { x, y } => {
            format!(r#"{{"type":"mouseMove","x":{},"y":{}}}"#, x, y)
        }
        UiEvent::KeyPress { key, modifiers } => {
            format!(
                r#"{{"type":"keyPress","key":"{}","shift":{},"ctrl":{},"alt":{},"meta":{}}}"#,
                escape_json_string(key),
                modifiers.shift,
                modifiers.ctrl,
                modifiers.alt,
                modifiers.meta,
            )
        }
        UiEvent::KeyRelease { key, modifiers } => {
            format!(
                r#"{{"type":"keyRelease","key":"{}","shift":{},"ctrl":{},"alt":{},"meta":{}}}"#,
                escape_json_string(key),
                modifiers.shift,
                modifiers.ctrl,
                modifiers.alt,
                modifiers.meta,
            )
        }
        UiEvent::TextInput { text } => {
            format!(
                r#"{{"type":"textInput","text":"{}"}}"#,
                escape_json_string(text)
            )
        }
        UiEvent::WidgetAction { widget_id, action } => {
            let action_str = match action {
                crate::ipc::WidgetActionKind::Click => "click".to_string(),
                crate::ipc::WidgetActionKind::DoubleClick => "doubleClick".to_string(),
                crate::ipc::WidgetActionKind::TextChanged(t) => {
                    format!(r#"textChanged","value":"{}""#, escape_json_string(t))
                }
                crate::ipc::WidgetActionKind::ValueChanged(v) => {
                    format!(r#"valueChanged","value":{}"#, v)
                }
                crate::ipc::WidgetActionKind::Custom(c) => {
                    format!(r#"custom","value":"{}""#, escape_json_string(c))
                }
            };
            format!(
                r#"{{"type":"widgetAction","widgetId":"{}","action":"{}"}}"#,
                escape_json_string(widget_id),
                action_str,
            )
        }
        UiEvent::WindowFocusChanged { focused } => {
            format!(r#"{{"type":"windowFocusChanged","focused":{}}}"#, focused)
        }
        UiEvent::WindowCloseRequested => r#"{"type":"windowCloseRequested"}"#.to_string(),
        UiEvent::AppExit => r#"{"type":"appExit"}"#.to_string(),
    }
}

/// Escape special characters in a JSON string value
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// ============================================================================
// Extension registration
// ============================================================================

deno_core::extension!(
    appjs_ipc,
    ops = [
        op_set_title,
        op_create_widget,
        op_remove_widget,
        op_set_widget_text,
        op_set_widget_visible,
        op_resize_window,
        op_close_window,
        op_exit_app,
        op_log,
        op_wait_for_event,
    ],
    esm_entry_point = "ext:appjs_ipc/runtime.js",
    esm = ["ext:appjs_ipc/runtime.js" = {
        source = r#"
// AppJS IPC Bridge -- JavaScript API
// Exposes globalThis.appjs for controlling the UI and listening for events
const core = globalThis.Deno.core;

// ============================================================
// Event emitter internals
// ============================================================
const _listeners = {};
let _eventLoopRunning = false;

function _dispatch(eventJson) {
    const event = JSON.parse(eventJson);
    const type = event.type;
    if (!type) return;

    const handlers = _listeners[type];
    if (handlers) {
        for (const handler of handlers) {
            try {
                handler(event);
            } catch (err) {
                console.error(`[appjs] Error in '${type}' handler:`, err);
            }
        }
    }

    // Also dispatch to wildcard listeners
    const wildcardHandlers = _listeners["*"];
    if (wildcardHandlers) {
        for (const handler of wildcardHandlers) {
            try {
                handler(event);
            } catch (err) {
                console.error("[appjs] Error in wildcard handler:", err);
            }
        }
    }
}

async function _startEventLoop() {
    if (_eventLoopRunning) return;
    _eventLoopRunning = true;

    while (_eventLoopRunning) {
        try {
            const eventJson = await core.ops.op_wait_for_event();
            if (!eventJson) {
                _eventLoopRunning = false;
                break;
            }

            const parsed = JSON.parse(eventJson);
            if (parsed.type === "disconnected") {
                _eventLoopRunning = false;
                break;
            }

            _dispatch(eventJson);
        } catch (err) {
            console.error("[appjs] Event loop error:", err);
            _eventLoopRunning = false;
            break;
        }
    }
}

// ============================================================
// Public API: globalThis.appjs
// ============================================================
globalThis.appjs = {
    // ---- Window management ----
    window: {
        setTitle: (title) => core.ops.op_set_title(title),
        resize: (width, height) => core.ops.op_resize_window(width, height),
        close: () => core.ops.op_close_window(),
    },

    // ---- UI / Widget management ----
    ui: {
        createWidget: (id, kind, parentId) =>
            core.ops.op_create_widget(id, kind, parentId ?? null),
        removeWidget: (id) => core.ops.op_remove_widget(id),
        setWidgetText: (id, text) => core.ops.op_set_widget_text(id, text),
        setWidgetVisible: (id, visible) => core.ops.op_set_widget_visible(id, visible),
    },

    // ---- Event system ----
    events: {
        /**
         * Register a listener for a UI event type.
         * Supported types: windowResized, mouseClick, mouseMove, keyPress,
         *   keyRelease, textInput, widgetAction, windowFocusChanged,
         *   windowCloseRequested, appExit
         * Use "*" to listen for all events.
         *
         * @param {string} type - Event type name
         * @param {function} callback - Handler function receiving the event object
         * @returns {function} unsubscribe function
         */
        on: (type, callback) => {
            if (!_listeners[type]) {
                _listeners[type] = [];
            }
            _listeners[type].push(callback);

            // Auto-start the event loop on first listener registration
            if (!_eventLoopRunning) {
                _startEventLoop();
            }

            // Return unsubscribe function
            return () => {
                const handlers = _listeners[type];
                if (handlers) {
                    const idx = handlers.indexOf(callback);
                    if (idx >= 0) handlers.splice(idx, 1);
                }
            };
        },

        /**
         * Remove all listeners for a specific event type, or all listeners.
         * @param {string} [type] - If provided, only remove listeners for this type
         */
        off: (type) => {
            if (type) {
                delete _listeners[type];
            } else {
                for (const key of Object.keys(_listeners)) {
                    delete _listeners[key];
                }
            }
        },
    },

    // ---- Logging ----
    log: {
        debug: (msg) => core.ops.op_log("debug", String(msg)),
        info: (msg) => core.ops.op_log("info", String(msg)),
        warn: (msg) => core.ops.op_log("warn", String(msg)),
        error: (msg) => core.ops.op_log("error", String(msg)),
    },

    // ---- App lifecycle ----
    exit: () => core.ops.op_exit_app(),
};
"#
    }],
);
