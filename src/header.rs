#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct QOIHeader {
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
}

impl QOIHeader {
    pub fn new(width: u32, height: u32, channels: u8, colorspace: u8) -> Self {
        Self {
            height,
            width,
            channels,
            colorspace,
        }
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn colorspace(&self) -> u8 {
        self.colorspace
    }
}