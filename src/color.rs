pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, u8::MAX)
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn red() -> Self {
        Self::from_rgb(0xFF, 0, 0)
    }

    pub fn green() -> Self {
        Self::from_rgb(0, 0xFF, 0)
    }

    pub fn blue() -> Self {
        Self::from_rgb(0, 0, 0xFF)
    }

    pub fn black() -> Color {
        Self::from_rgb(0, 0, 0)
    }

    pub fn white() -> Self {
        Self::from_rgb(0xFF, 0xFF, 0xFF)
    }

    fn to_rgb565(&self) -> u16 {
        let r = (self.r as u16 >> 3) & 0x1F;
        let g = (self.g as u16 >> 2) & 0x3F;
        let b = (self.b as u16 >> 3) & 0x1F;
        (r << 11) | (g << 5) | b
    }

    fn to_rgb(&self) -> u32 {
        #[cfg(feature = "ltr-rgb")]
        return u32::from_le_bytes([self.r, self.g, self.b, self.a]);

        #[cfg(not(feature = "ltr-rgb"))]
        return u32::from_le_bytes([self.b, self.g, self.r, self.a]);
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
