use std::collections::HashMap;
use masonry::core::WidgetId;
use masonry::widgets::Flex;
use masonry::core::WidgetTag;
use crate::ipc::WidgetKind;

/// Tag for the root Flex container that holds all dynamically created widgets.
pub const ROOT_FLEX_TAG: WidgetTag<Flex> = WidgetTag::new("root_flex");

/// Information tracked for each JS-created widget.
#[derive(Debug)]
pub struct WidgetInfo {
    /// The masonry WidgetId assigned when the widget was inserted.
    pub widget_id: WidgetId,
    /// What kind of widget this is.
    pub kind: WidgetKind,
    /// The parent widget JS id (None means root Flex).
    pub parent_id: Option<String>,
    /// Index in the parent Flex's children list.
    pub child_index: usize,
}

/// Manages the mapping from JS widget IDs to masonry widget state.
pub struct WidgetManager {
    /// Maps JS string IDs → tracked widget info.
    pub widgets: HashMap<String, WidgetInfo>,
    /// Tracks how many children each Flex container has (by JS id, or "__root__" for root).
    pub child_counts: HashMap<String, usize>,
}

impl WidgetManager {
    pub fn new() -> Self {
        let mut child_counts = HashMap::new();
        child_counts.insert("__root__".to_string(), 0);
        Self {
            widgets: HashMap::new(),
            child_counts,
        }
    }

    pub fn next_child_index(&mut self, parent_key: &str) -> usize {
        let count = self.child_counts.entry(parent_key.to_string()).or_insert(0);
        let idx = *count;
        *count += 1;
        idx
    }
}
