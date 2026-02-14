// UI Thread Module
// Handles the main window, widget tree, and rendering using masonry_winit

use std::sync::mpsc::TryRecvError;

use masonry::core::{ErasedAction, NewWidget, StyleProperty, Widget, WidgetId};
use masonry::dpi::LogicalSize;
use masonry::parley::style::FontWeight;
use masonry::properties::types::Length;
use masonry::theme::default_property_set;
use masonry::widgets::{Button, ButtonPress, Flex, Label};
use masonry_winit::app::{AppDriver, DriverCtx, NewWindow, WindowId};
use masonry_winit::winit::window::Window;

use crate::ipc::{JsCommand, JsCommandReceiver, LogLevel, UiEvent, UiEventSender};

const VERTICAL_WIDGET_SPACING: Length = Length::const_px(20.0);

/// The main application driver that handles UI events and commands
pub struct AppJsDriver {
    /// The main window ID
    window_id: WindowId,
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
                    self.handle_command(command, _ctx);
                }
                Err(TryRecvError::Empty) => {
                    // No more commands, return
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    // JS thread has disconnected, should handle gracefully
                    eprintln!("JS thread disconnected");
                    break;
                }
            }
        }
    }

    /// Handle a single command from the JS thread
    fn handle_command(&mut self, command: JsCommand, _ctx: &mut DriverCtx<'_, '_>) {
        match command {
            JsCommand::SetTitle(title) => {
                // TODO: Update window title when masonry API supports it
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
                // TODO: Implement widget creation
                println!(
                    "[UI] Create widget: id={}, kind={:?}, parent={:?}",
                    id, kind, parent_id
                );
            }
            JsCommand::UpdateWidget { id, updates } => {
                // TODO: Implement widget updates
                println!("[UI] Update widget: id={}, updates={:?}", id, updates);
            }
            JsCommand::RemoveWidget { id } => {
                // TODO: Implement widget removal
                println!("[UI] Remove widget: id={}", id);
            }
            JsCommand::SetWidgetText { id, text } => {
                // TODO: Implement widget text update
                println!("[UI] Set widget text: id={}, text={}", id, text);
            }
            JsCommand::SetWidgetVisible { id, visible } => {
                // TODO: Implement widget visibility
                println!("[UI] Set widget visible: id={}, visible={}", id, visible);
            }
            JsCommand::ResizeWindow { width, height } => {
                // TODO: Implement window resize
                println!("[UI] Resize window: {}x{}", width, height);
            }
            JsCommand::CloseWindow => {
                // TODO: Implement window close
                println!("[UI] Close window requested");
            }
            JsCommand::ExitApp => {
                // TODO: Implement app exit
                println!("[UI] Exit app requested");
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
                action: crate::ipc::WidgetActionKind::Click,
            });
        } else {
            eprintln!("Unexpected action {:?}", action);
        }
    }
}

/// Create the initial widget tree for the application
fn create_initial_ui() -> impl Widget {
    let label = Label::new("Welcome to AppJS!")
        .with_style(StyleProperty::FontSize(32.0))
        .with_style(StyleProperty::FontWeight(FontWeight::BOLD));

    let status_label = Label::new("Waiting for JS runtime...");

    let button = Button::with_text("Send Event to JS");

    // Arrange widgets vertically with spacing
    Flex::column()
        .with_child(label.with_auto_id())
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(status_label.with_auto_id())
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(button.with_auto_id())
}

/// Run the UI application on the main thread
///
/// This function blocks and runs the event loop.
/// The `event_sender` and `command_receiver` are used for IPC with the JS thread.
pub fn run_ui(event_sender: UiEventSender, command_receiver: JsCommandReceiver) {
    let window_size = LogicalSize::new(800.0, 600.0);
    let window_id = WindowId::next();

    let window_attributes = Window::default_attributes()
        .with_title("AppJS - JavaScript Desktop Runtime")
        .with_resizable(true)
        .with_min_inner_size(LogicalSize::new(400.0, 300.0))
        .with_inner_size(window_size);

    let driver = AppJsDriver::new(window_id, event_sender, command_receiver);
    let main_widget = create_initial_ui();

    // Create the event loop using masonry_winit's EventLoop
    let event_loop = masonry_winit::app::EventLoop::with_user_event();

    masonry_winit::app::run(
        event_loop,
        vec![NewWindow::new_with_id(
            window_id,
            window_attributes,
            NewWidget::new(main_widget).erased(),
        )],
        driver,
        default_property_set(),
    )
    .expect("Failed to run masonry application");
}
