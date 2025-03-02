pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        Self{ width: 1920, height: 1080 }
    }
}

#[allow(unused)]
impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self{ width, height }
    }
}

#[derive(Default)]
pub struct Settings {
    pub resolution: Resolution,
    pub buffering: u32,
}

#[allow(unused)]
impl Settings {
    pub fn resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn buffering(mut self, buffering: u32) -> Self {
        self.buffering = buffering;
        self
    }
}

