use crate::{
    api::*,
    context::*,
    geometry::*,
    procedure::{window_proc, UserMessage},
};
use std::sync::{Arc, Once, RwLock};
use winapi::shared::{minwindef::*, windef::*};
use winapi::um::{
    libloaderapi::GetModuleHandleW,
    wingdi::{GetStockObject, WHITE_BRUSH},
    winuser::*,
};

#[derive(Clone)]
pub(crate) struct WindowHandle(HWND);

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

/// The object that allows you to build windows.
pub struct WindowBuilder<Ti = (), S = ()> {
    title: Ti,
    position: ScreenPosition,
    inner_size: S,
    resizable: bool,
    visibility: bool,
    has_minimize_box: bool,
    has_maximize_box: bool,
    visible_ime_composition_window: bool,
    visible_ime_candidate_window: bool,
}

impl WindowBuilder<(), ()> {
    pub fn new() -> WindowBuilder<&'static str, LogicalSize<f32>> {
        WindowBuilder {
            title: "",
            position: ScreenPosition::new(0, 0),
            inner_size: LogicalSize::new(640.0, 480.0),
            resizable: true,
            visibility: true,
            has_minimize_box: true,
            has_maximize_box: true,
            visible_ime_composition_window: true,
            visible_ime_candidate_window: true,
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
            has_minimize_box: self.has_minimize_box,
            has_maximize_box: self.has_maximize_box,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
        }
    }

    pub fn position(mut self, position: ScreenPosition) -> Self {
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
            has_minimize_box: self.has_minimize_box,
            has_maximize_box: self.has_maximize_box,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
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

    pub fn has_minimize_box(mut self, has_minimize_box: bool) -> WindowBuilder<Ti, S> {
        self.has_minimize_box = has_minimize_box;
        self
    }

    pub fn has_maximize_box(mut self, has_maximize_box: bool) -> WindowBuilder<Ti, S> {
        self.has_maximize_box = has_maximize_box;
        self
    }

    pub fn visible_ime_composition_window(mut self, show_ime_composition_window: bool) -> WindowBuilder<Ti, S> {
        self.visible_ime_composition_window = show_ime_composition_window;
        self
    }

    pub fn visible_ime_candidate_window(mut self, show_ime_cadidate_window: bool) -> WindowBuilder<Ti, S> {
        self.visible_ime_candidate_window = show_ime_cadidate_window;
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
            if RegisterClassExW(&wc) == 0 {
                panic!("cannot register the window class");
            }
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
        let mut style = WS_OVERLAPPED | WS_SYSMENU | WS_CAPTION;
        if self.resizable {
            style |= WS_THICKFRAME;
        }
        if self.has_minimize_box {
            style |= WS_MINIMIZEBOX;
        }
        if self.has_maximize_box {
            style |= WS_MAXIMIZEBOX;
        }
        unsafe {
            let dpi = get_dpi_from_point(self.position.clone());
            let inner_size = self.inner_size.to_physical(dpi as f32 / DEFAULT_DPI);
            let rc = adjust_window_rect(inner_size, WS_OVERLAPPEDWINDOW, 0, dpi);
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
            if hwnd == std::ptr::null_mut() {
                panic!("cannot create the window");
            }
            let window = Window::new(
                hwnd,
                WindowState {
                    set_position: self.position,
                    set_inner_size: inner_size,
                    visible_ime_composition_window: self.visible_ime_composition_window,
                    visible_ime_candidate_window: self.visible_ime_candidate_window,
                    ime_position: PhysicalPosition::new(0, 0),
                    ime_context: ImmContext::new(hwnd),
                    closed: false,
                },
            );
            if self.visibility {
                window.show();
            }
            context.window_table().push((hwnd, window.clone()));
            window
        }
    }
}

pub(crate) struct WindowState {
    pub set_position: ScreenPosition,
    pub set_inner_size: PhysicalSize<f32>,
    pub visible_ime_composition_window: bool,
    pub visible_ime_candidate_window: bool,
    pub ime_position: PhysicalPosition<i32>,
    pub ime_context: ImmContext,
    pub closed: bool,
}

/// Represents a window.
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

    pub fn position(&self) -> ScreenPosition {
        unsafe {
            let mut rc = RECT::default();
            GetWindowRect(self.hwnd.0, &mut rc);
            ScreenPosition::new(rc.left, rc.top)
        }
    }

    pub fn set_position(&self, position: ScreenPosition) {
        unsafe {
            let mut state = self.state.write().unwrap();
            state.set_position = position;
            PostMessageW(self.hwnd.0, WM_USER, UserMessage::SetPosition as usize, 0);
        }
    }

    pub fn inner_size(&self) -> PhysicalSize<f32> {
        unsafe {
            let mut rc = RECT::default();
            GetClientRect(self.hwnd.0, &mut rc);
            PhysicalSize::new((rc.right - rc.left) as f32, (rc.bottom - rc.top) as f32)
        }
    }

    pub fn set_inner_size(&self, size: impl ToPhysicalSize<f32>) {
        unsafe {
            let mut state = self.state.write().unwrap();
            state.set_inner_size = size.to_physical(self.scale_factor());
            PostMessageW(self.hwnd.0, WM_USER, UserMessage::SetInnerSize as usize, 0);
        }
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

    pub fn redraw(&self) {
        unsafe {
            InvalidateRect(self.hwnd.0, std::ptr::null_mut(), FALSE);
        }
    }

    pub fn is_closed(&self) -> bool {
        let state = self.state.read().unwrap();
        state.closed
    }

    pub fn close(&self) {
        unsafe {
            PostMessageW(self.hwnd.0, WM_CLOSE, 0, 0);
        }
    }

    pub fn raw_handle(&self) -> *const std::ffi::c_void {
        self.hwnd.0 as _
    }

    pub fn ime_position(&self) -> PhysicalPosition<f32> {
        let state = self.state.read().unwrap();
        PhysicalPosition {
            x: state.ime_position.x as f32,
            y: state.ime_position.y as f32,
        }
    }

    pub fn enable_ime(&self) {
        unsafe {
            PostMessageW(self.hwnd.0, WM_USER, UserMessage::EnableIme as usize, 0);
        }
    }

    pub fn disable_ime(&self) {
        unsafe {
            PostMessageW(self.hwnd.0, WM_USER, UserMessage::DisableIme as usize, 0);
        }
    }

    pub fn set_ime_position(&self, position: impl ToPhysicalPosition<f32>) {
        let mut state = self.state.write().unwrap();
        let position = position.to_physical(self.scale_factor());
        state.ime_position.x = position.x as i32;
        state.ime_position.y = position.y as i32;
    }
}
