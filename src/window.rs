//! Window layout: a binary split tree over independent viewports.
//!
//! A window points at a buffer and keeps its own viewport and point.
//! Several windows may show the same buffer; they share the text but not the
//! cursor position.

use crate::view::View;

/// A reasonable cap for a terminal UI; each pane still needs enough room to
/// show anything useful.
pub const MAX_WINDOWS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitDir {
    /// One pane above the other (`C-x 2`).
    AboveBelow,
    /// One pane left of the other (`C-x 3`).
    LeftRight,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    Leaf(usize),
    Split {
        dir: SplitDir,
        first: Box<Layout>,
        second: Box<Layout>,
    },
}

impl Layout {
    fn leaf_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        self.collect_leaf_ids(&mut ids);
        ids
    }

    fn collect_leaf_ids(&self, ids: &mut Vec<usize>) {
        match self {
            Layout::Leaf(id) => ids.push(*id),
            Layout::Split { first, second, .. } => {
                first.collect_leaf_ids(ids);
                second.collect_leaf_ids(ids);
            }
        }
    }

    fn split_leaf(&mut self, target: usize, new: usize, dir: SplitDir) -> bool {
        match self {
            Layout::Leaf(id) if *id == target => {
                *self = Layout::Split {
                    dir,
                    first: Box::new(Layout::Leaf(target)),
                    second: Box::new(Layout::Leaf(new)),
                };
                true
            }
            Layout::Leaf(_) => false,
            Layout::Split { first, second, .. } => {
                first.split_leaf(target, new, dir) || second.split_leaf(target, new, dir)
            }
        }
    }

    /// Replace the parent of `target` with its sibling.
    fn remove_leaf(self, target: usize) -> Option<Layout> {
        match self {
            Layout::Leaf(id) if id == target => None,
            Layout::Leaf(_) => Some(self),
            Layout::Split { dir, first, second } => {
                match (first.remove_leaf(target), second.remove_leaf(target)) {
                    (None, None) => None,
                    (Some(first), None) => Some(first),
                    (None, Some(second)) => Some(second),
                    (Some(first), Some(second)) => Some(Layout::Split {
                        dir,
                        first: Box::new(first),
                        second: Box::new(second),
                    }),
                }
            }
        }
    }

    fn renumber_after(&mut self, removed: usize) {
        match self {
            Layout::Leaf(id) => {
                if *id > removed {
                    *id -= 1;
                }
            }
            Layout::Split { first, second, .. } => {
                first.renumber_after(removed);
                second.renumber_after(removed);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Window {
    pub view: View,
    /// Index into `Editor::buffers`.
    pub buffer: usize,
}

#[derive(Clone, Debug)]
pub struct WindowSet {
    windows: Vec<Window>,
    layout: Layout,
    active: usize,
}

impl Default for WindowSet {
    fn default() -> Self {
        Self::single(View::pane())
    }
}

impl WindowSet {
    pub fn single(view: View) -> Self {
        Self {
            windows: vec![Window { view, buffer: 0 }],
            layout: Layout::Leaf(0),
            active: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.windows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn active(&self) -> usize {
        self.active
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.windows.len() {
            self.active = index;
        }
    }

    pub fn active_window(&self) -> &Window {
        &self.windows[self.active]
    }

    pub fn active_window_mut(&mut self) -> &mut Window {
        &mut self.windows[self.active]
    }

    pub fn active_view(&self) -> &View {
        &self.windows[self.active].view
    }

    pub fn active_view_mut(&mut self) -> &mut View {
        &mut self.windows[self.active].view
    }

    pub fn window(&self, index: usize) -> &Window {
        &self.windows[index]
    }

    pub fn window_mut(&mut self, index: usize) -> &mut Window {
        &mut self.windows[index]
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn windows_mut(&mut self) -> &mut [Window] {
        &mut self.windows
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn windows_for_buffer(&self, buffer: usize) -> Vec<usize> {
        self.windows
            .iter()
            .enumerate()
            .filter(|(_, window)| window.buffer == buffer)
            .map(|(index, _)| index)
            .collect()
    }

    /// Split the active window, leaving the active view selected.
    pub fn split(&mut self, dir: SplitDir) -> bool {
        if self.windows.len() >= MAX_WINDOWS {
            return false;
        }
        let mut new_window = self.windows[self.active].clone();
        new_window.view.top_offset = 0;
        let new = self.windows.len();
        if !self.layout.split_leaf(self.active, new, dir) {
            return false;
        }
        self.windows.push(new_window);
        true
    }

    /// Select the next window in layout order.
    pub fn other(&mut self) -> bool {
        if self.windows.len() <= 1 {
            return false;
        }
        let ids = self.layout.leaf_ids();
        let position = ids.iter().position(|id| *id == self.active).unwrap_or(0);
        self.active = ids[(position + 1) % ids.len()];
        true
    }

    /// Close the active window. The last window is never closed.
    pub fn delete_active(&mut self) -> bool {
        if self.windows.len() <= 1 {
            return false;
        }
        let target = self.active;
        let Some(mut layout) = self.layout.clone().remove_leaf(target) else {
            return false;
        };
        layout.renumber_after(target);
        self.windows.remove(target);
        self.layout = layout;
        self.active = target.min(self.windows.len() - 1);
        true
    }

    /// Close every window except the active one.
    pub fn delete_others(&mut self) -> bool {
        if self.windows.len() <= 1 {
            return false;
        }
        let window = self.windows[self.active].clone();
        self.windows = vec![window];
        self.layout = Layout::Leaf(0);
        self.active = 0;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> WindowSet {
        WindowSet::single(View::pane())
    }

    #[test]
    fn splits_and_cycles_in_layout_order() {
        let mut windows = set();
        assert!(windows.split(SplitDir::AboveBelow));
        assert_eq!(windows.len(), 2);
        assert_eq!(windows.layout().leaf_ids(), vec![0, 1]);
        assert_eq!(windows.active(), 0);

        assert!(windows.other());
        assert_eq!(windows.active(), 1);
        assert!(windows.other());
        assert_eq!(windows.active(), 0);

        // Splitting the first pane nests below it, not beside it.
        assert!(windows.split(SplitDir::LeftRight));
        assert_eq!(windows.len(), 3);
        assert_eq!(windows.layout().leaf_ids(), vec![0, 2, 1]);
    }

    #[test]
    fn split_windows_share_the_buffer_but_not_the_point() {
        let mut windows = set();
        windows.active_view_mut().point.row = 4;
        windows.active_view_mut().point.col = 2;
        windows.split(SplitDir::LeftRight);
        assert_eq!(windows.window(0).buffer, 0);
        assert_eq!(windows.window(1).buffer, 0);
        assert_eq!(windows.window(1).view.point.row, 4);
        assert_eq!(windows.window(1).view.point.col, 2);
        windows.window_mut(1).view.point.row = 10;
        assert_eq!(windows.window(0).view.point.row, 4);
    }

    #[test]
    fn windows_can_point_at_different_buffers() {
        let mut windows = set();
        windows.split(SplitDir::LeftRight);
        windows.window_mut(1).buffer = 3;
        assert_eq!(windows.windows_for_buffer(0), vec![0]);
        assert_eq!(windows.windows_for_buffer(3), vec![1]);
    }

    #[test]
    fn deleting_a_window_collapses_its_parent() {
        let mut windows = set();
        windows.split(SplitDir::LeftRight);
        windows.split(SplitDir::AboveBelow);
        assert_eq!(windows.len(), 3);
        assert_eq!(windows.active(), 0);

        windows.other();
        assert!(windows.delete_active());
        assert_eq!(windows.len(), 2);
        assert_eq!(windows.active(), 1);
        assert_eq!(windows.windows().len(), windows.layout().leaf_ids().len());
        assert!(windows.layout().leaf_ids().iter().all(|id| *id < 2));
    }

    #[test]
    fn delete_others_keeps_the_active_view() {
        let mut windows = set();
        windows.split(SplitDir::AboveBelow);
        windows.split(SplitDir::LeftRight);
        windows.other();
        let active_view = *windows.active_view();
        let active_buffer = windows.active_window().buffer;
        assert!(windows.delete_others());
        assert_eq!(windows.len(), 1);
        assert_eq!(windows.active(), 0);
        assert_eq!(*windows.active_view(), active_view);
        assert_eq!(windows.active_window().buffer, active_buffer);
    }

    #[test]
    fn the_last_window_is_not_deleted() {
        let mut windows = set();
        assert!(!windows.delete_active());
        assert!(!windows.other());
        assert!(!windows.delete_others());
        assert_eq!(windows.len(), 1);
    }

    #[test]
    fn split_cap_is_enforced() {
        let mut windows = set();
        for _ in 1..MAX_WINDOWS {
            assert!(windows.split(SplitDir::AboveBelow));
        }
        assert!(!windows.split(SplitDir::AboveBelow));
        assert_eq!(windows.len(), MAX_WINDOWS);
    }
}
