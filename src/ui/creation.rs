use masonry::app::RenderRoot;

use super::widget_manager::WidgetManager;
use super::widgets;
use crate::ipc::{BoxStyle, WidgetData, WidgetKind};

#[allow(clippy::too_many_arguments)]
pub fn create_and_add_widget(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    kind: WidgetKind,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<BoxStyle>,
    data: Option<WidgetData>,
) {
    println!(
        "[UI] Creating widget: id={}, kind={:?}, parent={:?}",
        id, kind, parent_id
    );

    let parent_key = parent_id.as_deref().unwrap_or("__root__").to_string();
    let child_index = widget_manager.next_child_index(&parent_key);

    match kind {
        WidgetKind::Label => {
            widgets::label::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                child_index,
            );
        }
        WidgetKind::Button => {
            widgets::button::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
            );
        }
        WidgetKind::Svg => {
            widgets::svg::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                data,
                child_index,
            );
        }
        WidgetKind::Flex | WidgetKind::Container => {
            widgets::flex::create(
                render_root,
                widget_manager,
                id,
                kind,
                parent_id,
                style,
                data,
                child_index,
            );
        }
        WidgetKind::SizedBox => {
            widgets::sized_box::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
            );
        }
        WidgetKind::Checkbox => {
            widgets::checkbox::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                data,
                child_index,
            );
        }
        WidgetKind::TextInput => {
            widgets::text_input::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                data,
                child_index,
            );
        }
        WidgetKind::TextArea => {
            widgets::text_area::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                child_index,
            );
        }
        WidgetKind::Prose => {
            widgets::prose::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                child_index,
            );
        }
        WidgetKind::ProgressBar => {
            widgets::progress_bar::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                data,
                child_index,
            );
        }
        WidgetKind::Spinner => {
            widgets::spinner::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
            );
        }
        WidgetKind::Slider => {
            widgets::slider::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                data,
                child_index,
            );
        }
        WidgetKind::ZStack => {
            widgets::zstack::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
            );
        }
        WidgetKind::Portal => {
            widgets::portal::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
            );
        }
        WidgetKind::Grid => {
            widgets::grid::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
            );
        }
        WidgetKind::Hoverable => {
            widgets::hoverable_create::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
            );
        }
        WidgetKind::Custom(_) => {
            widgets::custom::create(
                render_root,
                widget_manager,
                id,
                kind,
                parent_id,
                text,
                style,
                child_index,
            );
        }
        WidgetKind::Image => {
            widgets::image::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                data,
                child_index,
            );
        }
        WidgetKind::Video => {
            widgets::video::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                data,
                child_index,
            );
        }
    }
}
