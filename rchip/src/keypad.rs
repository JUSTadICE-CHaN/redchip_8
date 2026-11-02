pub struct Keypad {
    keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Self { keys: [false; 16] }
    }

    pub fn is_pressed(&self, key: usize) -> bool {
        self.keys[key]
    }

    pub fn set_key(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
    }

    pub fn get_pressed_key(&self) -> Option<usize> {
        for key in 0..self.keys.len() {
            if self.keys[key] {
                return Some(key);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_press() {
        let mut keypad = Keypad::new();

        keypad.set_key(0, true);

        assert!(keypad.is_pressed(0));
        assert!(!keypad.is_pressed(1));
    }

    #[test]
    fn return_pressed_key() {
        let mut keypad = Keypad::new();

        assert_eq!(keypad.get_pressed_key(), None);

        keypad.set_key(0xA, true);

        assert_eq!(keypad.get_pressed_key(), Some(0xA));

        keypad.set_key(0xB, true);

        assert_eq!(keypad.get_pressed_key(), Some(0xA));
    }
}
