// UI Thread Module
// Handles the main window, widget tree, and rendering using masonry_winit

pub mod driver;
pub mod handler;
pub mod layout;

use masonry::core::NewWidget;
use masonry::dpi::LogicalSize;
use masonry::theme::default_property_set;
use masonry_winit::app::NewWindow;
use masonry_winit::winit::window::Window;

use self::driver::AppJsDriver;
use self::layout::create_initial_ui;
use crate::ipc::{JsCommandReceiver, UiEventSender};

/// Run the UI application on the main thread
pub fn run_ui(event_sender: UiEventSender, command_receiver: JsCommandReceiver) {
    let window_size = LogicalSize::new(800.0, 600.0);
    let window_id = masonry_winit::app::WindowId::next();

    let window_attributes = Window::default_attributes()
        .with_title("AppJS - JavaScript Desktop Runtime")
        .with_resizable(true)
        .with_min_inner_size(LogicalSize::new(400.0, 300.0))
        .with_inner_size(window_size);

    let driver = AppJsDriver::new(window_id, event_sender, command_receiver);
    let main_widget = create_initial_ui();

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
