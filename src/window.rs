use crate::context::{Context, DEFAULT_DPI};
use crate::geometry::*;
use winapi::shared::windef::*;
use winapi::um::winuser::*;

pub struct WindowBuilder<Ti = (), S = ()> {
    pub(crate) title: Ti,
    pub(crate) position: PhysicalPosition<i32>,
    pub(crate) inner_size: S,
    pub(crate) resizable: bool,
    pub(crate) visibility: bool,
}

impl WindowBuilder<(), ()> {
    pub fn new() -> WindowBuilder<&'static str, LogicalSize<f32>> {
        WindowBuilder {
            title: "",
            position: PhysicalPosition::new(0, 0),
            inner_size: LogicalSize::new(640.0, 480.0),
            resizable: true,
            visibility: true,
        }
    }
}

impl<Ti, S> WindowBuilder<Ti, S> {
    pub fn title<T>(self, title: T) -> WindowBuilder<T, S> {
        WindowBuilder {
            title,
            position: self.position,
            inner_size: self.inner_size,
            resizable: self.resizable,
            visibility: self.visibility,
        }
    }

    pub fn position(mut self, position: PhysicalPosition<i32>) -> Self {
        self.position = position;
        self
    }

    pub fn inner_size<T>(self, inner_size: T) -> WindowBuilder<Ti, T> {
        WindowBuilder {
            title: self.title,
            position: self.position,
            inner_size,
            resizable: self.resizable,
            visibility: self.visibility,
        }
    }

    pub fn resizable(mut self, resizable: bool) -> WindowBuilder<Ti, S> {
        self.resizable = resizable;
        self
    }

    pub fn visible(mut self, visibility: bool) -> WindowBuilder<Ti, S> {
        self.visibility = visibility;
        self
    }
}

impl<Ti, S> WindowBuilder<Ti, S>
where
    Ti: AsRef<str>,
    S: ToPhysicalSize<f32>,
{
    pub fn build(self, context: &Context) -> Window {
        context.create_window(self)
    }
}

#[derive(Clone)]
pub struct Window(pub(crate) HWND);

impl Window {
    pub fn inner_size(&self) -> PhysicalSize<f32> {
        unsafe {
            let mut rc = RECT::default();
            GetClientRect(self.0, &mut rc);
            PhysicalSize::new((rc.right - rc.left) as f32, (rc.bottom - rc.top) as f32)
        }
    }

    pub fn show(&self) {
        unsafe {
            ShowWindowAsync(self.0, SW_SHOW);
        }
    }

    pub fn hide(&self) {
        unsafe {
            ShowWindowAsync(self.0, SW_HIDE);
        }
    }

    pub fn scale_factor(&self) -> f32 {
        unsafe { GetDpiForWindow(self.0) as f32 / DEFAULT_DPI }
    }

    pub fn raw_handle(&self) -> *const std::ffi::c_void {
        self.0 as _
    }
}
