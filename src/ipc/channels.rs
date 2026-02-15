use std::sync::mpsc::{self, Receiver, Sender};

use super::{JsCommand, UiEvent};

/// Sender for UI events (UI thread holds this)
pub type UiEventSender = Sender<UiEvent>;

/// Receiver for UI events (JS thread holds this)
pub type UiEventReceiver = Receiver<UiEvent>;

/// Sender for JS commands (JS thread holds this)
pub type JsCommandSender = Sender<JsCommand>;

/// Receiver for JS commands (UI thread holds this)
pub type JsCommandReceiver = Receiver<JsCommand>;

/// Contains all channel endpoints needed for IPC
pub struct IpcChannels {
    /// Endpoints for the UI thread
    pub ui_thread: UiThreadChannels,
    /// Endpoints for the JS thread
    pub js_thread: JsThreadChannels,
}

/// Channel endpoints held by the UI thread
pub struct UiThreadChannels {
    /// Send UI events to JS thread
    pub event_sender: UiEventSender,
    /// Receive commands from JS thread
    pub command_receiver: JsCommandReceiver,
}

/// Channel endpoints held by the JS thread
pub struct JsThreadChannels {
    /// Receive UI events from UI thread
    pub event_receiver: UiEventReceiver,
    /// Send commands to UI thread
    pub command_sender: JsCommandSender,
}

impl IpcChannels {
    /// Create a new pair of IPC channels for communication between threads
    pub fn new() -> Self {
        let (ui_event_tx, ui_event_rx) = mpsc::channel::<UiEvent>();
        let (js_command_tx, js_command_rx) = mpsc::channel::<JsCommand>();

        IpcChannels {
            ui_thread: UiThreadChannels {
                event_sender: ui_event_tx,
                command_receiver: js_command_rx,
            },
            js_thread: JsThreadChannels {
                event_receiver: ui_event_rx,
                command_sender: js_command_tx,
            },
        }
    }
}

impl Default for IpcChannels {
    fn default() -> Self {
        Self::new()
    }
}
