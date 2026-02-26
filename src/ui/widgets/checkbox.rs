use masonry::app::RenderRoot;
use masonry::core::{NewWidget, PropertySet, WidgetId, WidgetOptions, WidgetTag};
use masonry::widgets::Checkbox;

use crate::ipc::{BoxStyle, WidgetData, WidgetKind};
use crate::ui::styles::build_box_properties;
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<BoxStyle>,
    data: Option<WidgetData>,
    child_index: usize,
) {
    let style_ref = style.as_ref();
    let label_text = text.unwrap_or_default();

    // Extract checked state from WidgetData
    let checked = match &data {
        Some(WidgetData::Checkbox { checked }) => *checked,
        _ => false,
    };

    let checkbox = Checkbox::new(checked, label_text);

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(PropertySet::new);
    let new_widget = NewWidget::new_with(checkbox, None, WidgetOptions::default(), props);
    let widget_id = new_widget.id();

    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style_ref.and_then(|s| s.flex),
    ) {
        widget_manager.register_widget(
            id,
            WidgetInfo {
                widget_id,
                kind: WidgetKind::Checkbox,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
