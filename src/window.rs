#[cfg(feature = "raw_input")]
use crate::raw_input;
use crate::DEFAULT_DPI;
use crate::{
    api::*,
    context::*,
    error::*,
    event::EventHandler,
    geometry::*,
    procedure::{window_proc, UserMessage},
    resource::*,
};
use raw_window_handle::{windows::WindowsHandle, HasRawWindowHandle, RawWindowHandle};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Once, RwLock};
use winapi::shared::{minwindef::*, windef::*};
use winapi::um::{
    libloaderapi::GetModuleHandleW,
    shellapi::*,
    wingdi::{GetStockObject, WHITE_BRUSH},
    winuser::*,
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct WindowHandle(HWND);

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

/// A window style and the borderless window style.
pub trait Style {
    fn value(&self) -> DWORD;

    fn is_borderless(&self) -> bool {
        self.value() == WS_POPUP
    }
}

/// Represents the borderless window style.
pub struct BorderlessStyle;

impl Style for BorderlessStyle {
    fn value(&self) -> DWORD {
        WS_POPUP
    }
}

/// Represents a window style.
pub struct WindowStyle(DWORD);

impl WindowStyle {
    pub fn default() -> Self {
        Self(
            WS_OVERLAPPED
                | WS_CAPTION
                | WS_SYSMENU
                | WS_THICKFRAME
                | WS_MINIMIZEBOX
                | WS_MAXIMIZEBOX,
        )
    }

    pub fn borderless() -> BorderlessStyle {
        BorderlessStyle
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        if resizable {
            self.0 |= WS_THICKFRAME;
        } else {
            self.0 &= !WS_THICKFRAME;
        }
        self
    }

    pub fn has_minimize_box(mut self, has_minimize_box: bool) -> Self {
        if has_minimize_box {
            self.0 |= WS_MINIMIZEBOX;
        } else {
            self.0 &= !WS_MINIMIZEBOX;
        }
        self
    }

    pub fn has_maximize_box(mut self, has_maximize_box: bool) -> Self {
        if has_maximize_box {
            self.0 |= WS_MAXIMIZEBOX;
        } else {
            self.0 &= !WS_MAXIMIZEBOX;
        }
        self
    }

    pub fn is_borderless(&self) -> bool {
        self.value() == WS_POPUP
    }
}

impl Style for WindowStyle {
    fn value(&self) -> DWORD {
        self.0
    }
}

/// The object that allows you to build windows.
pub struct WindowBuilder<Ti = (), S = ()> {
    title: Ti,
    position: ScreenPosition,
    inner_size: S,
    visibility: bool,
    style: DWORD,
    visible_ime_composition_window: bool,
    visible_ime_candidate_window: bool,
    parent: Option<Window>,
    children: Vec<Window>,
    accept_drag_files: bool,
    icon: Option<Icon>,
    #[cfg(feature = "raw_input")]
    raw_input_window_state: raw_input::WindowState,
}

impl WindowBuilder<(), ()> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> WindowBuilder<&'static str, LogicalSize<u32>> {
        WindowBuilder {
            title: "",
            position: ScreenPosition::new(0, 0),
            inner_size: LogicalSize::new(640, 480),
            style: WindowStyle::default().value(),
            visibility: true,
            visible_ime_composition_window: true,
            visible_ime_candidate_window: true,
            parent: None,
            children: Vec::new(),
            accept_drag_files: false,
            icon: None,
            #[cfg(feature = "raw_input")]
            raw_input_window_state: raw_input::WindowState::Foreground,
        }
    }
}

impl<Ti, S> WindowBuilder<Ti, S> {
    pub fn title<T>(self, title: T) -> WindowBuilder<T, S> {
        WindowBuilder {
            title,
            position: self.position,
            inner_size: self.inner_size,
            style: self.style,
            visibility: self.visibility,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            parent: self.parent,
            children: self.children,
            accept_drag_files: self.accept_drag_files,
            icon: self.icon,
            #[cfg(feature = "raw_input")]
            raw_input_window_state: self.raw_input_window_state,
        }
    }

    pub fn position(mut self, position: impl Into<ScreenPosition>) -> Self {
        self.position = position.into();
        self
    }

    pub fn inner_size<T>(self, inner_size: T) -> WindowBuilder<Ti, T> {
        WindowBuilder {
            title: self.title,
            position: self.position,
            inner_size,
            style: self.style,
            visibility: self.visibility,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            parent: self.parent,
            children: self.children,
            accept_drag_files: self.accept_drag_files,
            icon: self.icon,
            #[cfg(feature = "raw_input")]
            raw_input_window_state: self.raw_input_window_state,
        }
    }

