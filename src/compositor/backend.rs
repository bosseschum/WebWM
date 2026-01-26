use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker,
            element::{
                surface::WaylandSurfaceRenderElement,
                AsRenderElements, RenderElement,
            },
            gles::GlesRenderer,
            Bind, Frame, Renderer,
        },
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    desktop::space::SpaceElement,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Rectangle, Size, Transform},
};

use crate::compositor::{WebWMCompositor, ClientState};
use crate::compositor::input::InputHandler;

pub struct WebWMBackend {
    pub winit: WinitGraphicsBackend<GlesRenderer>,
    pub damage_tracker: OutputDamageTracker,
    pub output: Output,
    pub input_handler: InputHandler,
}

impl WebWMBackend {
    pub fn new<F>(
        event_loop: &EventLoop<'static, WebWMCompositor>,
        mut event_handler: F,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        F: FnMut(WinitEvent, &mut WebWMCompositor) + 'static,
    {
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
        };

        let output = Output::new("WebWM-1".into(), physical_properties);
        output.create_global::<WebWMCompositor>(&backend.display().handle());
        output.change_current_state(Some(mode), Some(Transform::Flipped180), None, Some((0, 0).into()));
        output.set_preferred(mode);

        let damage_tracker = OutputDamageTracker::from_output(&output);

        // Insert event source into event loop
        event_loop
            .handle()
            .insert_source(winit_events, move |event, _, compositor| {
                event_handler(event, compositor);
            })?;

        Ok(Self {
            winit: backend,
            damage_tracker,
            output,
            input_handler: InputHandler::new(),
        })
    }

    pub fn render(
        &mut self,
        compositor: &mut WebWMCompositor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let size = self.winit.window_size();
        let scale = smithay::output::Scale::Integer(1);

        // Bind the renderer
        self.winit.bind()?;

        let mut renderer = self.winit.renderer();
        
        // Collect render elements from windows in active workspace only
        let mut elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = Vec::new();

        let active_workspace = compositor.workspace_manager.active_workspace();
        for window in &active_workspace.windows {
            let location = compositor.space.element_location(window).unwrap_or((0, 0).into());
            
            // Get render elements from the window
            let window_elements = window.render_elements::<WaylandSurfaceRenderElement<GlesRenderer>>(
                &mut renderer,
                location.to_physical_precise_round(scale),
                scale,
                1.0,
            );
            
            elements.extend(window_elements);
        }

        // Render
        let render_res = self.damage_tracker.render_output(
            &mut renderer,
            0,
            &elements,
            [0.1, 0.1, 0.1, 1.0], // Background color
        );

        match render_res {
            Ok(render_output_result) => {
                // Submit the frame
                self.winit.submit(Some(&render_output_result.damage))?;
                
                // Send frame callbacks to windows
                let time = compositor.clock.now();
                compositor.space.elements().for_each(|window| {
                    window.send_frame(
                        &self.output,
                        time,
                        std::time::Duration::ZERO,
                        |_, _| Some(self.output.clone()),
                    );
                });
            }
            Err(e) => {
                eprintln!("Render error: {:?}", e);
            }
        }

        Ok(())
    }
}

impl WebWMCompositor {
    pub fn handle_winit_event(&mut self, event: WinitEvent, input_handler: &mut InputHandler) {
        match event {
            WinitEvent::Resized { size, .. } => {
                println!("Window resized: {:?}", size);
            }
            WinitEvent::Input(input_event) => {
                // Use the input handler to process events
                input_handler.process_input_event(input_event, self);
            }
            WinitEvent::CloseRequested => {
                println!("Close requested - exiting");
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
