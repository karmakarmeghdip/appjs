use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::peniko::Color;
use masonry::properties::ContentColor;
use masonry::widgets::{Button, Label};

use crate::ipc::WidgetKind;
use crate::ipc::WidgetStyle;
use crate::ui_thread::styles::{build_box_properties, build_text_styles, default_text_style_props};
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
    let svg_data = style_ref.and_then(|s| s.svg_data.as_deref());

    let mut added = false;
    if let Some(svg) = svg_data {
        let button = Button::new(NewWidget::new(SvgWidget::new(svg.to_string())));
        let props = style_ref
            .map(build_box_properties)
            .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
        let new_widget = NewWidget::new_with(button, widget_id, WidgetOptions::default(), props);
        added = add_to_parent(
            render_root,
            widget_manager,
            &parent_id,
            new_widget,
            style_ref.and_then(|s| s.flex),
        );
    }

    if !added {
        let mut inner_label = Label::new(text.as_deref().unwrap_or("Button"));
        for s in &style_ref
            .map(build_text_styles)
            .unwrap_or_else(default_text_style_props)
        {
            inner_label = inner_label.with_style(s.clone());
        }
        let button = Button::new(NewWidget::new(inner_label));
        let props = style_ref
            .map(build_box_properties)
            .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
        let new_widget = NewWidget::new_with(button, widget_id, WidgetOptions::default(), props);
        added = add_to_parent(
            render_root,
            widget_manager,
            &parent_id,
            new_widget,
            style_ref.and_then(|s| s.flex),
        );
    }

    if added {
        widget_manager.widgets.insert(
            id,
            WidgetInfo {
                widget_id,
                kind: WidgetKind::IconButton,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
