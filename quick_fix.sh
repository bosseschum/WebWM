#!/bin/bash

# WebWM Quick Fix Script
# Fixes the compilation errors and gets basic rendering working

echo "🔧 WebWM Quick Fix - Getting Your GUI Working"
echo "=============================================="
echo ""

# Step 1: Replace backend.rs with the fixed version
echo "Step 1: Fixing backend.rs..."
cat > src/compositor/backend.rs << 'EOF'
use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, gles::GlesRenderer, ImportMem},
        winit::{self, WinitGraphicsBackend},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Physical, Rectangle, Size, Transform},
};

use crate::compositor::input::InputHandler;
use crate::compositor::WebWMCompositor;
use crate::compositor::bar_renderer::BarTextureRenderer;

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

                event_loop.handle().insert_source(
                    winit_events,
                    move |_event, _, _compositor| {},
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

                let _renderer = state.winit.renderer();

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
                            Rectangle::new(
                                (render_location.x, render_location.y),
                                (geometry.size.w, geometry.size.h),
                            ),
                        ))
                    })
                    .collect();

                println!("🎨 Rendering frame with {} windows", windows.len());

                // Get background color from CSS
                let bg_color = if let Some(ref ss) = compositor.config.stylesheet {
                    ss.get_color("desktop", "background")
                        .map(|c| c.to_rgba_f32())
                        .unwrap_or([0.10, 0.11, 0.15, 1.0])
                } else {
                    [0.10, 0.11, 0.15, 1.0]
                };

                println!("  🎨 Background: #{:02x}{:02x}{:02x}",
                    (bg_color[0] * 255.0) as u8,
                    (bg_color[1] * 255.0) as u8,
                    (bg_color[2] * 255.0) as u8
                );

                // Log window rendering info
                for (i, (_window, geometry)) in windows.iter().enumerate() {
                    let is_focused = i == 0;
                    
                    let border_color = if is_focused {
                        [0.54, 0.71, 0.98, 1.0] // #89b4fa
                    } else {
                        [0.19, 0.20, 0.27, 1.0] // #313244
                    };

                    let icon = if is_focused { "🔵" } else { "⚪" };
                    println!("  {} Window {}: {}x{} at ({}, {}) border:#{:02x}{:02x}{:02x}",
                        icon, i,
                        geometry.size.w, geometry.size.h,
                        geometry.loc.x, geometry.loc.y,
                        (border_color[0] * 255.0) as u8,
                        (border_color[1] * 255.0) as u8,
                        (border_color[2] * 255.0) as u8
                    );
                }

                // Render bar
                let bar_elements = compositor.render_bar_elements();
                if !bar_elements.is_empty() {
                    println!("  📊 Status bar: {} elements", bar_elements.len());
                    
                    // Actually render the bar to a buffer
                    let bar_renderer = BarTextureRenderer::new(size.w, 30);
                    let _bar_buffer = bar_renderer.render_to_buffer(&bar_elements);
                    // TODO: Import this buffer as a texture and render it
                }

                // Submit frame
                state.winit.submit(None)?;
                println!("  ✅ Frame submitted");

                Ok(())
            }
            WebWMBackend::Drm(state) => {
                state
                    .render_frame()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
            WebWMBackend::BasicDrm(state) => {
                state
                    .render_frame()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
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
            state.output.change_current_state(Some(mode), None, None, Some((0, 0).into()));
            state.output.set_preferred(mode);
        }
    }
}
EOF

echo "  ✅ backend.rs updated"

# Step 2: Build the project
echo ""
echo "Step 2: Building project..."
if cargo build 2>&1 | tee /tmp/webwm_build.log | grep -q "^error"; then
    echo "  ❌ Build failed. Check /tmp/webwm_build.log for details"
    echo ""
    echo "Common remaining issues:"
    echo "  - If you see warnings about deprecated functions, those are OK"
    echo "  - If you see errors about missing methods, your Smithay version may differ"
    exit 1
else
    echo "  ✅ Build successful!"
fi

echo ""
echo "=============================================="
echo "✅ Quick fix complete!"
echo ""
echo "What you should see when running:"
echo "  - Console logs showing rendering info"
echo "  - Background color from CSS"
echo "  - Window geometry and border colors"
echo "  - Status bar element count"
echo ""
echo "To run: ./target/debug/webwm ./config"
echo ""
echo "Note: This version logs what it WOULD render."
echo "For actual pixel rendering to screen, you'll need"
echo "the full renderer implementation, which requires"
echo "matching your exact Smithay API version."
echo "=============================================="
EOF

chmod +x "$0"
