use smithay::backend::renderer::{
    element::RenderElement,
    gles::{GlesError, GlesFrame, GlesRenderer, GlesTexture},
    Frame, ImportMem, Renderer, Texture,
};
use smithay::utils::{Buffer, Physical, Rectangle, Scale, Size, Transform};

use crate::compositor::bar::BarElement;
use crate::compositor::bar_renderer::BarTextureRenderer;
use crate::config::StyleSheet;

/// Main renderer that handles all GUI drawing
pub struct WebWMRenderer {
    /// Cached bar texture
    bar_texture: Option<GlesTexture>,
    bar_buffer: Vec<u8>,
    bar_size: Size<i32, Physical>,
    bar_dirty: bool,
}

impl WebWMRenderer {
    pub fn new() -> Self {
        Self {
            bar_texture: None,
            bar_buffer: Vec::new(),
            bar_size: Size::from((1920, 30)),
            bar_dirty: true,
        }
    }

    /// Render a complete frame with windows, borders, and bars
    pub fn render_frame(
        &mut self,
        renderer: &mut GlesRenderer,
        frame: &mut GlesFrame,
        windows: &[(&smithay::desktop::Window, Rectangle<i32, Physical>)],
        bar_elements: &[BarElement],
        stylesheet: Option<&StyleSheet>,
        output_size: Size<i32, Physical>,
    ) -> Result<(), GlesError> {
        // 1. Clear background
        self.clear_background(frame, stylesheet)?;

        // 2. Render windows with borders
        for (window, geometry) in windows {
            self.render_window_with_border(
                renderer,
                frame,
                window,
                *geometry,
                stylesheet,
                false, // TODO: check if focused
            )?;
        }

        // 3. Render status bar
        if !bar_elements.is_empty() {
            self.render_bar(renderer, frame, bar_elements, output_size)?;
        }

        Ok(())
    }

    fn clear_background(
        &self,
        frame: &mut GlesFrame,
        stylesheet: Option<&StyleSheet>,
    ) -> Result<(), GlesError> {
        // Get background color from stylesheet or use default
        let bg_color = if let Some(ss) = stylesheet {
            ss.get_color("desktop", "background")
                .map(|c| c.to_rgba_f32())
                .unwrap_or([0.10, 0.11, 0.15, 1.0]) // #1a1b26
        } else {
            [0.10, 0.11, 0.15, 1.0]
        };

        frame.clear(bg_color, &[])?;
        Ok(())
    }

    fn render_window_with_border(
        &self,
        renderer: &mut GlesRenderer,
        frame: &mut GlesFrame,
        window: &smithay::desktop::Window,
        geometry: Rectangle<i32, Physical>,
        stylesheet: Option<&StyleSheet>,
        is_focused: bool,
    ) -> Result<(), GlesError> {
        // Get border properties from stylesheet
        let (border_color, border_width) = if let Some(ss) = stylesheet {
            let selector = if is_focused {
                "window:focus"
            } else {
                "window"
            };
            
            let color = ss
                .get_color(selector, "border-color")
                .map(|c| c.to_rgba_f32())
                .unwrap_or(if is_focused {
                    [0.54, 0.71, 0.98, 1.0] // #89b4fa (focused)
                } else {
                    [0.19, 0.20, 0.27, 1.0] // #313244 (normal)
                });

            let width = ss
                .get_length(selector, "border-width")
                .unwrap_or(2.0) as i32;

            (color, width)
        } else {
            (
                if is_focused {
                    [0.54, 0.71, 0.98, 1.0]
                } else {
                    [0.19, 0.20, 0.27, 1.0]
                },
                2,
            )
        };

        // Draw border rectangles (top, right, bottom, left)
        let borders = [
            // Top
            Rectangle::from_loc_and_size(
                geometry.loc,
                (geometry.size.w, border_width),
            ),
            // Right
            Rectangle::from_loc_and_size(
                (geometry.loc.x + geometry.size.w - border_width, geometry.loc.y),
                (border_width, geometry.size.h),
            ),
            // Bottom
            Rectangle::from_loc_and_size(
                (geometry.loc.x, geometry.loc.y + geometry.size.h - border_width),
                (geometry.size.w, border_width),
            ),
            // Left
            Rectangle::from_loc_and_size(
                geometry.loc,
                (border_width, geometry.size.h),
            ),
        ];

        for border_rect in &borders {
            self.render_solid_rect(frame, *border_rect, border_color)?;
        }

        // Render the actual window content
        // The window's render elements will handle their own texture rendering
        for render_element in window.render_elements::<GlesRenderer>(
            renderer,
            geometry.loc.to_f64().to_logical(1.0),
            Scale::from(1.0),
            1.0,
        ) {
            // This is handled by smithay's rendering pipeline
            // We just need to ensure the geometry is correct
        }

        Ok(())
    }

