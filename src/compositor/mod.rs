pub mod backend;
pub mod input;
pub mod workspace;

use workspace::{WorkspaceManager, LayoutMode};

use smithay::{
    backend::renderer::element::RenderElement,
    delegate_compositor, delegate_data_device, delegate_output, delegate_seat,
    delegate_shm, delegate_xdg_shell,
    desktop::{
        PopupKind, PopupManager, Space, Window, WindowSurfaceType,
    },
    input::{
        keyboard::{KeyboardTarget, KeysymHandle, ModifiersState},
        pointer::{AxisFrame, ButtonEvent, MotionEvent, PointerTarget},
        Seat, SeatHandler, SeatState,
    },
    output::Output,
    reexports::{
        calloop::LoopHandle,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::{wl_seat, wl_surface::WlSurface},
            Client, Display, DisplayHandle, Resource,
        },
    },
    utils::{Clock, Monotonic, Rectangle, Size},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            get_parent, is_sync_subsurface, CompositorClientState, CompositorHandler,
            CompositorState,
        },
        data_device::{
            ClientDndGrabHandler, DataDeviceHandler, ServerDndGrabHandler,
        },
        output::OutputManagerState,
        seat::WaylandFocus,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::config::{Config, StyleSheet};

pub struct WebWMCompositor {
    pub display_handle: DisplayHandle,
    pub space: Space<Window>,
    pub clock: Clock<Monotonic>,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: smithay::wayland::data_device::DataDeviceState,
    pub output_manager_state: OutputManagerState,
    pub popup_manager: PopupManager,
    pub seat: Seat<Self>,
    pub workspace_manager: WorkspaceManager,
    pub config: Config,
    pub stylesheet: Option<StyleSheet>,
}

impl WebWMCompositor {
    pub fn new(
        display: &mut Display<Self>,
        loop_handle: LoopHandle<'static, Self>,
        config: Config,
    ) -> Self {
        let display_handle = display.handle();
        let clock = Clock::new();

        // Initialize Wayland globals
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let data_device_state = smithay::wayland::data_device::DataDeviceState::new::<Self>(&display_handle);

        // Create seat (keyboard and pointer)
        let mut seat = seat_state.new_wl_seat(&display_handle, "seat-0");
        seat.add_keyboard(Default::default(), 200, 25)
            .expect("Failed to add keyboard");
        seat.add_pointer();

        let space = Space::default();
        let popup_manager = PopupManager::default();

        let stylesheet = config.stylesheet.clone();
        
        // Initialize workspace manager
        let mut workspace_manager = WorkspaceManager::new();
        
        // Configure workspaces from config if available
        if let Some(ref desktop) = config.desktop {
            for ws_config in &desktop.workspaces {
                let layout_mode = LayoutMode::from(ws_config.layout.as_str());
                let mut workspace = workspace::Workspace::new(
                    ws_config.id,
                    ws_config.name.clone(),
                    layout_mode,
                );
                workspace_manager.add_workspace(workspace);
            }
        }

        Self {
            display_handle,
            space,
            clock,
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            data_device_state,
            output_manager_state,
            popup_manager,
            seat,
            workspace_manager,
            config,
            stylesheet,
        }
    }

    pub fn add_window(&mut self, toplevel: ToplevelSurface) {
        let window = Window::new(toplevel);
        
        // Check if window should go to specific workspace
        let target_workspace = self.get_target_workspace_for_window(&window);
        
        // Apply window rules from config
        self.apply_window_rules(&window);
        
        // Add to appropriate workspace
        if let Some(ws_id) = target_workspace {
            if let Some(workspace) = self.workspace_manager.get_workspace_mut(ws_id) {
                workspace.add_window(window.clone());
                println!("Window added to workspace {}: {} total windows in workspace", 
                         ws_id, workspace.len());
            }
        } else {
            // Add to active workspace
            self.workspace_manager.add_window_to_active(window.clone());
        }
        
        // Add to space (for rendering)
        self.space.map_element(window, (0, 0), false);
        
        // Relayout
        self.relayout();
    }

