use crate::ipc::WidgetKind;
use masonry::core::WidgetId;
use masonry::core::WidgetTag;
use masonry::widgets::Flex;
use std::collections::HashMap;

/// Tag for the root Flex container that holds all dynamically created widgets.
pub const ROOT_FLEX_TAG: WidgetTag<Flex> = WidgetTag::new("root_flex");

/// Information tracked for each JS-created widget.
#[derive(Debug, Clone)]
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
    /// Maps a parent ID (or "__root__") to an ordered list of child IDs.
    pub parent_to_children: HashMap<String, Vec<String>>,
}

impl WidgetManager {
    pub fn new() -> Self {
        let mut parent_to_children = HashMap::new();
        parent_to_children.insert("__root__".to_string(), Vec::new());
        Self {
            widgets: HashMap::new(),
            parent_to_children,
        }
    }

    pub fn register_widget(&mut self, id: String, info: WidgetInfo) {
        let parent_key = info
            .parent_id
            .clone()
            .unwrap_or_else(|| "__root__".to_string());
        self.widgets.insert(id.clone(), info);
        self.parent_to_children
            .entry(parent_key)
            .or_default()
            .push(id.clone());
        self.parent_to_children.entry(id).or_default();
    }

    pub fn current_child_count(&self, parent_key: &str) -> usize {
        self.parent_to_children
            .get(parent_key)
            .map(|children| children.len())
            .unwrap_or(0)
    }

    pub fn next_child_index(&self, parent_key: &str) -> usize {
        self.current_child_count(parent_key)
    }

    fn collect_descendants(&self, parent_id: &str, out: &mut Vec<String>) {
        if let Some(children) = self.parent_to_children.get(parent_id) {
            for child_id in children {
                out.push(child_id.clone());
                self.collect_descendants(child_id, out);
            }
        }
    }

    fn recompute_parent_state(&mut self, parent_key: &str) {
        if let Some(children) = self.parent_to_children.get(parent_key) {
            for (new_index, child_id) in children.iter().enumerate() {
                if let Some(info) = self.widgets.get_mut(child_id) {
                    info.child_index = new_index;
                }
            }
        }
    }

    pub fn remove_widget_subtree(&mut self, id: &str) -> Option<WidgetInfo> {
        let removed = self.widgets.remove(id)?;
        let parent_key = removed
            .parent_id
            .clone()
            .unwrap_or_else(|| "__root__".to_string());

        // Remove from parent's children list
        if let Some(siblings) = self.parent_to_children.get_mut(&parent_key) {
            siblings.retain(|child_id| child_id != id);
        }

        // Collect and remove all descendants recursively
        let mut descendants = Vec::new();
        self.collect_descendants(id, &mut descendants);
        for child_id in descendants {
            self.widgets.remove(&child_id);
            self.parent_to_children.remove(&child_id);
        }

        // Remove the sublist for the widget
        self.parent_to_children.remove(id);

        self.recompute_parent_state(&parent_key);

        Some(removed)
    }
}
