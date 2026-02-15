use masonry::core::{ErasedAction, WidgetId};
use masonry_winit::app::{AppDriver, DriverCtx, WindowId};

use crate::ipc::{JsCommandAction, UiEvent, UiEventSender, WidgetActionKind};

use super::handler::{WidgetManager, handle_js_command};

/// Application driver that bridges JS runtime commands with the masonry UI.
///
/// When on_action is called with a JsCommandAction (sent via EventLoopProxy from the JS thread),
/// it mutates the widget tree to create, update, or remove widgets.
pub struct AppJsDriver {
    /// The window ID for the main application window
    pub window_id: WindowId,
    /// Sender for UI events back to the JS thread
    pub event_sender: UiEventSender,
    /// Manages JS widget ID → masonry WidgetId mapping
    pub widget_manager: WidgetManager,
}

impl AppJsDriver {
    pub fn new(window_id: WindowId, event_sender: UiEventSender) -> Self {
        Self {
            window_id,
            event_sender,
            widget_manager: WidgetManager::new(),
        }
    }
}

impl AppDriver for AppJsDriver {
    fn on_action(
        &mut self,
        window_id: WindowId,
        ctx: &mut DriverCtx<'_, '_>,
        _widget_id: WidgetId,
        action: ErasedAction,
    ) {
        // Check if this action is a JsCommandAction sent via EventLoopProxy
        if let Some(js_action) = action.downcast_ref::<JsCommandAction>() {
            // Clone the command so we can process it (action is borrowed)
            let cmd = js_action.0.clone();
            let render_root = ctx.render_root(window_id);
            handle_js_command(
                cmd,
                window_id,
                render_root,
                &mut self.widget_manager,
                &self.event_sender,
            );
        } else {
            let type_name = action.type_name();
            // Find which JS widget this is
            let mut js_id = None;
            for (id, info) in &self.widget_manager.widgets {
                if info.widget_id == _widget_id {
                    js_id = Some(id.clone());
                    break;
                }
            }

            if let Some(id) = js_id {
                if type_name.contains("ButtonPress") {
                    let _ = self.event_sender.send(UiEvent::WidgetAction {
                        widget_id: id,
                        action: WidgetActionKind::Click,
                    });
                }
            }

            // Always log the action for debugging
            println!(
                "[UI] Widget action on {:?} in window {:?}: {:?}",
                _widget_id, window_id, type_name
            );
        }
    }
}
