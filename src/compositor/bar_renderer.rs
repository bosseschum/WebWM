use smithay::backend::renderer::{
    gles::{GlesRenderer, GlesTexture},
    Frame, Renderer, Texture,
};
use smithay::utils::{Physical, Rectangle, Size};

use crate::compositor::bar::BarElement;

pub struct BarTextureRenderer {
    width: i32,
    height: i32,
}

impl BarTextureRenderer {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Render bar elements to a RGBA buffer
    pub fn render_to_buffer(&self, elements: &[BarElement]) -> Vec<u8> {
        let size = (self.width * self.height * 4) as usize;
        let mut buffer = vec![0u8; size];

        for element in elements {
            match element {
                BarElement::Rectangle { geometry, color } => {
                    self.draw_rectangle(&mut buffer, geometry, *color);
                }
                BarElement::Circle { center, radius, color } => {
                    self.draw_circle(&mut buffer, *center, *radius, *color);
                }
                BarElement::Text { position, text, color, size } => {
                    self.draw_text(&mut buffer, *position, text, *color, *size);
                }
            }
        }

        buffer
    }

    fn draw_rectangle(&self, buffer: &mut [u8], geometry: &Rectangle<i32, Physical>, color: [f32; 4]) {
        let x = geometry.loc.x;
        let y = geometry.loc.y;
        let w = geometry.size.w;
        let h = geometry.size.h;

        for py in y..y + h {
            if py < 0 || py >= self.height {
                continue;
            }

            for px in x..x + w {
                if px < 0 || px >= self.width {
                    continue;
                }

                self.set_pixel(buffer, px, py, color);
            }
        }
    }

    fn draw_circle(&self, buffer: &mut [u8], center: (i32, i32), radius: i32, color: [f32; 4]) {
        let (cx, cy) = center;

        for y in cy - radius..=cy + radius {
            if y < 0 || y >= self.height {
                continue;
            }

            for x in cx - radius..=cx + radius {
                if x < 0 || x >= self.width {
                    continue;
                }

                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= radius * radius {
                    self.set_pixel(buffer, x, y, color);
                }
            }
        }
    }

    fn draw_text(&self, buffer: &mut [u8], position: (i32, i32), text: &str, color: [f32; 4], _size: u32) {
        // Simple bitmap font rendering
        // This is a very basic 5x7 font for ASCII characters
        
        let (mut x, y) = position;

        for ch in text.chars() {
            if ch.is_ascii() {
                self.draw_char(buffer, x, y, ch, color);
                x += 6; // Character width + spacing
            }
        }
    }

    fn draw_char(&self, buffer: &mut [u8], x: i32, y: i32, ch: char, color: [f32; 4]) {
        // Get bitmap for character (5x7)
        let bitmap = get_char_bitmap(ch);

        for row in 0..7 {
            if y + row < 0 || y + row >= self.height {
                continue;
            }

            for col in 0..5 {
                if x + col < 0 || x + col >= self.width {
                    continue;
                }

                if bitmap[row as usize] & (1 << (4 - col)) != 0 {
                    self.set_pixel(buffer, x + col, y + row, color);
                }
            }
        }
    }

    fn set_pixel(&self, buffer: &mut [u8], x: i32, y: i32, color: [f32; 4]) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }

        let idx = ((y * self.width + x) * 4) as usize;
        
        if idx + 3 < buffer.len() {
            // Alpha blend
            let src_alpha = color[3];
            let dst_alpha = buffer[idx + 3] as f32 / 255.0;
            let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

            if out_alpha > 0.0 {
                buffer[idx] = ((color[0] * src_alpha + buffer[idx] as f32 / 255.0 * dst_alpha * (1.0 - src_alpha)) / out_alpha * 255.0) as u8;
                buffer[idx + 1] = ((color[1] * src_alpha + buffer[idx + 1] as f32 / 255.0 * dst_alpha * (1.0 - src_alpha)) / out_alpha * 255.0) as u8;
                buffer[idx + 2] = ((color[2] * src_alpha + buffer[idx + 2] as f32 / 255.0 * dst_alpha * (1.0 - src_alpha)) / out_alpha * 255.0) as u8;
                buffer[idx + 3] = (out_alpha * 255.0) as u8;
            }
        }
    }
}

// Simple 5x7 bitmap font for ASCII characters
fn get_char_bitmap(ch: char) -> [u8; 7] {
    match ch {
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        
        'A' | 'a' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' | 'b' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' | 'c' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' | 'd' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' | 'e' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' | 'f' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' | 'g' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
        'H' | 'h' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' | 'i' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' | 'j' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        'K' | 'k' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' | 'l' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' | 'm' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' | 'n' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        'O' | 'o' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' | 'p' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' | 'q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' | 'r' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' | 's' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' | 't' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' | 'u' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' | 'v' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' | 'w' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        'X' | 'x' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' | 'y' => [0x11, 0x11, 0x11, 0x0A, 0x04, 0x04, 0x04],
        'Z' | 'z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x04],
        
        _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // Unknown character
    }
}
