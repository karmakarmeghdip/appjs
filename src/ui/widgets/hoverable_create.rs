use masonry::app::RenderRoot;
use masonry::core::{NewWidget, WidgetOptions};

use crate::ipc::{BoxStyle, WidgetKind};
use crate::ui::styles::build_box_properties;
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;

use super::hoverable::Hoverable;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    style: Option<BoxStyle>,
    child_index: usize,
) {
    let style_ref = style.as_ref();

    let hoverable = Hoverable::new_empty();

    let props = style_ref.map(build_box_properties).unwrap_or_default();
    let new_widget = NewWidget::new_with(hoverable, None, WidgetOptions::default(), props);
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
                kind: WidgetKind::Hoverable,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
