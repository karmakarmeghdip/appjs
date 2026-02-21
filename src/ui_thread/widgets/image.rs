use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::peniko::{ImageAlphaType, ImageData, ImageFormat};
use masonry::properties::ObjectFit;
use masonry::widgets::Image;

use crate::ipc::{WidgetKind, WidgetParams, WidgetStyle};
use crate::ui_thread::styles::build_box_properties;
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::utils::add_to_parent;

/// Decode raw file bytes (PNG/JPEG/WebP/etc.) into masonry ImageData
fn decode_image_bytes(data: &[u8]) -> Option<ImageData> {
    match image::load_from_memory(data) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some(ImageData {
                data: rgba.into_raw().into(),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width,
                height,
            })
        }
        Err(e) => {
            eprintln!("[UI] Failed to decode image: {}", e);
            None
        }
    }
}

/// Parse object-fit string into masonry ObjectFit
fn parse_object_fit(s: &str) -> ObjectFit {
    match s.to_lowercase().as_str() {
        "contain" => ObjectFit::Contain,
        "cover" => ObjectFit::Cover,
        "fill" => ObjectFit::Fill,
        "none" => ObjectFit::None,
        "scale-down" | "scaledown" | "scale_down" => ObjectFit::ScaleDown,
        _ => ObjectFit::Contain,
    }
}

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    style: Option<WidgetStyle>,
    params: Option<WidgetParams>,
    child_index: usize,
    widget_id: WidgetId,
) {
    let image_data_bytes = match &params {
        Some(WidgetParams::Image { data, .. }) => data.as_slice(),
        _ => {
            eprintln!("[UI] Image widget '{}' missing image data in params", id);
            return;
        }
    };

    let image_data = match decode_image_bytes(image_data_bytes) {
        Some(d) => d,
        None => {
            eprintln!("[UI] Image widget '{}' failed to decode image data", id);
            return;
        }
    };

    let object_fit = match &params {
        Some(WidgetParams::Image { object_fit, .. }) => object_fit
            .as_deref()
            .map(parse_object_fit)
            .unwrap_or(ObjectFit::Contain),
        _ => ObjectFit::Contain,
    };

    let style_ref = style.as_ref();
    let mut props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(Properties::new);
    props = props.with(object_fit);

    let new_widget = NewWidget::new_with(
        Image::new(image_data),
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
                kind: WidgetKind::Image,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}

/// Update an existing Image widget's data
pub fn update_data(render_root: &mut RenderRoot, widget_id: WidgetId, data: &[u8]) {
    let image_data = match decode_image_bytes(data) {
        Some(d) => d,
        None => {
            eprintln!("[UI] Failed to decode image data for update");
            return;
        }
    };

    render_root.edit_widget(widget_id, |mut widget| {
        let mut img = widget.downcast::<Image>();
        Image::set_image_data(&mut img, image_data);
    });
}
