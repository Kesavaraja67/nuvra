// nuvra-mod-notifications/src/queue.rs
// Ephemeral in-memory notification queue with DND and allow/block filtering (Section 14.4, R-20).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationItem {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
}

#[derive(Default)]
pub struct NotificationFilter {
    pub blocked_apps: HashSet<String>,
    pub dnd_active: bool,
}

pub struct NotificationQueue {
    queue: VecDeque<NotificationItem>,
    max_capacity: usize,
    filter: NotificationFilter,
    next_id: u32,
}

impl NotificationQueue {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_capacity),
            max_capacity,
            filter: NotificationFilter::default(),
            next_id: 1,
        }
    }

    pub fn set_dnd(&mut self, active: bool) {
        self.filter.dnd_active = active;
    }

    pub fn block_app(&mut self, app_name: &str) {
        self.filter
            .blocked_apps
            .insert(app_name.trim().to_lowercase());
    }

    pub fn unblock_app(&mut self, app_name: &str) {
        self.filter
            .blocked_apps
            .remove(&app_name.trim().to_lowercase());
    }

    /// Push a notification if not suppressed by DND or blocked app list.
    pub fn push(&mut self, app_name: String, summary: String) -> bool {
        if self.filter.dnd_active {
            return false;
        }

        if self
            .filter
            .blocked_apps
            .contains(&app_name.trim().to_lowercase())
        {
            return false;
        }

        if self.queue.len() >= self.max_capacity {
            self.queue.pop_front();
        }

        let item = NotificationItem {
            id: self.next_id,
            app_name,
            summary,
        };
        self.next_id = self.next_id.wrapping_add(1);
        self.queue.push_back(item);
        true
    }

    pub fn peek(&self) -> Option<&NotificationItem> {
        self.queue.front()
    }

    pub fn pop(&mut self) -> Option<NotificationItem> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_queue_push_pop() {
        let mut q = NotificationQueue::new(3);
        assert!(q.push("Slack".into(), "New message".into()));
        assert_eq!(q.len(), 1);
        assert_eq!(q.peek().unwrap().app_name, "Slack");

        let item = q.pop().unwrap();
        assert_eq!(item.app_name, "Slack");
        assert!(q.is_empty());
    }

    #[test]
    fn test_notification_filter_dnd() {
        let mut q = NotificationQueue::new(3);
        q.set_dnd(true);
        assert!(!q.push("Slack".into(), "New message".into()));
        assert!(q.is_empty());
    }

    #[test]
    fn test_notification_filter_blocked_app() {
        let mut q = NotificationQueue::new(3);
        q.block_app("SpamApp");
        assert!(!q.push("SpamApp".into(), "Ad banner".into()));
        assert!(q.push("ValidApp".into(), "Important".into()));
        assert_eq!(q.len(), 1);
    }
}
