use crate::ipc::{UiEvent, WidgetActionKind};

/// Serialize a UiEvent to JSON string for JavaScript consumption
pub fn serialize_event(event: &UiEvent) -> String {
    match event {
        UiEvent::WidgetAction { widget_id, action } => match action {
            WidgetActionKind::Click => {
                format!(
                    r#"{{"type":"widgetAction","widgetId":"{}","action":"click"}}"#,
                    escape_json_string(widget_id),
                )
            }
            WidgetActionKind::ValueChanged(v) => {
                format!(
                    r#"{{"type":"widgetAction","widgetId":"{}","action":"valueChanged","value":{}}}"#,
                    escape_json_string(widget_id),
                    v,
                )
            }
        },
    }
}

pub fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
