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
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop,
        },
        wayland_server::Display,
    },
    utils::{Rectangle, Size, Transform},
};
use std::time::Duration;

use crate::compositor::{WebWMCompositor, ClientState};

pub struct WebWMBackend {
    pub winit: WinitGraphicsBackend<GlesRenderer>,
    pub damage_tracker: OutputDamageTracker,
    pub output: Output,
}

impl WebWMBackend {
    pub fn new() -> Result<(Self, EventLoop<'static, WebWMCompositor>), Box<dyn std::error::Error>> {
        let mut event_loop = EventLoop::try_new()?;
        
        // Initialize winit backend (creates window on your existing desktop for testing)
        let (backend, mut input) = winit::init::<GlesRenderer>()?;
        
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

        // Insert winit backend into event loop
        event_loop
            .handle()
            .insert_source(input, move |event, _, state| {
                state.handle_winit_event(event);
            })?;

        Ok((
            Self {
                winit: backend,
                damage_tracker,
                output,
            },
            event_loop,
        ))
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
        
        // Collect render elements from all windows
        let mut elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = Vec::new();

        for window in &compositor.windows {
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
                        Duration::ZERO,
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
    pub fn handle_winit_event(&mut self, event: WinitEvent) {
        match event {
            WinitEvent::Resized { size, .. } => {
                println!("Window resized: {:?}", size);
                // Update output mode
            }
            WinitEvent::Input(input_event) => {
                // Handle input events
                use smithay::backend::input::Event;
                
                match input_event {
                    smithay::backend::input::InputEvent::Keyboard { event } => {
                        use smithay::backend::input::KeyboardKeyEvent;
                        
                        let keycode = event.key_code();
                        let state = event.state();
                        
                        println!("Keyboard event: keycode={}, state={:?}", keycode, state);
                        
                        // Get keyboard handle
                        if let Some(keyboard) = self.seat.get_keyboard() {
                            keyboard.input::<(), _>(
                                self,
                                keycode,
                                state,
                                smithay::utils::SERIAL_COUNTER.next_serial(),
                                0,
                                |_, _, _| {
                                    smithay::input::keyboard::FilterResult::Forward
                                },
                            );
                        }
                    }
                    smithay::backend::input::InputEvent::PointerMotion { event } => {
                        // Handle pointer motion
                        use smithay::backend::input::PointerMotionEvent;
                        
                        let delta = event.delta();
                        println!("Pointer motion: {:?}", delta);
                    }
                    smithay::backend::input::InputEvent::PointerButton { event } => {
                        // Handle pointer button
                        use smithay::backend::input::PointerButtonEvent;
                        
                        let button = event.button_code();
                        let state = event.state();
                        println!("Pointer button: button={}, state={:?}", button, state);
                        
                        // Focus window under cursor
                        // TODO: Implement pointer focus
                    }
                    _ => {}
                }
            }
            WinitEvent::CloseRequested => {
                println!("Close requested");
                // TODO: Implement graceful shutdown
            }
            _ => {}
        }
    }
}