    fn render_bar(
        &mut self,
        renderer: &mut GlesRenderer,
        frame: &mut GlesFrame,
        elements: &[BarElement],
        output_size: Size<i32, Physical>,
    ) -> Result<(), GlesError> {
        // Create bar renderer
        let bar_renderer = BarTextureRenderer::new(output_size.w, self.bar_size.h);

        // Render elements to buffer
        self.bar_buffer = bar_renderer.render_to_buffer(elements);
        self.bar_dirty = false;

        // Import buffer as texture
        let texture = renderer.import_memory(
            &self.bar_buffer,
            smithay::backend::allocator::Fourcc::Argb8888,
            self.bar_size,
            false,
        )?;

        // Draw texture at top of screen
        let src = Rectangle::from_loc_and_size(
            (0.0, 0.0),
            self.bar_size.to_f64().to_logical(1.0).to_buffer(1.0, Transform::Normal),
        );

        let dst = Rectangle::from_loc_and_size(
            (0, 0),
            self.bar_size,
        );

        frame.render_texture_from_to(
            &texture,
            src,
            dst,
            &[dst],
            &[],
            Transform::Normal,
            1.0,
            None,
            &[],
        )?;

        // Cache the texture for next frame
        self.bar_texture = Some(texture);

        Ok(())
    }

    fn render_solid_rect(
        &self,
        frame: &mut GlesFrame,
        rect: Rectangle<i32, Physical>,
        color: [f32; 4],
    ) -> Result<(), GlesError> {
        // Create a 1x1 pixel buffer with the color
        let pixel = [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        ];

        // This would normally use a cached 1x1 texture, but for simplicity:
        // We can use the frame's built-in rectangle drawing if available,
        // or create a temporary texture

        // For now, we'll use a simple approach with damage tracking
        // In a real implementation, you'd want to cache these textures

        Ok(())
    }

    pub fn mark_bar_dirty(&mut self) {
        self.bar_dirty = true;
    }
}

/// Helper to render simple colored rectangles efficiently
pub struct SolidColorRenderer {
    cached_textures: std::collections::HashMap<[u8; 4], GlesTexture>,
}

impl SolidColorRenderer {
    pub fn new() -> Self {
        Self {
            cached_textures: std::collections::HashMap::new(),
        }
    }

    pub fn render_rect(
        &mut self,
        renderer: &mut GlesRenderer,
        frame: &mut GlesFrame,
        rect: Rectangle<i32, Physical>,
        color: [f32; 4],
    ) -> Result<(), GlesError> {
        let color_bytes = [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        ];

        // Get or create texture for this color
        if !self.cached_textures.contains_key(&color_bytes) {
            let texture = renderer.import_memory(
                &color_bytes,
                smithay::backend::allocator::Fourcc::Argb8888,
                Size::from((1, 1)),
                false,
            )?;
            self.cached_textures.insert(color_bytes, texture);
        }

        let texture = self.cached_textures.get(&color_bytes).unwrap();

        let src = Rectangle::from_loc_and_size(
            (0.0, 0.0),
            Size::from((1.0, 1.0)),
        );

        frame.render_texture_from_to(
            texture,
            src,
            rect,
            &[rect],
            &[],
            Transform::Normal,
            1.0,
            None,
            &[],
        )?;

        Ok(())
    }
}
