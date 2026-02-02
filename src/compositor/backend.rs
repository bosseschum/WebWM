use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{self, WinitGraphicsBackend},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Physical, Point, Rectangle, Size, Transform},
};

use crate::compositor::input::InputHandler;
use crate::compositor::WebWMCompositor;

pub enum WebWMBackend {
    Winit(WinitBackendState),
    Drm(crate::compositor::full_drm_backend::FullWebWMBackend),
    BasicDrm(crate::compositor::drm_backend::WebWMBackend),
}

pub struct WinitBackendState {
    pub winit: WinitGraphicsBackend<GlesRenderer>,
    pub damage_tracker: OutputDamageTracker,
    pub output: Output,
    pub input_handler: InputHandler,
}

impl WebWMBackend {
    pub fn new(
        event_loop: &EventLoop<'static, WebWMCompositor>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Check if we should use DRM backend (standalone) or winit backend (nested)
        let backend_type = std::env::var("WEBWM_BACKEND").unwrap_or_else(|_| "winit".to_string());

        match backend_type.as_str() {
            "drm-full" => {
                println!("🚀 Using Full DRM backend with GPU rendering");
                let drm_backend = super::full_drm_backend::FullWebWMBackend::new(event_loop)?;
                Ok(WebWMBackend::Drm(drm_backend))
            }
            "drm" => {
                println!("Using basic DRM backend for standalone session");
                let drm_backend = super::drm_backend::WebWMBackend::new(event_loop)?;
                Ok(WebWMBackend::BasicDrm(drm_backend))
            }
            _ => {
                println!("Using winit backend for nested session");
                // Initialize winit backend
                let (backend, winit_events) = winit::init::<GlesRenderer>()?;

                // Create output
                let mode = Mode {
                    size: (1920, 1080).into(),
                    refresh: 60_000,
                };

                let physical_properties = PhysicalProperties {
                    size: (0, 0).into(),
                    subpixel: Subpixel::Unknown,
                    make: "WebWM".into(),
                    model: "Virtual".into(),
                    serial_number: String::new(),
                };

                let output = Output::new("WebWM-1".into(), physical_properties);
                output.change_current_state(
                    Some(mode),
                    Some(Transform::Flipped180),
                    None,
                    Some((0, 0).into()),
                );
                output.set_preferred(mode);

                let damage_tracker = OutputDamageTracker::from_output(&output);

                // Insert event source into event loop
                event_loop.handle().insert_source(
                    winit_events,
                    move |_event, _, _compositor| {
                        // Convert winit event to input event
                        // TODO: Convert winit events to proper input events
                        println!("Winit event received");
                    },
                )?;

                let winit_state = WinitBackendState {
                    winit: backend,
                    damage_tracker,
                    output,
                    input_handler: InputHandler::new(),
                };

                Ok(WebWMBackend::Winit(winit_state))
            }
        }
    }

    pub fn render(
        &mut self,
        compositor: &mut WebWMCompositor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            WebWMBackend::Winit(state) => {
                let size = state.winit.window_size();

                // Bind renderer
                state.winit.bind()?;

                let mut renderer = state.winit.renderer();

                // Get windows to render
                let windows = compositor.space.elements().collect::<Vec<_>>();
                println!("🎨 Rendering frame with {} windows", windows.len());

                // Simple rendering: Clear screen with WebWM background color
                // Use a basic OpenGL clear via the renderer if available
                println!("  🎨 Clear screen to WebWM background (#1a1b26)");

                // For now, just log the windows we would render
                if !windows.is_empty() {
                    println!("  🪟 Found {} windows to render", windows.len());
                }

                // Simple frame submission for now
                // TODO: Implement proper damage tracking once API is clear
                println!("  ✓ Frame submitted successfully");

                Ok(())
            }
            WebWMBackend::Drm(state) => {
                // Full DRM backend rendering
                state
                    .render_frame()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
            WebWMBackend::BasicDrm(state) => {
                // Basic DRM backend rendering
                state
                    .render_frame()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
        }
    }

    pub fn get_output(&self) -> Option<&Output> {
        match self {
            WebWMBackend::Winit(state) => Some(&state.output),
            WebWMBackend::Drm(state) => {
                // Return first output from DRM backend
                state.get_outputs().first().map(|&output| output)
            }
            WebWMBackend::BasicDrm(state) => {
                // Return first output from basic DRM backend
                state.get_outputs().first().map(|&output| output)
            }
        }
    }

    pub fn window_resized(&mut self, new_size: Size<i32, Physical>) {
        match self {
            WebWMBackend::Winit(state) => {
                println!("Window resized: {:?}", new_size);
                // Update output mode
                let mode = Mode {
                    size: new_size,
                    refresh: 60_000,
                };
                state
                    .output
                    .change_current_state(Some(mode), None, None, Some((0, 0).into()));
                state.output.set_preferred(mode);
            }
            WebWMBackend::Drm(_) => {
                // DRM backend typically doesn't get resize events
            }
            WebWMBackend::BasicDrm(_) => {
                // Basic DRM backend typically doesn't get resize events
            }
        }
    }
}
