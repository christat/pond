use std::ffi::CString;

pub struct Info {
    pub app_name: CString,
    pub app_version: u32,
    pub engine_name: CString,
    pub engine_version: u32,
}

pub fn new(
    app_name: String,
    app_version: u32,
) -> Info {
    Info {
        app_name: CString::new(app_name).unwrap(),
        app_version: app_version,
        engine_name: CString::new("koi").unwrap(),
        engine_version: make_version(0, 1, 0, 0)
    }
}

pub const fn make_version(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    ((variant) << 29) | ((major) << 22) | ((minor) << 12) | (patch)
}