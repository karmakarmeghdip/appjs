use masonry::core::{Properties, StyleProperty};
use masonry::parley::style::{
    FontFamily, FontStack, FontStyle, FontWeight, GenericFamily, LineHeight,
};
use masonry::peniko::Color;
use masonry::properties::types::{CrossAxisAlignment, Length, MainAxisAlignment};
use masonry::properties::{
    Background, BorderColor, BorderWidth, ContentColor, CornerRadius, HoveredBorderColor, Padding,
};
use masonry::widgets::Flex;

use crate::ipc::{BoxStyle, ColorValue, CrossAlign, FontStyleValue, MainAlign, PaddingValue};

// ── Color conversion helper ──

pub fn color_value_to_peniko(cv: &ColorValue) -> Color {
    match cv {
        ColorValue::Rgba { r, g, b, a } => Color::from_rgba8(*r, *g, *b, *a),
        ColorValue::Named(name) => {
            // Fallback for any named colors that didn't parse
            eprintln!("[UI] Unknown named color '{}', using white", name);
            Color::WHITE
        }
    }
}

// ── Style application helpers ──

/// Apply text-related StyleProperty items to a builder that supports `with_style`
pub fn build_text_styles(style: &BoxStyle) -> Vec<StyleProperty> {
    let mut props = Vec::new();

    if let Some(size) = style.font_size {
        props.push(StyleProperty::FontSize(size));
    }
    if let Some(weight) = style.font_weight {
        props.push(StyleProperty::FontWeight(FontWeight::new(weight)));
    }
    if let Some(ref fs) = style.font_style {
        props.push(StyleProperty::FontStyle(match fs {
            FontStyleValue::Normal => FontStyle::Normal,
            FontStyleValue::Italic => FontStyle::Italic,
        }));
    }
    if let Some(ref family) = style.font_family {
        props.push(StyleProperty::FontStack(FontStack::Single(
            FontFamily::Named(std::borrow::Cow::Owned(family.clone())),
        )));
    } else {
        // Default to sans-serif
        props.push(StyleProperty::FontStack(FontStack::Single(
            FontFamily::Generic(GenericFamily::SansSerif),
        )));
    }
    if let Some(ls) = style.letter_spacing {
        props.push(StyleProperty::LetterSpacing(ls));
    }
    if let Some(lh) = style.line_height {
        props.push(StyleProperty::LineHeight(LineHeight::FontSizeRelative(lh)));
    }
    if let Some(ws) = style.word_spacing {
        props.push(StyleProperty::WordSpacing(ws));
    }
    if let Some(true) = style.underline {
        props.push(StyleProperty::Underline(true));
    }
    if let Some(true) = style.strikethrough {
        props.push(StyleProperty::Strikethrough(true));
    }
    // Note: For Labels, color is handled via ContentColor property, not StyleProperty::Brush

    props
}

/// Build a Properties set with box-model styling
pub fn build_box_properties(style: &BoxStyle) -> Properties {
    let mut props = Properties::new();

    if let Some(ref color) = style.color {
        props = props.with(ContentColor::new(color_value_to_peniko(color)));
    }
    if let Some(ref bg) = style.background {
        props = props.with(Background::Color(color_value_to_peniko(bg)));
    }
    if let Some(ref bc) = style.border_color {
        props = props.with(BorderColor::new(color_value_to_peniko(bc)));
    }
    if let Some(ref hbc) = style.hover_border_color {
        props = props.with(HoveredBorderColor(BorderColor::new(color_value_to_peniko(
            hbc,
        ))));
    }
    if let Some(bw) = style.border_width {
        props = props.with(BorderWidth::all(bw));
    }
    if let Some(cr) = style.corner_radius {
        props = props.with(CornerRadius::all(cr));
    }
    if let Some(ref pad) = style.padding {
        match pad {
            PaddingValue::Uniform(v) => {
                props = props.with(Padding::all(*v));
            }
            PaddingValue::Sides {
                top,
                right,
                bottom,
                left,
            } => {
                props = props.with(Padding {
                    left: *left,
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                });
            }
        }
    }

    props
}