    fn get_target_workspace_for_window(&self, window: &Window) -> Option<u32> {
        if let Some(surface) = window.toplevel() {
            let app_id = surface.app_id().unwrap_or_default();
            
            // Check window rules for workspace assignment
            for rule in &self.config.window_rules {
                if rule.app_id == app_id {
                    if let Some(ws) = rule.workspace {
                        return Some(ws);
                    }
                }
            }
        }
        None
    }

    fn apply_window_rules(&self, window: &Window) {
        if let Some(surface) = window.toplevel() {
            let app_id = surface.app_id().unwrap_or_default();
            
            // Check config for matching rules
            for rule in &self.config.window_rules {
                if rule.app_id == app_id {
                    println!("Applied rule for app_id: {}", app_id);
                    
                    if let Some(workspace) = rule.workspace {
                        println!("  → Would move to workspace {}", workspace);
                        // TODO: Implement workspace management
                    }
                    
                    if let Some(floating) = rule.floating {
                        println!("  → Would set floating = {}", floating);
                        // TODO: Implement floating mode
                    }
                    
                    if let Some(ref class) = rule.css_class {
                        println!("  → Would apply CSS class: {}", class);
                        // TODO: Apply CSS styling
                    }
                }
            }
        }
    }

    pub fn remove_window(&mut self, toplevel: &ToplevelSurface) {
        // Find and remove the window
        let windows = self.workspace_manager.active_workspace().windows.clone();
        
        if let Some(window) = windows.iter()
            .find(|w| w.toplevel().map(|t| &t == toplevel).unwrap_or(false))
            .cloned()
        {
            self.space.unmap_elem(&window);
            self.workspace_manager.remove_window(&window);
            
            let active_ws = self.workspace_manager.active_workspace();
            println!("Window removed: {} remaining in workspace {}", 
                     active_ws.len(), active_ws.id);
            
            self.relayout();
        }
    }

    fn relayout(&mut self) {
        let active_workspace = self.workspace_manager.active_workspace();
        
        if active_workspace.is_empty() {
            return;
        }

        // Get output size (hardcoded for now, would detect actual output)
        let output_size = Size::from((1920, 1080));
        let gaps = self.config.layout.gaps as i32;
        
        match active_workspace.layout_mode {
            LayoutMode::Tiling => self.layout_tiling(output_size, gaps),
            LayoutMode::Floating => self.layout_floating(output_size),
            LayoutMode::Monocle => self.layout_monocle(output_size),
        }
    }

    fn layout_tiling(&mut self, output_size: Size<i32, smithay::utils::Physical>, gaps: i32) {
        let windows = &self.workspace_manager.active_workspace().windows;
        let window_count = windows.len();
        
        if window_count == 0 {
            return;
        }

        // Simple tiling: stack windows vertically
        let available_height = output_size.h - (gaps * (window_count as i32 + 1));
        let window_height = available_height / window_count as i32;
        let available_width = output_size.w - (gaps * 2);

        for (i, window) in windows.iter().enumerate() {
            let x = gaps;
            let y = gaps + (i as i32 * (window_height + gaps));
            
            // Position the window
            self.space.map_element(window.clone(), (x, y), false);
            
            // Request window to resize
            if let Some(toplevel) = window.toplevel() {
                toplevel.with_pending_state(|state| {
                    state.size = Some((available_width as u32, window_height as u32).into());
                });
                toplevel.send_configure();
            }
        }

        let active_ws = self.workspace_manager.active_workspace();
        println!("Relayout: {} windows in tiling mode on workspace {} (gaps: {}px)", 
                 window_count, active_ws.id, gaps);
    }

    fn layout_floating(&mut self, output_size: Size<i32, smithay::utils::Physical>) {
        let windows = &self.workspace_manager.active_workspace().windows;
        
        // Floating mode: center windows with offset
        let base_x = (output_size.w - 800) / 2;
        let base_y = (output_size.h - 600) / 2;

        for (i, window) in windows.iter().enumerate() {
            let offset = i as i32 * 30;
            let x = base_x + offset;
            let y = base_y + offset;
            
            self.space.map_element(window.clone(), (x, y), false);
            
            if let Some(toplevel) = window.toplevel() {
                toplevel.with_pending_state(|state| {
                    state.size = Some((800, 600).into());
                });
                toplevel.send_configure();
            }
        }

        let active_ws = self.workspace_manager.active_workspace();
        println!("Relayout: {} windows in floating mode on workspace {}", 
                 windows.len(), active_ws.id);
    }

