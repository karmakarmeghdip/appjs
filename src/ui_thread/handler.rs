use super::driver::AppJsDriver;
use crate::ipc::{JsCommand, LogLevel};

/// Handle a single command from the JS thread
pub fn handle_js_command(_driver: &mut AppJsDriver, command: JsCommand) {
    match command {
        JsCommand::SetTitle(title) => {
            println!("[UI] Set title: {}", title);
        }
        JsCommand::Log { level, message } => {
            let prefix = match level {
                LogLevel::Debug => "[DEBUG]",
                LogLevel::Info => "[INFO]",
                LogLevel::Warn => "[WARN]",
                LogLevel::Error => "[ERROR]",
            };
            println!("{} {}", prefix, message);
        }
        JsCommand::CreateWidget {
            id,
            kind,
            parent_id,
        } => {
            println!(
                "[UI] Create widget: id={}, kind={:?}, parent={:?}",
                id, kind, parent_id
            );
        }
        JsCommand::UpdateWidget { id, updates } => {
            println!("[UI] Update widget: id={}, updates={:?}", id, updates);
        }
        JsCommand::RemoveWidget { id } => {
            println!("[UI] Remove widget: id={}", id);
        }
        JsCommand::SetWidgetText { id, text } => {
            println!("[UI] Set widget text: id={}, text={}", id, text);
        }
        JsCommand::SetWidgetVisible { id, visible } => {
            println!("[UI] Set widget visible: id={}, visible={}", id, visible);
        }
        JsCommand::ResizeWindow { width, height } => {
            println!("[UI] Resize window: {}x{}", width, height);
        }
        JsCommand::CloseWindow => {
            println!("[UI] Close window requested");
        }
        JsCommand::ExitApp => {
            println!("[UI] Exit app requested");
        }
    }
}
