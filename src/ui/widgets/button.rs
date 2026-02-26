use masonry::app::RenderRoot;
use masonry::core::{NewWidget, PropertySet, WidgetId, WidgetOptions, WidgetTag};
use masonry::layout::Length;
use masonry::peniko::Color;
use masonry::properties::{ContentColor, Dimensions};
use masonry::widgets::{Button, Flex, Label};

use crate::ipc::{BoxStyle, CrossAlign, FlexDirection, MainAlign, WidgetData, WidgetKind};
use crate::ui::styles::{
    build_box_properties, build_text_styles, color_value_to_peniko, default_text_style_props,
};
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::svg_widget_impl::SvgWidget;
use crate::ui::widgets::utils::add_to_parent;

use masonry::properties::types::{CrossAxisAlignment, MainAxisAlignment};

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

    // Extract button-specific data
    let svg_data = match &data {
        Some(WidgetData::Button { svg_data }) => svg_data.clone(),
        _ => None,
    };

    // Button inner layout comes from BoxStyle (direction/alignment/gap/fill).
    // Direction defaults to Row.
    let dir = style_ref.and_then(|s| s.direction.clone());
    let mut new_flex = match dir.as_ref() {
        Some(FlexDirection::Column) => Flex::column(),
        _ => Flex::row(), // Default to row for buttons
    };

    // Cross axis alignment
    let cross = style_ref.and_then(|s| s.cross_axis_alignment.clone());
    if let Some(ref ca) = cross {
        new_flex = new_flex.cross_axis_alignment(match ca {
            CrossAlign::Start => CrossAxisAlignment::Start,
            CrossAlign::Center => CrossAxisAlignment::Center,
            CrossAlign::End => CrossAxisAlignment::End,
            CrossAlign::Fill => CrossAxisAlignment::Stretch,
            CrossAlign::Baseline => CrossAxisAlignment::Start,
        });
    } else {
        new_flex = new_flex.cross_axis_alignment(CrossAxisAlignment::Center);
    }

    // Main axis alignment
    let main = style_ref.and_then(|s| s.main_axis_alignment.clone());
    if let Some(ref ma) = main {
        new_flex = new_flex.main_axis_alignment(match ma {
            MainAlign::Start => MainAxisAlignment::Start,
            MainAlign::Center => MainAxisAlignment::Center,
            MainAlign::End => MainAxisAlignment::End,
            MainAlign::SpaceBetween => MainAxisAlignment::SpaceBetween,
            MainAlign::SpaceAround => MainAxisAlignment::SpaceAround,
            MainAlign::SpaceEvenly => MainAxisAlignment::SpaceEvenly,
        });
    } else {
        new_flex = new_flex.main_axis_alignment(MainAxisAlignment::Center);
    }

    // Add SVG icon if present
    if let Some(ref svg_str) = svg_data {
        let svg_widget = SvgWidget::new(svg_str.clone());
        let mut svg_props = PropertySet::new();

        if let Some(color) = style_ref.and_then(|s| s.color.as_ref()) {
            svg_props = svg_props.with(ContentColor::new(color_value_to_peniko(color)));
        }

        if let Some(icon_size) = style_ref.and_then(|s| s.icon_size) {
            svg_props = svg_props.with(Dimensions::fixed(
                Length::px(icon_size),
                Length::px(icon_size),
            ));
        }

        let svg_new_widget =
            NewWidget::new_with(svg_widget, None, WidgetOptions::default(), svg_props);
        new_flex = new_flex.with_fixed(svg_new_widget);
    }

    // Add label
    if let Some(btn_text) = text.as_deref() {
        let mut inner_label = Label::new(btn_text);
        let text_styles = style_ref
            .map(build_text_styles)
            .unwrap_or_else(default_text_style_props);
        for s in &text_styles {
            inner_label = inner_label.with_style(s.clone());
        }

        let label_widget = if let Some(color) = style_ref.and_then(|s| s.color.as_ref()) {
            let label_props =
                PropertySet::new().with(ContentColor::new(color_value_to_peniko(color)));
            NewWidget::new_with(inner_label, None, WidgetOptions::default(), label_props)
        } else {
            NewWidget::new(inner_label)
        };

        new_flex = new_flex.with_fixed(label_widget);
    }

    let button = Button::new(NewWidget::new(new_flex));
    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(|| PropertySet::new().with(ContentColor::new(Color::WHITE)));
    let new_widget = NewWidget::new_with(button, None, WidgetOptions::default(), props);
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
                kind: WidgetKind::Button,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
