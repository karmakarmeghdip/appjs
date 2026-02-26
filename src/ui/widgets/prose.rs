use masonry::app::RenderRoot;
use masonry::core::{NewWidget, PropertySet, WidgetId, WidgetOptions, WidgetTag};
use masonry::widgets::{Prose, TextArea};

use crate::ipc::{BoxStyle, WidgetKind};
use crate::ui::styles::{build_box_properties, build_text_styles};
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<BoxStyle>,
    child_index: usize,
) {
    let style_ref = style.as_ref();
    let initial_text = text.unwrap_or_default();

    let mut prose_area = TextArea::new_immutable(&initial_text);
    if let Some(s) = style_ref {
        for text_style in build_text_styles(s) {
            prose_area = prose_area.with_style(text_style);
        }
    }
    let prose = Prose::from_text_area(NewWidget::new(prose_area));

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(PropertySet::new);
    let new_widget = NewWidget::new_with(prose, None, WidgetOptions::default(), props);
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
                kind: WidgetKind::Prose,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
