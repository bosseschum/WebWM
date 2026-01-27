use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker,
            element::{
                surface::WaylandSurfaceRenderElement,
                AsRenderElements, RenderElement,
            },
            gles::GlesRenderer,
            Bind, Frame, Renderer, ImportMem,
        },
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    desktop::space::SpaceElement,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Rectangle, Size, Transform, Physical},
};

use crate::compositor::{WebWMCompositor, ClientState};
use crate::compositor::input::InputHandler;
use crate::compositor::bar_renderer::BarTextureRenderer;
use crate::compositor::bar_element::BarRenderElement;

pub struct WebWMBackend {
    pub winit: WinitGraphicsBackend<GlesRenderer>,
    pub damage_tracker: OutputDamageTracker,
    pub output: Output,
    pub input_handler: InputHandler,
    pub bar_element: Option<BarRenderElement>,
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
            bar_element: None,
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
        let mut elements: Vec<Box<dyn RenderElement<GlesRenderer>>> = Vec::new();

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
            
            for elem in window_elements {
                elements.push(Box::new(elem));
            }
        }

        // Render the bar
        let bar_elements = compositor.render_bar_elements();
        if !bar_elements.is_empty() {
            let bar_height = compositor.bar_height();
            if bar_height > 0 {
                // Render bar to buffer
                let bar_renderer = BarTextureRenderer::new(size.w, bar_height);
                let bar_buffer = bar_renderer.render_to_buffer(&bar_elements);
                
                let bar_size = Size::from((size.w, bar_height));
                let bar_geometry = Rectangle::from_loc_and_size((0, 0), bar_size);
                
                // Create or update bar element
                if let Some(ref mut bar_elem) = self.bar_element {
                    // Update existing bar element
                    if let Err(e) = bar_elem.update(&mut renderer, &bar_buffer, bar_size) {
                        eprintln!("Failed to update bar texture: {:?}", e);
                    } else {
                        elements.push(Box::new(bar_elem.clone()) as Box<dyn RenderElement<GlesRenderer>>);
                    }
                } else {
                    // Create new bar element
                    match BarRenderElement::new(&mut renderer, &bar_buffer, bar_size, bar_geometry) {
                        Ok(bar_elem) => {
                            elements.push(Box::new(bar_elem.clone()) as Box<dyn RenderElement<GlesRenderer>>);
                            self.bar_element = Some(bar_elem);
                        }
                        Err(e) => {
                            eprintln!("Failed to create bar texture: {:?}", e);
                        }
                    }
                }
            }
        }

        // Convert to trait object references
        let render_elements: Vec<&dyn RenderElement<GlesRenderer>> = elements
            .iter()
            .map(|e| e.as_ref() as &dyn RenderElement<GlesRenderer>)
            .collect();

        // Render
        let render_res = self.damage_tracker.render_output(
            &mut renderer,
            0,
            &render_elements,
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
