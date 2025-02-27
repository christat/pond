use winit::{raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle}, window::Window as WindowHandle};

#[derive(Debug)]
#[allow(unused)]
pub struct Window {
    #[cfg(target_os = "windows")]
    pub display: WindowsDisplayHandle,
    #[cfg(target_os = "windows")]
    pub window: Win32WindowHandle,

    #[cfg(target_os = "linux")]
    pub display: XcbDisplayHandle,
    #[cfg(target_os = "linux")]
    pub window: XcbWindowHandle,
}

#[derive(Debug)]
pub enum WindowError {
    DisplayHandleError,
    WindowHandleError,
}

impl Window {
    pub fn new(window: &WindowHandle) -> Result<Window, WindowError> {
        let display_handle = window.display_handle().expect("koi::ren::WindowHandle - failed to get display handle");
        let display = match display_handle.as_raw() {
            #[cfg(target_os = "windows")]
            RawDisplayHandle::Windows(handle) => Ok(handle),
            #[cfg(target_os = "linux")]
            RawDisplayHandle::Xcb(handle) => Ok(handle),
            _ => Err(WindowError::DisplayHandleError),
        }?;

        let window_handle = window.window_handle().expect("koi::ren::WindowHandle - failed to get window handle");
        let window = match window_handle.as_raw() {
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(handle) => Ok(handle),
            #[cfg(target_os = "linux")]
            RawWindowHandle::Xcb(handle) => Ok(handle),
            _ => Err(WindowError::WindowHandleError)
        }?;
        
        Ok(Self {
            display: display,
            window: window,
        })
    }
}