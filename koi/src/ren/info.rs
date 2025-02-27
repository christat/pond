use crate::app;

pub struct Info {
    pub api_version: u32
}

impl Info {
    pub fn new() -> Self {
        Self {
            api_version: app::info::make_version(1, 3, 0, 0),
        }
    }
}