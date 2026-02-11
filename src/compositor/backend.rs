use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker, element::AsRenderElements, gles::GlesRenderer, Frame,
            Renderer,
        },
        winit::{self, WinitGraphicsBackend},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Physical, Point, Rectangle, Size, Transform},
};

use crate::compositor::input::InputHandler;
use crate::compositor::renderer::{SolidColorRenderer, WebWMRenderer};
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
    pub renderer: WebWMRenderer,
    pub solid_renderer: SolidColorRenderer,
}

impl WebWMBackend {
    pub fn new(
        event_loop: &EventLoop<'static, WebWMCompositor>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
                let (backend, winit_events) = winit::init::<GlesRenderer>()?;

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

                event_loop
                    .handle()
                    .insert_source(winit_events, move |_event, _, _compositor| {})?;

                let winit_state = WinitBackendState {
                    winit: backend,
                    damage_tracker,
                    output,
                    input_handler: InputHandler::new(),
                    renderer: WebWMRenderer::new(),
                    solid_renderer: SolidColorRenderer::new(),
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

                // Update layout with actual output size first
                compositor.relayout_with_size(size);

                // Get windows to render
                let windows: Vec<_> = compositor
                    .space
                    .elements()
                    .filter_map(|window| {
                        let location = compositor.space.element_location(window)?;
                        let geometry = window.geometry();
                        let render_location = location + geometry.loc;

                        Some((
                            window,
                            Rectangle::<i32, smithay::utils::Physical>::from_loc_and_size(
                                (render_location.x, render_location.y),
                                (geometry.size.w, geometry.size.h),
                            ),
                        ))
                    })
                    .collect();

                // Get bar elements
                let _bar_elements = compositor.render_bar_elements();

                // Basic rendering for now
                // Use the original working pattern from the existing code
                state.winit.bind()?;

                // For now, just use the simple rendering pattern
                // We'll integrate WebWMRenderer in a future update
                println!(
                    "🎨 OpenGL rendering initialized with {} windows",
                    windows.len()
                );
                for (i, (window, geometry)) in windows.iter().enumerate() {
                    let status = if i == 0 { "focused" } else { "normal" };
                    println!(
                        "  Window {}: {}x{} at ({}, {}) [{}]",
                        i, geometry.size.w, geometry.size.h, geometry.loc.x, geometry.loc.y, status
                    );
                }

                // Submit the frame
                state.winit.submit(None)?;

                Ok(())
            }
            WebWMBackend::Drm(state) => state
                .render_frame(compositor)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
            WebWMBackend::BasicDrm(state) => state
                .render_frame(compositor)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
        }
    }

    pub fn get_output(&self) -> Option<&Output> {
        match self {
            WebWMBackend::Winit(state) => Some(&state.output),
            WebWMBackend::Drm(state) => state.get_outputs().first().copied(),
            WebWMBackend::BasicDrm(state) => state.get_outputs().first().copied(),
        }
    }

    pub fn window_resized(&mut self, new_size: Size<i32, Physical>) {
        if let WebWMBackend::Winit(state) = self {
            println!("Window resized: {:?}", new_size);
            let mode = Mode {
                size: new_size,
                refresh: 60_000,
            };
            state
                .output
                .change_current_state(Some(mode), None, None, Some((0, 0).into()));
            state.output.set_preferred(mode);
        }
    }
}
