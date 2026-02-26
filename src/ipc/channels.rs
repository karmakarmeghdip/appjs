use std::sync::mpsc::{self, Receiver, Sender};

use masonry::core::ErasedAction;
use masonry_winit::app::{EventLoopProxy, MasonryUserEvent, WindowId};

use super::commands::ClientCommand;
use super::{ClientCommandAction, UiEvent};

/// Sender for UI events (UI thread holds this)
pub type UiEventSender = Sender<UiEvent>;

/// Receiver for UI events (JS thread holds this)
pub type UiEventReceiver = Receiver<UiEvent>;

/// Sender that wraps EventLoopProxy to send ClientCommands directly to the UI event loop.
/// This is held by the client thread and wakes the event loop on each send (zero polling).
#[derive(Clone)]
pub struct ClientCommandSender {
    proxy: EventLoopProxy,
    window_id: WindowId,
    }

impl ClientCommandSender {
    pub fn new(proxy: EventLoopProxy, window_id: WindowId) -> Self {
        Self {
            proxy,
            window_id,
                    }
    }

    /// Send a ClientCommand to the UI thread by wrapping it in MasonryUserEvent::Action.
    /// This immediately wakes the winit event loop — no polling needed.
    pub fn send(&self, cmd: ClientCommand) -> Result<(), String> {
        let action: ErasedAction = Box::new(ClientCommandAction(cmd));
        self.proxy
            .send_event(MasonryUserEvent::AsyncAction(self.window_id, action))
            .map_err(|e| format!("EventLoopProxy send failed: {e:?}"))
    }
}

/// Contains all channel endpoints needed for IPC
pub struct IpcChannels {
    /// Endpoints for the UI thread
    pub ui: UiChannels,
    /// Endpoints for the IPC server thread
    pub ipc_server: IpcServerChannels,
}

/// Channel endpoints held by the UI thread
pub struct UiChannels {
    /// Send UI events to IPC server thread
    pub event_sender: UiEventSender,
}

/// Channel endpoints held by the IPC server thread
pub struct IpcServerChannels {
    /// Receive UI events from UI thread
    pub event_receiver: UiEventReceiver,
    /// Send commands to UI thread (via EventLoopProxy, zero polling)
    pub command_sender: ClientCommandSender,
}

impl IpcChannels {
    /// Create a new set of IPC channels for communication between threads.
    /// The `proxy` and `window_id` are needed so JS commands can wake the UI event loop.
    pub fn new(proxy: EventLoopProxy, window_id: WindowId) -> Self {
        let (ui_event_tx, ui_event_rx) = mpsc::channel::<UiEvent>();

        IpcChannels {
            ui: UiChannels {
                event_sender: ui_event_tx,
            },
            ipc_server: IpcServerChannels {
                event_receiver: ui_event_rx,
                command_sender: ClientCommandSender::new(proxy, window_id),
            },
        }
    }
}
