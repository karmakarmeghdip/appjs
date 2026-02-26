use masonry::app::RenderRoot;
use masonry::core::{NewWidget, WidgetOptions};
use masonry::widgets::Slider;

use crate::ipc::{BoxStyle, WidgetData, WidgetKind};
use crate::ui::styles::build_box_properties;
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    style: Option<BoxStyle>,
    data: Option<WidgetData>,
    child_index: usize,
) {
    let style_ref = style.as_ref();

    // Extract slider data from WidgetData
    let (min, max, value, step) = match &data {
        Some(WidgetData::Slider {
            min,
            max,
            value,
            step,
        }) => (*min, *max, *value, *step),
        _ => (0.0, 1.0, 0.5, None),
    };

    let mut slider = Slider::new(min, max, value);
    if let Some(step) = step {
        slider = slider.with_step(step);
    }

    let props = style_ref.map(build_box_properties).unwrap_or_default();
    let new_widget = NewWidget::new_with(slider, None, WidgetOptions::default(), props);
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
                kind: WidgetKind::Slider,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