/// Apply box-model style properties to an existing widget via insert_prop.
/// Works on any WidgetMut that implements HasProperty for the relevant properties.
pub fn apply_box_props_to_widget(
    widget: &mut masonry::core::WidgetMut<'_, impl masonry::core::Widget>,
    style: &BoxStyle,
) {
    if let Some(ref color) = style.color {
        widget.insert_prop(ContentColor::new(color_value_to_peniko(color)));
    }
    if let Some(ref bg) = style.background {
        widget.insert_prop(Background::Color(color_value_to_peniko(bg)));
    }
    if let Some(ref bc) = style.border_color {
        widget.insert_prop(BorderColor::new(color_value_to_peniko(bc)));
    }
    if let Some(ref hbc) = style.hover_border_color {
        widget.insert_prop(HoveredBorderColor(BorderColor::new(color_value_to_peniko(
            hbc,
        ))));
    }
    if let Some(bw) = style.border_width {
        widget.insert_prop(BorderWidth::all(bw));
    }
    if let Some(cr) = style.corner_radius {
        widget.insert_prop(CornerRadius::all(cr));
    }
    if let Some(ref pad) = style.padding {
        match pad {
            PaddingValue::Uniform(v) => {
                widget.insert_prop(Padding::all(*v));
            }
            PaddingValue::Sides {
                top,
                right,
                bottom,
                left,
            } => {
                widget.insert_prop(Padding {
                    left: *left,
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                });
            }
        }
    }
}

/// Apply style to a Flex widget (root or otherwise). Handles box props + flex-specific props.
pub fn apply_flex_style(flex: &mut masonry::core::WidgetMut<'_, Flex>, style: &BoxStyle) {
    apply_box_props_to_widget(flex, style);

    if let Some(ref ca) = style.cross_axis_alignment {
        Flex::set_cross_axis_alignment(
            flex,
            match ca {
                CrossAlign::Start => CrossAxisAlignment::Start,
                CrossAlign::Center => CrossAxisAlignment::Center,
                CrossAlign::End => CrossAxisAlignment::End,
                CrossAlign::Fill => CrossAxisAlignment::Fill,
                CrossAlign::Baseline => CrossAxisAlignment::Baseline,
            },
        );
    }
    if let Some(ref ma) = style.main_axis_alignment {
        Flex::set_main_axis_alignment(
            flex,
            match ma {
                MainAlign::Start => MainAxisAlignment::Start,
                MainAlign::Center => MainAxisAlignment::Center,
                MainAlign::End => MainAxisAlignment::End,
                MainAlign::SpaceBetween => MainAxisAlignment::SpaceBetween,
                MainAlign::SpaceAround => MainAxisAlignment::SpaceAround,
                MainAlign::SpaceEvenly => MainAxisAlignment::SpaceEvenly,
            },
        );
    }
    if let Some(gap) = style.gap {
        Flex::set_gap(flex, Length::px(gap));
    }
    if let Some(fill) = style.must_fill_main_axis {
        Flex::set_must_fill_main_axis(flex, fill);
    }
}

pub fn default_text_style_props() -> Vec<StyleProperty> {
    vec![
        StyleProperty::FontSize(20.0),
        StyleProperty::FontStack(FontStack::Single(FontFamily::Generic(
            GenericFamily::SansSerif,
        ))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{BoxStyle, FontStyleValue, PaddingValue};
    use masonry::core::StyleProperty;

    #[test]
    fn test_color_value_to_peniko() {
        let rgba = ColorValue::Rgba {
            r: 255,
            g: 128,
            b: 64,
            a: 255,
        };
        let color = color_value_to_peniko(&rgba);
        assert_eq!(color, Color::from_rgba8(255, 128, 64, 255));

        let named = ColorValue::Named("invalid_color".to_string());
        let fallback = color_value_to_peniko(&named);
        assert_eq!(fallback, Color::WHITE);
    }

    #[test]
    fn test_build_text_styles() {
        let style = BoxStyle {
            font_size: Some(24.0),
            font_weight: Some(700.0),
            font_style: Some(FontStyleValue::Italic),
            font_family: Some("Arial".to_string()),
            letter_spacing: Some(1.5),
            line_height: Some(1.2),
            word_spacing: Some(2.0),
            underline: Some(true),
            strikethrough: Some(true),
            ..Default::default()
        };

        let props = build_text_styles(&style);

        // Assert we get exactly 9 properties
        assert_eq!(props.len(), 9);
    }

    #[test]
    fn test_build_text_styles_default_font() {
        let style = BoxStyle::default();
        let props = build_text_styles(&style);

        // Should only have the default font stack
        assert_eq!(props.len(), 1);
        assert!(matches!(props[0], StyleProperty::FontStack(_)));
    }

    #[test]
    fn test_build_box_properties() {
        let style = BoxStyle {
            color: Some(ColorValue::Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            background: Some(ColorValue::Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            }),
            border_color: Some(ColorValue::Rgba {
                r: 200,
                g: 200,
                b: 200,
                a: 255,
            }),
            hover_border_color: Some(ColorValue::Rgba {
                r: 100,
                g: 100,
                b: 100,
                a: 255,
            }),
            border_width: Some(2.0),
            corner_radius: Some(4.0),
            padding: Some(PaddingValue::Uniform(10.0)),
            ..Default::default()
        };

        let props = build_box_properties(&style);
        let _ = props;
    }

    #[test]
    fn test_default_text_style_props() {
        let defaults = default_text_style_props();
        assert_eq!(defaults.len(), 2);
    }
}
