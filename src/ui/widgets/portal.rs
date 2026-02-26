use masonry::app::RenderRoot;
use masonry::core::{NewWidget, PropertySet, WidgetId, WidgetOptions, WidgetTag};
use masonry::widgets::{Flex, Portal};

use crate::ipc::{BoxStyle, WidgetKind};
use crate::ui::styles::build_box_properties;
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    style: Option<BoxStyle>,
    child_index: usize,
) {
    let style_ref = style.as_ref();
    let inner_flex = Flex::column();
    let portal = Portal::new(NewWidget::new(inner_flex));

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(PropertySet::new);
    let new_widget = NewWidget::new_with(portal, None, WidgetOptions::default(), props);
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
                kind: WidgetKind::Portal,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