    pub fn style(mut self, style: impl Style) -> WindowBuilder<Ti, S> {
        self.style = style.value();
        self
    }

    pub fn visible(mut self, visibility: bool) -> WindowBuilder<Ti, S> {
        self.visibility = visibility;
        self
    }

    pub fn visible_ime_composition_window(
        mut self,
        show_ime_composition_window: bool,
    ) -> WindowBuilder<Ti, S> {
        self.visible_ime_composition_window = show_ime_composition_window;
        self
    }

    pub fn visible_ime_candidate_window(
        mut self,
        show_ime_cadidate_window: bool,
    ) -> WindowBuilder<Ti, S> {
        self.visible_ime_candidate_window = show_ime_cadidate_window;
        self
    }

    pub fn parent(mut self, parent: &Window) -> WindowBuilder<Ti, S> {
        self.parent = Some(parent.clone());
        self
    }

    pub fn child(mut self, child: &Window) -> WindowBuilder<Ti, S> {
        self.children.push(child.clone());
        self
    }

    pub fn children(mut self, children: &[&Window]) -> WindowBuilder<Ti, S> {
        for &c in children {
            self.children.push(c.clone());
        }
        self
    }

    pub fn accept_drag_files(mut self, enabled: bool) -> WindowBuilder<Ti, S> {
        self.accept_drag_files = enabled;
        self
    }

    pub fn icon(mut self, icon: Icon) -> WindowBuilder<Ti, S> {
        self.icon = Some(icon);
        self
    }

    #[cfg(feature = "raw_input")]
    pub fn raw_input_window_state(mut self, state: raw_input::WindowState) -> WindowBuilder<Ti, S> {
        self.raw_input_window_state = state;
        self
    }
}

fn window_class_name() -> &'static Vec<u16> {
    static mut WINDOW_CLASS_NAME: Vec<u16> = Vec::new();
    static REGISTER: Once = Once::new();
    unsafe {
        REGISTER.call_once(|| {
            let class_name = "curun_window_class"
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            WINDOW_CLASS_NAME = class_name;
        });
        &WINDOW_CLASS_NAME
    }
}

pub(crate) fn register_class<T: EventHandler + 'static>() {
    unsafe {
        let class_name = window_class_name();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: CS_VREDRAW | CS_HREDRAW,
            lpfnWndProc: Some(window_proc::<T>),
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
    }
}

impl<Ti, S> WindowBuilder<Ti, S>
where
    Ti: AsRef<str>,
    S: ToPhysicalSize<u32>,
{
    pub fn build(self) -> Result<Window, ApiError> {
        if is_context_null() {
            panic!("The window can be created after run");
        }
        let class_name = window_class_name();
        let title = self
            .title
            .as_ref()
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>();
        unsafe {
            let dpi = get_dpi_from_point(self.position);
            let inner_size = self.inner_size.to_physical(dpi);
            let rc = adjust_window_rect(inner_size, WS_OVERLAPPEDWINDOW, 0, dpi);
            let hinst = GetModuleHandleW(std::ptr::null_mut()) as HINSTANCE;
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                self.style,
                self.position.x,
                self.position.y,
                (rc.right - rc.left) as i32,
                (rc.bottom - rc.top) as i32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinst,
                std::ptr::null_mut(),
            );
            if hwnd == std::ptr::null_mut() {
                return Err(ApiError::new());
            }
            let window = LocalWindow::new(
                hwnd,
                WindowState {
                    title: self.title.as_ref().to_string(),
                    style: self.style,
                    set_position: self.position,
                    set_inner_size: inner_size,
                    visible_ime_composition_window: self.visible_ime_composition_window,
                    visible_ime_candidate_window: self.visible_ime_candidate_window,
                    ime_position: PhysicalPosition::new(0, 0),
                    children: self.children,
                    closed: false,
                },
            );
            let handle = window.handle.clone();
            if let Some(parent) = self.parent {
                let mut state = parent.state.write().unwrap();
                state.children.push(handle.clone());
            }
            if self.visibility {
                window.handle.show();
            }
            if self.accept_drag_files {
                DragAcceptFiles(hwnd, TRUE);
            }
            if let Some(icon) = self.icon {
                let big = load_icon(&icon, hinst);
                let small = load_small_icon(&icon, hinst);
                SendMessageW(
                    handle.raw_handle() as _,
                    WM_SETICON,
                    ICON_BIG as WPARAM,
                    big as LPARAM,
                );
                SendMessageW(
                    handle.raw_handle() as _,
                    WM_SETICON,
                    ICON_SMALL as WPARAM,
                    small as LPARAM,
                );
            }
            #[cfg(feature = "raw_input")]
            raw_input::register_devices(&window.handle, self.raw_input_window_state);
            push_window(hwnd, window);
            Ok(handle)
        }
    }
}

