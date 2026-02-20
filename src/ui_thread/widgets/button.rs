use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::peniko::Color;
use masonry::properties::ContentColor;
use masonry::widgets::{Button, Flex, Label};

use crate::ipc::{CrossAlign, FlexDirection, MainAlign, WidgetKind, WidgetStyle};
use crate::ui_thread::styles::{build_box_properties, build_text_styles, default_text_style_props};
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::utils::add_to_parent;
use crate::ui_thread::widgets::svg_widget_impl::SvgWidget;
use masonry::properties::types::{CrossAxisAlignment, Length, MainAxisAlignment};

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
    let dir = style_ref.and_then(|s| s.direction.as_ref());
    let mut new_flex = match dir {
        Some(FlexDirection::Column) => Flex::column(),
        _ => Flex::row(), // Default to row for buttons
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
        } else {
            new_flex = new_flex.cross_axis_alignment(CrossAxisAlignment::Center);
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
        } else {
            new_flex = new_flex.main_axis_alignment(MainAxisAlignment::Center);
        }
        
        if let Some(gap) = s.gap {
            new_flex = new_flex.with_gap(Length::px(gap));
        } else {
            new_flex = new_flex.with_gap(Length::px(8.0)); // Default gap
        }
        
        if s.flex.is_some() {
            new_flex = new_flex.must_fill_main_axis(s.must_fill_main_axis.unwrap_or(true));
        } else if let Some(fill) = s.must_fill_main_axis {
            new_flex = new_flex.must_fill_main_axis(fill);
        }
    } else {
        new_flex = new_flex
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .main_axis_alignment(MainAxisAlignment::Center)
            .with_gap(Length::px(8.0));
    }

    if let Some(svg_data) = style_ref.and_then(|s| s.svg_data.as_deref()) {
        let svg_widget = SvgWidget::new(svg_data.to_string());
        new_flex = new_flex.with_child(NewWidget::new(svg_widget));
    }

    if let Some(btn_text) = text.as_deref() {
        let mut inner_label = Label::new(btn_text);
        let text_styles = style_ref
            .map(build_text_styles)
            .unwrap_or_else(default_text_style_props);
        for s in &text_styles {
            inner_label = inner_label.with_style(s.clone());
        }
        new_flex = new_flex.with_child(NewWidget::new(inner_label));
    }

    let button = Button::new(NewWidget::new(new_flex));
    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
    let new_widget = NewWidget::new_with(button, widget_id, WidgetOptions::default(), props);
    
    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style_ref.and_then(|s| s.flex),
    ) {
        // Allow Button to manage children dynamically by initializing its child count
        widget_manager.child_counts.insert(id.clone(), 0);
        widget_manager.widgets.insert(
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
