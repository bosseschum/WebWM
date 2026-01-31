use smithay::desktop::Window;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: u32,
    pub name: String,
    pub layout_mode: LayoutMode,
    pub windows: Vec<Window>,
    pub focused_window_idx: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
    Tiling,
    Floating,
    Monocle,
}

impl Workspace {
    pub fn new(id: u32, name: String, layout_mode: LayoutMode) -> Self {
        Self {
            id,
            name,
            layout_mode,
            windows: Vec::new(),
            focused_window_idx: None,
        }
    }

    pub fn add_window(&mut self, window: Window) {
        self.windows.push(window);

        // Focus the newly added window
        if self.focused_window_idx.is_none() {
            self.focused_window_idx = Some(0);
        } else {
            self.focused_window_idx = Some(self.windows.len() - 1);
        }
    }

    pub fn remove_window(&mut self, window: &Window) -> bool {
        if let Some(idx) = self.windows.iter().position(|w| w == window) {
            self.windows.remove(idx);

            // Adjust focused window index
            if let Some(focused) = self.focused_window_idx {
                if focused >= self.windows.len() {
                    self.focused_window_idx = if self.windows.is_empty() {
                        None
                    } else {
                        Some(self.windows.len() - 1)
                    };
                }
            }

            true
        } else {
            false
        }
    }

    pub fn focused_window(&self) -> Option<&Window> {
        self.focused_window_idx
            .and_then(|idx| self.windows.get(idx))
    }

    pub fn focused_window_mut(&mut self) -> Option<&mut Window> {
        self.focused_window_idx
            .and_then(|idx| self.windows.get_mut(idx))
    }

    pub fn focus_next(&mut self) {
        if self.windows.is_empty() {
            self.focused_window_idx = None;
            return;
        }

        let current = self.focused_window_idx.unwrap_or(0);
        let next = (current + 1) % self.windows.len();
        self.focused_window_idx = Some(next);
    }

    pub fn focus_prev(&mut self) {
        if self.windows.is_empty() {
            self.focused_window_idx = None;
            return;
        }

        let current = self.focused_window_idx.unwrap_or(0);
        let prev = if current == 0 {
            self.windows.len() - 1
        } else {
            current - 1
        };
        self.focused_window_idx = Some(prev);
    }

    pub fn focus_window(&mut self, window: &Window) -> bool {
        if let Some(idx) = self.windows.iter().position(|w| w == window) {
            self.focused_window_idx = Some(idx);
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.windows.len()
    }
}

pub struct WorkspaceManager {
    workspaces: HashMap<u32, Workspace>,
    active_workspace_id: u32,
    workspace_order: Vec<u32>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        let mut manager = Self {
            workspaces: HashMap::new(),
            active_workspace_id: 1,
            workspace_order: Vec::new(),
        };

        // Create default workspaces (1-9)
        for i in 1..=9 {
            manager.add_workspace(Workspace::new(i, format!("{}", i), LayoutMode::Tiling));
        }

        manager
    }

    pub fn add_workspace(&mut self, workspace: Workspace) {
        let id = workspace.id;
        self.workspaces.insert(id, workspace);
        if !self.workspace_order.contains(&id) {
            self.workspace_order.push(id);
        }
    }

    pub fn get_workspace(&self, id: u32) -> Option<&Workspace> {
        self.workspaces.get(&id)
    }

    pub fn get_workspace_mut(&mut self, id: u32) -> Option<&mut Workspace> {
        self.workspaces.get_mut(&id)
    }

    pub fn active_workspace(&self) -> &Workspace {
        self.workspaces
            .get(&self.active_workspace_id)
            .expect("Active workspace must exist")
    }

    pub fn active_workspace_mut(&mut self) -> &mut Workspace {
        self.workspaces
            .get_mut(&self.active_workspace_id)
            .expect("Active workspace must exist")
    }

    pub fn active_workspace_id(&self) -> u32 {
        self.active_workspace_id
    }

    pub fn switch_to_workspace(&mut self, id: u32) -> bool {
        if self.workspaces.contains_key(&id) {
            println!("Switching to workspace {}", id);
            self.active_workspace_id = id;
            true
        } else {
            println!("Workspace {} does not exist", id);
            false
        }
    }

