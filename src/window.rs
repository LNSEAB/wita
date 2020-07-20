use crate::{
    api::*,
    geometry::*,
    procedure::window_proc,
    context::*,
};
use winapi::shared::{
    minwindef::*,
    windef::*,
};
use winapi::um::{
    libloaderapi::GetModuleHandleW,
    wingdi::{GetStockObject, WHITE_BRUSH},
    winuser::*,
};
use std::sync::{Arc, RwLock, Once};

#[derive(Clone)]
pub(crate) struct WindowHandle(HWND);

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

pub struct WindowBuilder<Ti = (), S = ()> {
    title: Ti,
    position: PhysicalPosition<i32>,
    inner_size: S,
    resizable: bool,
    visibility: bool,
    visible_composition_window: bool,
    visible_candidate_window: bool,
}

impl WindowBuilder<(), ()> {
    pub fn new() -> WindowBuilder<&'static str, LogicalSize<f32>> {
        WindowBuilder {
            title: "",
            position: PhysicalPosition::new(0, 0),
            inner_size: LogicalSize::new(640.0, 480.0),
            resizable: true,
            visibility: true,
            visible_composition_window: true,
            visible_candidate_window: true,
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
            visible_composition_window: self.visible_composition_window,
            visible_candidate_window: self.visible_candidate_window,
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
            visible_composition_window: self.visible_composition_window,
            visible_candidate_window: self.visible_candidate_window,
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

    pub fn visible_composition_window(mut self, show_composition_window: bool) -> WindowBuilder<Ti, S> {
        self.visible_composition_window = show_composition_window;
        self
    }

    pub fn visible_candidate_window(mut self, show_cadidate_window: bool) -> WindowBuilder<Ti, S> {
        self.visible_candidate_window = show_cadidate_window;
        self
    }
}

fn register_class() -> &'static Vec<u16> {
    static mut WINDOW_CLASS_NAME: Vec<u16> = Vec::new();
    static REGISTER: Once = Once::new();
    unsafe {
        REGISTER.call_once(|| {
            let class_name = "curun_window_class".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
                style: CS_VREDRAW | CS_HREDRAW,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleW(std::ptr::null_mut()),
                hIcon: std::ptr::null_mut(),
                hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
                hbrBackground: GetStockObject(WHITE_BRUSH as i32) as HBRUSH,
                lpszMenuName: std::ptr::null_mut(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: std::ptr::null_mut(),
            };
            RegisterClassExW(&wc);
            WINDOW_CLASS_NAME = class_name;
        });
        &WINDOW_CLASS_NAME
    }
}

impl<Ti, S> WindowBuilder<Ti, S>
where
    Ti: AsRef<str>,
    S: ToPhysicalSize<f32>,
{
    pub fn build(self, context: &Context) -> Window {
        let class_name = register_class();
        let title = self.title.as_ref().encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let mut style = WS_OVERLAPPED | WS_SYSMENU | WS_CAPTION | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
        if self.resizable {
            style |= WS_THICKFRAME;
        }
        unsafe {
            let dpi = get_dpi_from_point(self.position.clone());
            let rc = adjust_window_rect(
                self.inner_size.to_physical(dpi as f32 / DEFAULT_DPI),
                WS_OVERLAPPEDWINDOW,
                0,
                dpi,
            );
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                style,
                self.position.x,
                self.position.y,
                (rc.right - rc.left) as i32,
                (rc.bottom - rc.top) as i32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null_mut()),
                std::ptr::null_mut(),
            );
            let window = Window::new(hwnd, WindowState {
                visible_composition_window: self.visible_composition_window,
                visible_candidate_window: self.visible_candidate_window,
                ime_position: PhysicalPosition::new(0, 0),
                ime_context: ImmContext::new(hwnd),
            });
            if self.visibility {
                window.show();
            }
            context.window_table().push((hwnd, window.clone()));
            window
        }
    }
}

pub(crate) struct WindowState {
    pub visible_composition_window: bool,
    pub visible_candidate_window: bool,
    pub ime_position: PhysicalPosition<i32>,
    pub ime_context: ImmContext,
}

#[derive(Clone)]
pub struct Window {
    hwnd: WindowHandle,
    pub(crate) state: Arc<RwLock<WindowState>>,
}

impl Window {
    pub(crate) fn new(hwnd: HWND, state: WindowState) -> Self {
        Self {
            hwnd: WindowHandle(hwnd),
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn inner_size(&self) -> PhysicalSize<f32> {
        unsafe {
            let mut rc = RECT::default();
            GetClientRect(self.hwnd.0, &mut rc);
            PhysicalSize::new((rc.right - rc.left) as f32, (rc.bottom - rc.top) as f32)
        }
    }

    pub fn ime_position(&self) -> PhysicalPosition<f32> {
        let state = self.state.read().unwrap();
        PhysicalPosition {
            x: state.ime_position.x as f32,
            y: state.ime_position.y as f32,
        }
    }

    pub fn set_ime_position(&self, position: impl ToPhysicalPosition<f32>) {
        let mut state = self.state.write().unwrap();
        let position = position.to_physical(self.scale_factor());
        state.ime_position.x = position.x as i32;
        state.ime_position.y = position.y as i32;
    }

    pub fn visible_composition_window(&self) -> bool {
        let state = self.state.read().unwrap();
        state.visible_composition_window
    }

    pub fn set_visible_composition_window(&self, visibility: bool) {
        let mut state = self.state.write().unwrap();
        state.visible_composition_window = visibility;
    }

    pub fn visible_candidate_window(&self) -> bool {
        let state = self.state.read().unwrap();
        state.visible_candidate_window
    }

    pub fn set_visible_candidate_window(&self, visibility: bool) {
        let mut state = self.state.write().unwrap();
        state.visible_candidate_window = visibility;
    }

    pub fn scale_factor(&self) -> f32 {
        unsafe { GetDpiForWindow(self.hwnd.0) as f32 / DEFAULT_DPI }
    }

    pub fn show(&self) {
        unsafe {
            ShowWindowAsync(self.hwnd.0, SW_SHOW);
        }
    }

    pub fn hide(&self) {
        unsafe {
            ShowWindowAsync(self.hwnd.0, SW_HIDE);
        }
    }

    pub fn raw_handle(&self) -> *const std::ffi::c_void {
        self.hwnd.0 as _
    }
}
