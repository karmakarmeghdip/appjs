use masonry::app::RenderRoot;
use masonry::core::{NewWidget, WidgetId};
use masonry::widgets::Flex;

use crate::ipc::WidgetKind;
use crate::ipc::WidgetStyle;
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    style: Option<WidgetStyle>,
    child_index: usize,
    widget_id: WidgetId,
) {
    // Grid is complex with typed dimensions; use a Flex as fallback
    println!("[UI] Grid widget not yet fully supported, using Flex column as fallback");
    let new_flex = Flex::column();
    let new_widget = NewWidget::new_with_id(new_flex, widget_id);
    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style.and_then(|s| s.flex),
    ) {
        widget_manager.child_counts.insert(id.clone(), 0);
        widget_manager.widgets.insert(
            id,
            WidgetInfo {
                widget_id,
                kind: WidgetKind::Flex,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
