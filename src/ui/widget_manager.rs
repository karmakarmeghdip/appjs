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

#[cfg(test)]
mod tests {
    use super::*;
    use masonry::core::WidgetId;

    #[test]
    fn test_widget_manager_new() {
        let manager = WidgetManager::new();
        assert!(manager.widgets.is_empty());
        assert_eq!(manager.parent_to_children.len(), 1);
        assert!(manager.parent_to_children.contains_key("__root__"));
    }

    #[test]
    fn test_register_widget() {
        let mut manager = WidgetManager::new();

        let info1 = WidgetInfo {
            widget_id: WidgetId::next(),
            kind: WidgetKind::Button,
            parent_id: None,
            child_index: 0,
        };

        manager.register_widget("btn_1".to_string(), info1.clone());

        assert_eq!(manager.widgets.len(), 1);
        assert_eq!(manager.current_child_count("__root__"), 1);
        assert_eq!(
            manager.parent_to_children.get("__root__").unwrap()[0],
            "btn_1"
        );

        let info2 = WidgetInfo {
            widget_id: WidgetId::next(),
            kind: WidgetKind::Label,
            parent_id: Some("btn_1".to_string()),
            child_index: 0,
        };
        manager.register_widget("lbl_1".to_string(), info2);

        assert_eq!(manager.widgets.len(), 2);
        assert_eq!(manager.current_child_count("btn_1"), 1);
        assert_eq!(manager.parent_to_children.get("btn_1").unwrap()[0], "lbl_1");
    }

    #[test]
    fn test_remove_widget_subtree() {
        let mut manager = WidgetManager::new();

        manager.register_widget(
            "btn_1".to_string(),
            WidgetInfo {
                widget_id: WidgetId::next(),
                kind: WidgetKind::Button,
                parent_id: None,
                child_index: 0,
            },
        );
        manager.register_widget(
            "btn_2".to_string(),
            WidgetInfo {
                widget_id: WidgetId::next(),
                kind: WidgetKind::Button,
                parent_id: None,
                child_index: 1,
            },
        );

        manager.register_widget(
            "lbl_1".to_string(),
            WidgetInfo {
                widget_id: WidgetId::next(),
                kind: WidgetKind::Label,
                parent_id: Some("btn_1".to_string()),
                child_index: 0,
            },
        );
        manager.register_widget(
            "lbl_2".to_string(),
            WidgetInfo {
                widget_id: WidgetId::next(),
                kind: WidgetKind::Label,
                parent_id: Some("btn_1".to_string()),
                child_index: 1,
            },
        );
        manager.register_widget(
            "span_1".to_string(),
            WidgetInfo {
                widget_id: WidgetId::next(),
                kind: WidgetKind::Label,
                parent_id: Some("lbl_2".to_string()),
                child_index: 0,
            },
        );

        assert_eq!(manager.widgets.len(), 5);

        let removed = manager.remove_widget_subtree("btn_1");
        assert!(removed.is_some());

        assert_eq!(manager.widgets.len(), 1);
        assert!(manager.widgets.contains_key("btn_2"));

        let root_children = manager.parent_to_children.get("__root__").unwrap();
        assert_eq!(root_children.len(), 1);
        assert_eq!(root_children[0], "btn_2");

        assert_eq!(manager.widgets.get("btn_2").unwrap().child_index, 0);
    }

    #[test]
    fn test_next_child_index_and_counts() {
        let mut manager = WidgetManager::new();
        assert_eq!(manager.next_child_index("__root__"), 0);
        manager.register_widget(
            "btn_1".to_string(),
            WidgetInfo {
                widget_id: WidgetId::next(),
                kind: WidgetKind::Button,
                parent_id: None,
                child_index: 0,
            },
        );
        assert_eq!(manager.next_child_index("__root__"), 1);
        assert_eq!(manager.current_child_count("__root__"), 1);
    }
}