    pub fn move_window_to_workspace(&mut self, window: Window, target_workspace_id: u32) -> bool {
        // Find which workspace currently has this window
        let mut source_workspace_id = None;

        for (ws_id, workspace) in &self.workspaces {
            if workspace.windows.iter().any(|w| w == &window) {
                source_workspace_id = Some(*ws_id);
                break;
            }
        }

        if let Some(source_id) = source_workspace_id {
            // Remove from source workspace
            if let Some(source_ws) = self.workspaces.get_mut(&source_id) {
                source_ws.remove_window(&window);
            }

            // Add to target workspace
            if let Some(target_ws) = self.workspaces.get_mut(&target_workspace_id) {
                target_ws.add_window(window);
                println!(
                    "Moved window from workspace {} to {}",
                    source_id, target_workspace_id
                );
                return true;
            }
        }

        false
    }

    pub fn add_window_to_active(&mut self, window: Window) {
        let active_id = self.active_workspace_id;
        if let Some(workspace) = self.workspaces.get_mut(&active_id) {
            workspace.add_window(window);
            println!("Added window to workspace {}", active_id);
        }
    }

    pub fn remove_window(&mut self, window: &Window) -> bool {
        // Try to remove from any workspace
        for workspace in self.workspaces.values_mut() {
            if workspace.remove_window(window) {
                println!("Removed window from workspace {}", workspace.id);
                return true;
            }
        }
        false
    }

    pub fn find_window_workspace(&self, window: &Window) -> Option<u32> {
        for (id, workspace) in &self.workspaces {
            if workspace.windows.iter().any(|w| w == window) {
                return Some(*id);
            }
        }
        None
    }

    pub fn focused_window(&self) -> Option<&Window> {
        self.active_workspace().focused_window()
    }

    pub fn focused_window_mut(&mut self) -> Option<&mut Window> {
        self.active_workspace_mut().focused_window_mut()
    }

    pub fn focus_next_window(&mut self) {
        self.active_workspace_mut().focus_next();
    }

    pub fn focus_prev_window(&mut self) {
        self.active_workspace_mut().focus_prev();
    }

    pub fn all_workspaces(&self) -> Vec<&Workspace> {
        self.workspace_order
            .iter()
            .filter_map(|id| self.workspaces.get(id))
            .collect()
    }

    pub fn get_workspace_windows(&self, id: u32) -> Vec<&Window> {
        self.workspaces
            .get(&id)
            .map(|ws| ws.windows.iter().collect())
            .unwrap_or_default()
    }

    pub fn workspace_count(&self) -> usize {
        self.workspaces.len()
    }

    pub fn cycle_workspace_next(&mut self) {
        let current_pos = self
            .workspace_order
            .iter()
            .position(|&id| id == self.active_workspace_id)
            .unwrap_or(0);

        let next_pos = (current_pos + 1) % self.workspace_order.len();
        let next_id = self.workspace_order[next_pos];

        self.switch_to_workspace(next_id);
    }

    pub fn cycle_workspace_prev(&mut self) {
        let current_pos = self
            .workspace_order
            .iter()
            .position(|&id| id == self.active_workspace_id)
            .unwrap_or(0);

        let prev_pos = if current_pos == 0 {
            self.workspace_order.len() - 1
        } else {
            current_pos - 1
        };
        let prev_id = self.workspace_order[prev_pos];

        self.switch_to_workspace(prev_id);
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for LayoutMode {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tiling" => LayoutMode::Tiling,
            "floating" => LayoutMode::Floating,
            "monocle" => LayoutMode::Monocle,
            _ => LayoutMode::Tiling,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let ws = Workspace::new(1, "main".to_string(), LayoutMode::Tiling);
        assert_eq!(ws.id, 1);
        assert_eq!(ws.name, "main");
        assert!(ws.is_empty());
    }

    #[test]
    fn test_workspace_manager() {
        let mut manager = WorkspaceManager::new();

        // Should have 9 default workspaces
        assert_eq!(manager.workspace_count(), 9);

        // Should start on workspace 1
        assert_eq!(manager.active_workspace_id(), 1);

        // Switch to workspace 2
        assert!(manager.switch_to_workspace(2));
        assert_eq!(manager.active_workspace_id(), 2);

        // Try to switch to non-existent workspace
        assert!(!manager.switch_to_workspace(99));
    }

    #[test]
    fn test_workspace_cycling() {
        let mut manager = WorkspaceManager::new();

        // Start at 1
        assert_eq!(manager.active_workspace_id(), 1);

        // Cycle forward
        manager.cycle_workspace_next();
        assert_eq!(manager.active_workspace_id(), 2);

        // Cycle backward
        manager.cycle_workspace_prev();
        assert_eq!(manager.active_workspace_id(), 1);

        // Cycle backward from 1 should wrap to 9
        manager.cycle_workspace_prev();
        assert_eq!(manager.active_workspace_id(), 9);
    }
}
