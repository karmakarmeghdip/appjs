use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};

use crate::ipc::WidgetKind;
use crate::ipc::WidgetStyle;
use crate::ui_thread::styles::build_box_properties;
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::svg_widget_impl::SvgWidget;
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
    let style_ref = style.as_ref();
    let svg_data = text
        .as_deref()
        .or_else(|| style_ref.and_then(|s| s.svg_data.as_deref()));

    if let Some(svg) = svg_data {
        let props = style_ref
            .map(build_box_properties)
            .unwrap_or_else(Properties::new);
        let new_widget = NewWidget::new_with(
            SvgWidget::new(svg.to_string()),
            widget_id,
            WidgetOptions::default(),
            props,
        );

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
                    kind: WidgetKind::Svg,
                    parent_id: parent_id.clone(),
                    child_index,
                },
            );
        }
    } else {
        eprintln!("[UI] SVG widget '{}' missing svg_data/text payload", id);
    }
}
