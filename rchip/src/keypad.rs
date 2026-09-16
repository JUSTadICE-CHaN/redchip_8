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
}
