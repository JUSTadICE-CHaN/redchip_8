pub struct Display {
    pixels: [bool; 64 * 32],
}

impl Display {
    pub fn new() -> Display {
        Self {
            pixels: [false; 64 * 32],
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [false; 64 * 32];
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> bool {
        let index = (y % 32) * 64 + (x % 64);
        let pixel_status = self.pixels[index];
        pixel_status
    }

    fn toggle_pixel(&mut self, x: usize, y: usize) -> bool {
        let index = (y % 32) * 64 + (x % 64);
        let was_on = self.pixels[index];

        self.pixels[index] = !was_on;
        was_on
    }

    fn draw_byte(&mut self, x: usize, y: usize, byte: u8) -> bool {
        let mut was_on_flag = false;
        for bit in 0..8 {
            let x = x + bit;
            let bit_value = (byte >> (7 - bit)) & 1;
            if bit_value == 1 {
                let was_on = self.toggle_pixel(x, y);
                was_on_flag |= was_on;
            }
        }
        was_on_flag
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut was_on_flag = false;
        for sprite_row in 0..sprite.len() {
            let y = y + sprite_row;
            let was_on_row = self.draw_byte(x, y, sprite[sprite_row]);
            was_on_flag |= was_on_row;
        }
        was_on_flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_clear() {
        let mut display = Display::new();

        assert_eq!(display.pixels, [false; 64 * 32]);

        display.pixels = [true; 64 * 32];

        display.clear();

        assert_eq!(display.pixels, [false; 64 * 32]);
    }

    #[test]
    fn toggle_pixels() {
        let mut display = Display::new();

        assert!(!display.toggle_pixel(10, 5));
        assert!(display.toggle_pixel(10, 5));
    }

    #[test]
    fn toggle_pixels_wrapping() {
        let mut display = Display::new();

        assert!(!display.toggle_pixel(64, 0));
        assert!(display.toggle_pixel(0, 0));

        assert!(!display.toggle_pixel(63, 32));
        assert!(display.toggle_pixel(63, 0));
    }

    #[test]
    fn draw_byte() {
        let mut display = Display::new();

        let byte = 0b1100_0000;

        // Draw the byte at (10, 5)
        assert!(!display.draw_byte(10, 5, byte));

        // Check pixel at (10, 5)
        assert!(display.pixels[5 * 64 + 10]);
        // Check pixel at (11, 5)
        assert!(display.pixels[5 * 64 + 11]);

        // Erase the byte at (10, 5)
        assert!(display.draw_byte(10, 5, byte));

        // Check pixel at (10, 5)
        assert!(!display.pixels[5 * 64 + 10]);
        // Check pixel at (11, 5)
        assert!(!display.pixels[5 * 64 + 11]);
    }

    #[test]
    fn draw_sprite() {
        let mut display = Display::new();

        let sprite: &[u8] = &[0b1100_0000, 0b0011_0000];

        // Draw the sprite at (10, 5)
        assert!(!display.draw_sprite(10, 5, sprite));

        // Check pixel at (10, 5)
        assert!(display.pixels[5 * 64 + 10]);
        // Check pixel at (11, 5)
        assert!(display.pixels[5 * 64 + 11]);

        // Check pixel at (12, 6)
        assert!(display.pixels[6 * 64 + 12]);
        // Check pixel at (13, 6)
        assert!(display.pixels[6 * 64 + 13]);

        // Erase the sprite at (10, 5)
        assert!(display.draw_sprite(10, 5, sprite));

        // Check pixel at (10, 5)
        assert!(!display.pixels[5 * 64 + 10]);
        // Check pixel at (11, 5)
        assert!(!display.pixels[5 * 64 + 11]);

        // Check pixel at (12, 6)
        assert!(!display.pixels[6 * 64 + 12]);
        // Check pixel at (13, 6)
        assert!(!display.pixels[6 * 64 + 13]);
    }
}
