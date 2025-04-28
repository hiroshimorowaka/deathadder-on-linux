pub enum KeyboardButtons {
    F23,
    F24,
}

impl KeyboardButtons {
    pub fn from_code(code: u8) -> Option<KeyboardButtons> {
        match code {
            114 => Some(KeyboardButtons::F23),
            115 => Some(KeyboardButtons::F24),
            _ => None,
        }
    }
}
