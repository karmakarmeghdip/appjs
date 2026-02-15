// IPC (Inter-Process Communication) Module
// Handles communication between the UI thread and JS runtime thread

pub mod channels;
pub mod commands;
pub mod events;

pub use channels::*;
pub use commands::*;
pub use events::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_communication() {
        let channels = IpcChannels::new();

        channels
            .ui_thread
            .event_sender
            .send(UiEvent::MouseClick { x: 100.0, y: 200.0 })
            .expect("Failed to send UI event");

        let event = channels
            .js_thread
            .event_receiver
            .recv()
            .expect("Failed to receive UI event");

        match event {
            UiEvent::MouseClick { x, y } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
            }
            _ => panic!("Unexpected event type"),
        }

        channels
            .js_thread
            .command_sender
            .send(JsCommand::SetTitle("Test Title".to_string()))
            .expect("Failed to send JS command");

        let command = channels
            .ui_thread
            .command_receiver
            .recv()
            .expect("Failed to receive JS command");

        match command {
            JsCommand::SetTitle(title) => {
                assert_eq!(title, "Test Title");
            }
            _ => panic!("Unexpected command type"),
        }
    }
}
