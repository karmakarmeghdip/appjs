// IPC (Inter-Process Communication) Module
// Handles communication between the UI thread and JS runtime thread

pub mod channels;
pub mod color;
pub mod commands;
pub mod events;
pub mod msgpack;
pub mod server;

pub use channels::*;
pub use color::ColorValue;
pub use commands::*;
pub use events::*;
