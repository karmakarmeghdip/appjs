use std::sync::mpsc::TryRecvError;

use masonry::core::{ErasedAction, WidgetId};
use masonry::widgets::ButtonPress;
use masonry_winit::app::{AppDriver, DriverCtx, WindowId};

use super::handler::handle_js_command;
use crate::ipc::{JsCommandReceiver, UiEvent, UiEventSender, WidgetActionKind};

/// The main application driver that handles UI events and commands
pub struct AppJsDriver {
    /// The main window ID
    pub window_id: WindowId,
    /// Channel to send UI events to JS thread
    event_sender: UiEventSender,
    /// Channel to receive commands from JS thread
    command_receiver: JsCommandReceiver,
}

impl AppJsDriver {
    /// Create a new AppJsDriver with the given channels
    pub fn new(
        window_id: WindowId,
        event_sender: UiEventSender,
        command_receiver: JsCommandReceiver,
    ) -> Self {
        Self {
            window_id,
            event_sender,
            command_receiver,
        }
    }

    /// Process any pending commands from the JS thread
    fn process_js_commands(&mut self, _ctx: &mut DriverCtx<'_, '_>) {
        loop {
            match self.command_receiver.try_recv() {
                Ok(command) => {
                    handle_js_command(self, command);
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    eprintln!("JS thread disconnected");
                    break;
                }
            }
        }
    }

    /// Send a UI event to the JS thread
    fn send_event(&self, event: UiEvent) {
        if let Err(e) = self.event_sender.send(event) {
            eprintln!("Failed to send UI event: {}", e);
        }
    }
}

impl AppDriver for AppJsDriver {
    fn on_action(
        &mut self,
        window_id: WindowId,
        ctx: &mut DriverCtx<'_, '_>,
        widget_id: WidgetId,
        action: ErasedAction,
    ) {
        debug_assert_eq!(window_id, self.window_id, "unknown window");

        // Process any pending JS commands
        self.process_js_commands(ctx);

        // Handle widget actions
        if action.is::<ButtonPress>() {
            println!("[UI] Button pressed: {:?}", widget_id);
            self.send_event(UiEvent::WidgetAction {
                widget_id: format!("{:?}", widget_id),
                action: WidgetActionKind::Click,
            });
        } else {
            eprintln!("Unexpected action {:?}", action);
        }
    }
}
