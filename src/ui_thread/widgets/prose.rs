use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::peniko::Color;
use masonry::properties::ContentColor;
use masonry::widgets::Prose;

use crate::ipc::WidgetKind;
use crate::ipc::WidgetStyle;
use crate::ui_thread::styles::build_box_properties;
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<WidgetStyle>,
    child_index: usize,
    widget_id: WidgetId,
) {
    let prose_text = text.as_deref().unwrap_or("");
    let prose = Prose::new(prose_text);
    let style_ref = style.as_ref();
    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
    let new_widget = NewWidget::new_with(prose, widget_id, WidgetOptions::default(), props);
    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style_ref.and_then(|s| s.flex),
    ) {
        widget_manager.widgets.insert(
            id,
            WidgetInfo {
                widget_id,
                kind: WidgetKind::Prose,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