pub(crate) struct WindowState {
    pub title: String,
    pub style: DWORD,
    pub set_position: ScreenPosition,
    pub set_inner_size: PhysicalSize<u32>,
    pub visible_ime_composition_window: bool,
    pub visible_ime_candidate_window: bool,
    pub ime_position: PhysicalPosition<i32>,
    pub children: Vec<Window>,
    pub closed: bool,
}

/// Represents a window.
#[derive(Clone)]
pub(crate) struct LocalWindow {
    pub handle: Window,
    pub ime_context: Rc<RefCell<ImmContext>>,
}

impl LocalWindow {
    pub(crate) fn new(hwnd: HWND, state: WindowState) -> Self {
        Self {
            handle: Window {
                hwnd: WindowHandle(hwnd),
                state: Arc::new(RwLock::new(state)),
            },
            ime_context: Rc::new(RefCell::new(ImmContext::new(hwnd))),
        }
    }
}

/// The object is like Window, can send to other threads.
#[derive(Clone)]
pub struct Window {
    pub(crate) hwnd: WindowHandle,
    pub(crate) state: Arc<RwLock<WindowState>>,
}

impl Window {
    pub fn title(&self) -> String {
        let state = self.state.read().unwrap();
        state.title.clone()
    }

    pub fn set_title(&self, title: impl AsRef<str>) {
        let mut state = self.state.write().unwrap();
        state.title = title.as_ref().to_string();
        unsafe {
            PostMessageW(self.hwnd.0, WM_USER, UserMessage::SetTitle as usize, 0);
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

    pub fn inner_size(&self) -> PhysicalSize<u32> {
        unsafe {
            let mut rc = RECT::default();
            GetClientRect(self.hwnd.0, &mut rc);
            PhysicalSize::new((rc.right - rc.left) as u32, (rc.bottom - rc.top) as u32)
        }
    }

    pub fn set_inner_size(&self, size: impl ToPhysicalSize<u32>) {
        unsafe {
            let mut state = self.state.write().unwrap();
            state.set_inner_size = size.to_physical(self.dpi());
            PostMessageW(self.hwnd.0, WM_USER, UserMessage::SetInnerSize as usize, 0);
        }
    }

    pub fn dpi(&self) -> u32 {
        unsafe { GetDpiForWindow(self.hwnd.0) }
    }

    pub fn scale_factor(&self) -> f32 {
        unsafe { GetDpiForWindow(self.hwnd.0) as f32 / DEFAULT_DPI as f32 }
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
            RedrawWindow(
                self.hwnd.0,
                std::ptr::null(),
                std::ptr::null_mut(),
                RDW_INTERNALPAINT,
            );
        }
    }

    pub fn is_closed(&self) -> bool {
        let state = self.state.read().unwrap();
        state.closed
    }

    pub fn close(&self) {
        unsafe {
            if !self.is_closed() {
                PostMessageW(self.hwnd.0, WM_CLOSE, 0, 0);
            }
        }
    }

    pub fn ime_position(&self) -> PhysicalPosition<i32> {
        let state = self.state.read().unwrap();
        PhysicalPosition {
            x: state.ime_position.x,
            y: state.ime_position.y,
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

    pub fn set_ime_position(&self, position: impl ToPhysicalPosition<i32>) {
        let mut state = self.state.write().unwrap();
        let position = position.to_physical(self.dpi() as i32);
        state.ime_position.x = position.x;
        state.ime_position.y = position.y;
    }

    pub fn style(&self) -> WindowStyle {
        let state = self.state.read().unwrap();
        WindowStyle(state.style as DWORD)
    }

    pub fn set_style(&self, style: impl Style) {
        unsafe {
            let mut state = self.state.write().unwrap();
            state.style = style.value();
            PostMessageW(self.hwnd.0, WM_USER, UserMessage::SetStyle as usize, 0);
        }
    }

    pub fn accept_drag_files(&self, enabled: bool) {
        unsafe {
            PostMessageW(
                self.hwnd.0,
                WM_USER,
                UserMessage::AcceptDragFiles as usize,
                (if enabled { TRUE } else { FALSE }) as LPARAM,
            );
        }
    }

    pub fn raw_handle(&self) -> *mut std::ffi::c_void {
        self.hwnd.0 as _
    }
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        self.hwnd == other.hwnd
    }
}

impl Eq for Window {}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Windows(WindowsHandle {
            hinstance: unsafe { GetWindowLongPtrW(self.hwnd.0, GWLP_HINSTANCE) as _ },
            hwnd: self.hwnd.0 as _,
            ..WindowsHandle::empty()
        })
    }
}
