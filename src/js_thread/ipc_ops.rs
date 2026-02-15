use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;

use crate::ipc::{
    JsCommand, JsCommandSender, LogLevel, UiEvent, UiEventReceiver, WidgetActionKind, WidgetKind,
};

/// Wrapper so we can store the UiEventReceiver in OpState (needs Arc<Mutex<>> for spawn_blocking)
pub struct SharedEventReceiver(pub Arc<Mutex<UiEventReceiver>>);

fn send_command(state: &mut OpState, cmd: JsCommand) -> Result<(), JsErrorBox> {
    let sender = state.borrow::<JsCommandSender>();
    sender
        .send(cmd)
        .map_err(|e| JsErrorBox::generic(format!("IPC send failed: {}", e)))
}

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
    #[string] text: Option<String>,
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
            text,
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

/// Wait for the next UI event. Blocks until an event arrives.
/// Returns a JSON string representing the event, or null if the channel is disconnected.
#[op2]
#[string]
pub async fn op_wait_for_event(state: Rc<RefCell<OpState>>) -> Result<String, JsErrorBox> {
    let receiver = {
        let state = state.borrow();
        let shared = state.borrow::<SharedEventReceiver>();
        shared.0.clone()
    };

    let event = tokio::task::spawn_blocking(move || {
        let rx = receiver.lock().unwrap();
        rx.recv()
    })
    .await
    .map_err(|e| JsErrorBox::generic(format!("spawn_blocking failed: {}", e)))?;

    match event {
        Ok(event) => Ok(serialize_event(&event)),
        Err(_) => Ok(r#"{"type":"disconnected"}"#.to_string()),
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
                WidgetActionKind::Click => "click".to_string(),
                WidgetActionKind::DoubleClick => "doubleClick".to_string(),
                WidgetActionKind::TextChanged(t) => {
                    format!(r#"textChanged","value":"{}""#, escape_json_string(t))
                }
                WidgetActionKind::ValueChanged(v) => {
                    format!(r#"valueChanged","value":{}"#, v)
                }
                WidgetActionKind::Custom(c) => {
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

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

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
    esm = ["ext:appjs_ipc/runtime.js" = "src/js_thread/appjs.js"],
);
