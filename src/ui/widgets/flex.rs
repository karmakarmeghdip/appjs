use masonry::app::RenderRoot;
use masonry::core::{NewWidget, PropertySet, WidgetId, WidgetOptions, WidgetTag};
use masonry::layout::Dim;
use masonry::properties::Dimensions;
use masonry::properties::types::{CrossAxisAlignment, MainAxisAlignment};
use masonry::widgets::Flex;

use crate::ipc::{BoxStyle, CrossAlign, FlexDirection, MainAlign, WidgetData, WidgetKind};
use crate::ui::styles::build_box_properties;
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    kind: WidgetKind, // Flex or Container
    parent_id: Option<String>,
    style: Option<BoxStyle>,
    _data: Option<WidgetData>,
    child_index: usize,
) {
    let style_ref = style.as_ref();

    let dir = style_ref.and_then(|s| s.direction.clone());
    let mut new_flex = match dir.as_ref() {
        Some(FlexDirection::Row) => Flex::row(),
        _ => Flex::column(),
    };

    let cross = style_ref.and_then(|s| s.cross_axis_alignment.clone());
    if let Some(ref ca) = cross {
        new_flex = new_flex.cross_axis_alignment(match ca {
            CrossAlign::Start => CrossAxisAlignment::Start,
            CrossAlign::Center => CrossAxisAlignment::Center,
            CrossAlign::End => CrossAxisAlignment::End,
            CrossAlign::Fill => CrossAxisAlignment::Stretch,
            CrossAlign::Baseline => CrossAxisAlignment::Start,
        });
    }

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
    }

    let mut props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(PropertySet::new);

    if style_ref
        .and_then(|s| s.must_fill_main_axis)
        .unwrap_or(false)
    {
        let stretch_dims = match dir.as_ref() {
            Some(FlexDirection::Row) => Dimensions::width(Dim::Stretch),
            _ => Dimensions::height(Dim::Stretch),
        };
        props = props.with(stretch_dims);
    }

    let new_widget = NewWidget::new_with(new_flex, None, WidgetOptions::default(), props);
    let widget_id = new_widget.id();

    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style_ref.and_then(|s| s.flex),
    ) {
        // Flex/Container can have children, so init child count
        widget_manager.register_widget(
            id,
            WidgetInfo {
                widget_id,
                kind,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
