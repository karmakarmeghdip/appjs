use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::properties::types::{CrossAxisAlignment, Length, MainAxisAlignment};
use masonry::widgets::Flex;

use crate::ipc::{CrossAlign, FlexDirection, MainAlign, WidgetKind, WidgetStyle};
use crate::ui_thread::styles::build_box_properties;
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    kind: WidgetKind, // Flex or Container
    parent_id: Option<String>,
    style: Option<WidgetStyle>,
    child_index: usize,
    widget_id: WidgetId,
) {
    let style_ref = style.as_ref();
    let dir = style_ref.and_then(|s| s.direction.as_ref());
    let mut new_flex = match dir {
        Some(FlexDirection::Row) => Flex::row(),
        _ => Flex::column(),
    };

    if let Some(s) = style_ref {
        if let Some(ref ca) = s.cross_axis_alignment {
            new_flex = new_flex.cross_axis_alignment(match ca {
                CrossAlign::Start => CrossAxisAlignment::Start,
                CrossAlign::Center => CrossAxisAlignment::Center,
                CrossAlign::End => CrossAxisAlignment::End,
                CrossAlign::Fill => CrossAxisAlignment::Fill,
                CrossAlign::Baseline => CrossAxisAlignment::Baseline,
            });
        }
        if let Some(ref ma) = s.main_axis_alignment {
            new_flex = new_flex.main_axis_alignment(match ma {
                MainAlign::Start => MainAxisAlignment::Start,
                MainAlign::Center => MainAxisAlignment::Center,
                MainAlign::End => MainAxisAlignment::End,
                MainAlign::SpaceBetween => MainAxisAlignment::SpaceBetween,
                MainAlign::SpaceAround => MainAxisAlignment::SpaceAround,
                MainAlign::SpaceEvenly => MainAxisAlignment::SpaceEvenly,
            });
        }
        if let Some(gap) = s.gap {
            new_flex = new_flex.with_gap(Length::px(gap));
        }
        // If flex factor is set, auto-enable must_fill_main_axis
        // so the container actually expands to fill the space granted by flex
        if s.flex.is_some() {
            new_flex = new_flex.must_fill_main_axis(s.must_fill_main_axis.unwrap_or(true));
        } else if let Some(fill) = s.must_fill_main_axis {
            new_flex = new_flex.must_fill_main_axis(fill);
        }
    }

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(Properties::new);
    let new_widget = NewWidget::new_with(new_flex, widget_id, WidgetOptions::default(), props);

    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style_ref.and_then(|s| s.flex),
    ) {
        // Flex/Container can have children, so init child count
        widget_manager.child_counts.insert(id.clone(), 0);
        widget_manager.widgets.insert(
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
