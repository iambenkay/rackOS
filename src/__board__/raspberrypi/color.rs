pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba(r, g, b, u8::MAX)
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    pub fn red() -> Color {
        Color::from_rgb(0xFF, 0, 0)
    }

    pub fn green() -> Color {
        Color::from_rgb(0, 0xFF, 0)
    }

    pub fn blue() -> Color {
        Color::from_rgb(0, 0, 0xFF)
    }

    pub fn black() -> Color {
        Color::from_rgb(0, 0, 0)
    }

    pub fn white() -> Color {
        Color::from_rgb(0xFF, 0xFF, 0xFF)
    }

    fn to_rgb565(&self) -> u16 {
        let r = (self.r as u16 >> 3) & 0x1F;
        let g = (self.g as u16 >> 2) & 0x3F;
        let b = (self.b as u16 >> 3) & 0x1F;
        (r << 11) | (g << 5) | b
    }

    fn to_rgb(&self) -> u32 {
        u32::from_le_bytes([self.r, self.g, self.b, self.a])
    }
}

impl From<&Color> for u32 {
    fn from(value: &Color) -> Self {
        value.to_rgb()
    }
}

impl From<&Color> for u16 {
    fn from(value: &Color) -> Self {
        value.to_rgb565()
    }
}
