use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, StyleProperty, WidgetId, WidgetOptions};
use masonry::parley::style::{FontFamily, FontStack, GenericFamily};
use masonry::peniko::Color;
use masonry::properties::ContentColor;
use masonry::widgets::Label;

use crate::ipc::WidgetKind;
use crate::ipc::WidgetStyle;
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    kind: WidgetKind,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<WidgetStyle>,
    child_index: usize,
    widget_id: WidgetId,
) {
    eprintln!(
        "[UI] Widget kind {:?} not recognized, creating Label as fallback",
        kind
    );
    let fallback = format!("[{:?}]", kind);
    let label_text = text.as_deref().unwrap_or(&fallback);
    let label = Label::new(label_text)
        .with_style(StyleProperty::FontSize(20.0))
        .with_style(StyleProperty::FontStack(FontStack::Single(
            FontFamily::Generic(GenericFamily::SansSerif),
        )));
    let new_widget = NewWidget::new_with(
        label,
        widget_id,
        WidgetOptions::default(),
        Properties::new().with(ContentColor::new(Color::WHITE)),
    );
    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style.and_then(|s| s.flex),
    ) {
        widget_manager.widgets.insert(
            id,
            WidgetInfo {
                widget_id,
                kind: kind.clone(),
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
