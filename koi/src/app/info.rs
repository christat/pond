use std::ffi::CStr;

pub struct Info<'a> {
    pub app_name: &'a CStr,
    pub app_version: u32,
    pub engine_name: &'a CStr,
    pub engine_version: u32,
}

pub fn new(
    app_name: &CStr,
    app_version: u32,
) -> Info {
    Info {
        app_name: app_name,
        app_version: app_version,
        engine_name: c"koi",
        engine_version: make_version(0, 1, 0, 0)
    }
}

pub const fn make_version(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    ((variant) << 29) | ((major) << 22) | ((minor) << 12) | (patch)
}