    fn layout_monocle(&mut self, output_size: Size<i32, smithay::utils::Physical>) {
        let windows = &self.workspace_manager.active_workspace().windows;
        let focused_idx = self.workspace_manager.active_workspace().focused_window_idx;
        
        // Monocle: fullscreen the focused window, hide others
        if let Some(idx) = focused_idx {
            if let Some(window) = windows.get(idx) {
                self.space.map_element(window.clone(), (0, 0), false);
                
                if let Some(toplevel) = window.toplevel() {
                    toplevel.with_pending_state(|state| {
                        state.size = Some((output_size.w as u32, output_size.h as u32).into());
                    });
                    toplevel.send_configure();
                }
            }
        }

        let active_ws = self.workspace_manager.active_workspace();
        println!("Relayout: monocle mode on workspace {} (focused window fullscreen)", 
                 active_ws.id);
    }

    pub fn handle_keyboard_input(&mut self, keycode: u32, modifiers: ModifiersState) {
        println!("Key pressed: {} (mods: {:?})", keycode, modifiers);
        
        // TODO: Match against keybindings from config
        // Execute corresponding actions
    }

    pub fn get_border_color(&self, window: &Window, focused: bool) -> [f32; 4] {
        if let Some(ref stylesheet) = self.stylesheet {
            let selector = if focused { "window:focus" } else { "window" };
            
            if let Some(color) = stylesheet.get_color(selector, "border-color") {
                return color.to_rgba_f32();
            }
        }

        // Fallback to config theme
        let hex_color = if focused {
            &self.config.theme.border_focused
        } else {
            &self.config.theme.border_normal
        };

        parse_hex_color(hex_color)
    }

    pub fn get_border_width(&self) -> u32 {
        if let Some(ref stylesheet) = self.stylesheet {
            if let Some(width) = stylesheet.get_length("window", "border-width") {
                return width as u32;
            }
        }

        self.config.layout.border_width
    }
}

fn parse_hex_color(hex: &str) -> [f32; 4] {
    let hex = hex.trim_start_matches('#');
    
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        [r, g, b, 1.0]
    } else {
        [0.5, 0.5, 0.5, 1.0] // Gray fallback
    }
}

// Smithay delegate implementations
delegate_compositor!(WebWMCompositor);
delegate_xdg_shell!(WebWMCompositor);
delegate_shm!(WebWMCompositor);
delegate_seat!(WebWMCompositor);
delegate_data_device!(WebWMCompositor);
delegate_output!(WebWMCompositor);

// Implement required traits
impl BufferHandler for WebWMCompositor {
    fn buffer_destroyed(&mut self, _buffer: &smithay::wayland::buffer::Buffer) {}
}

impl CompositorHandler for WebWMCompositor {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        self.space.commit(surface);
        self.popup_manager.commit(surface);
    }
}

impl XdgShellHandler for WebWMCompositor {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        println!("New toplevel window created");
        self.add_window(surface);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        println!("Toplevel window destroyed");
        self.remove_window(&surface);
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        self.popup_manager.track_popup(PopupKind::Xdg(surface)).ok();
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: smithay::utils::Serial) {
        // Handle popup grabs
    }
}

impl ShmHandler for WebWMCompositor {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for WebWMCompositor {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        // Handle focus changes
        if let Some(surface) = focused {
            println!("Focus changed to surface");
        }
    }

    fn cursor_image(&mut self, seat: &Seat<Self>, image: smithay::input::pointer::CursorImageStatus) {
        // Handle cursor image changes
    }
}

impl DataDeviceHandler for WebWMCompositor {
    fn data_device_state(&self) -> &smithay::wayland::data_device::DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for WebWMCompositor {}
impl ServerDndGrabHandler for WebWMCompositor {}

pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
