use masonry::core::{StyleProperty, Widget};
use masonry::parley::style::FontWeight;
use masonry::properties::types::Length;
use masonry::widgets::{Button, Flex, Label};

const VERTICAL_WIDGET_SPACING: Length = Length::const_px(20.0);

/// Create the initial widget tree for the application
pub fn create_initial_ui() -> impl Widget {
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
