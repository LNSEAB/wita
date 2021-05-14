use crate::bindings::Windows::Win32::{
    Graphics::Gdi::*, System::SystemServices::*, UI::DisplayDevices::*, UI::HiDpi::*,
    UI::MenusAndResources::*, UI::Shell::*, UI::WindowsAndMessaging::*,
};
#[cfg(feature = "raw_input")]
use crate::raw_input;
use crate::DEFAULT_DPI;
use crate::{
    api::*,
    context::*,
    error::*,
    event::EventHandler,
    geometry::*,
    ime,
    procedure::{window_proc, UserMessage},
    resource::*,
};
use raw_window_handle::{windows::WindowsHandle, HasRawWindowHandle, RawWindowHandle};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct WindowHandle(HWND);

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

/// A window style and the borderless window style.
pub trait Style {
    fn value(&self) -> u32;

    fn is_borderless(&self) -> bool {
        self.value() == WS_POPUP.0
    }
}

/// Represents the borderless window style.
pub struct BorderlessStyle;

impl Style for BorderlessStyle {
    fn value(&self) -> u32 {
        WS_POPUP.0
    }
}

/// Represents a window style.
pub struct WindowStyle(u32);

impl WindowStyle {
    #[inline]
    pub fn dialog() -> Self {
        Self(
            WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0
        )
    }

    #[inline]
    pub fn borderless() -> BorderlessStyle {
        BorderlessStyle
    }

    #[inline]
    pub fn resizable(mut self, resizable: bool) -> Self {
        if resizable {
            self.0 |= WS_THICKFRAME.0;
        } else {
            self.0 &= !WS_THICKFRAME.0;
        }
        self
    }

    #[inline]
    pub fn has_minimize_box(mut self, has_minimize_box: bool) -> Self {
        if has_minimize_box {
            self.0 |= WS_MINIMIZEBOX.0;
        } else {
            self.0 &= !WS_MINIMIZEBOX.0;
        }
        self
    }

    #[inline]
    pub fn has_maximize_box(mut self, has_maximize_box: bool) -> Self {
        if has_maximize_box {
            self.0 |= WS_MAXIMIZEBOX.0;
        } else {
            self.0 &= !WS_MAXIMIZEBOX.0;
        }
        self
    }

    #[inline]
    pub fn is_borderless(&self) -> bool {
        self.value() == WS_POPUP.0
    }
}

impl Default for WindowStyle {
    #[inline]
    fn default() -> Self {
        Self(
            WS_OVERLAPPED.0
                | WS_CAPTION.0
                | WS_SYSMENU.0
                | WS_THICKFRAME.0
                | WS_MINIMIZEBOX.0
                | WS_MAXIMIZEBOX.0,
        )
    }
}

impl Style for WindowStyle {
    fn value(&self) -> u32 {
        self.0
    }
}

const WINDOW_CLASS_NAME: &str = "wita_window_class";

pub(crate) fn register_class<T: EventHandler + 'static>() {
    unsafe {
        let class_name = WINDOW_CLASS_NAME
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as _,
            style: WNDCLASS_STYLES(CS_VREDRAW.0 | CS_HREDRAW.0),
            lpfnWndProc: Some(window_proc::<T>),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(PWSTR::NULL),
            hIcon: HICON::NULL,
            hCursor: LoadCursorW(HINSTANCE::NULL, IDC_ARROW),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: PWSTR::NULL,
            lpszClassName: PWSTR(class_name.as_ptr() as _),
            hIconSm: HICON::NULL,
        };
        if RegisterClassExW(&wc) == 0 {
            panic!("cannot register the window class");
        }
    }
}

/// The object to build a window.
pub struct WindowBuilder<Ti = (), S = ()> {
    title: Ti,
    position: ScreenPosition,
    inner_size: S,
    visibility: bool,
    style: u32,
    enabled_ime: bool,
    visible_ime_composition_window: bool,
    visible_ime_candidate_window: bool,
    parent: Option<Window>,
    children: Vec<Window>,
    accept_drag_files: bool,
    icon: Option<Icon>,
    no_redirection_bitmap: bool,
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
            enabled_ime: false,
            visible_ime_composition_window: true,
            visible_ime_candidate_window: true,
            parent: None,
            children: Vec::new(),
            accept_drag_files: false,
            icon: None,
            no_redirection_bitmap: false,
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
            enabled_ime: self.enabled_ime,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            parent: self.parent,
            children: self.children,
            accept_drag_files: self.accept_drag_files,
            icon: self.icon,
            no_redirection_bitmap: self.no_redirection_bitmap,
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
            enabled_ime: self.enabled_ime,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            parent: self.parent,
            children: self.children,
            accept_drag_files: self.accept_drag_files,
            icon: self.icon,
            no_redirection_bitmap: self.no_redirection_bitmap,
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

    pub fn ime(mut self, enable: bool) -> WindowBuilder<Ti, S> {
        self.enabled_ime = enable;
        self
    }

    pub fn no_redirection_bitmap(mut self, enable: bool) -> WindowBuilder<Ti, S> {
        self.no_redirection_bitmap = enable;
        self
    }

    #[cfg(feature = "raw_input")]
    pub fn raw_input_window_state(mut self, state: raw_input::WindowState) -> WindowBuilder<Ti, S> {
        self.raw_input_window_state = state;
        self
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
        unsafe {
            let dpi = get_dpi_from_point(self.position);
            let inner_size = self.inner_size.to_physical(dpi);
            let rc = adjust_window_rect(inner_size, self.style, 0, dpi);
            let hinst = GetModuleHandleW(PWSTR::NULL);
            let hwnd = CreateWindowExW(
                if self.no_redirection_bitmap {
                    WS_EX_NOREDIRECTIONBITMAP
                } else {
                    WINDOW_EX_STYLE(0)
                },
                WINDOW_CLASS_NAME,
                self.title.as_ref(),
                WINDOW_STYLE(self.style),
                self.position.x,
                self.position.y,
                (rc.right - rc.left) as i32,
                (rc.bottom - rc.top) as i32,
                HWND::NULL,
                HMENU::NULL,
                hinst,
                std::ptr::null_mut(),
            );
            if hwnd == HWND::NULL {
                return Err(ApiError::new());
            }
            let window = LocalWindow::new(
                hwnd,
                WindowState {
                    title: self.title.as_ref().to_string(),
                    style: self.style,
                    set_position: (self.position.x, self.position.y),
                    set_inner_size: inner_size,
                    enabled_ime: self.enabled_ime,
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
                    HWND(handle.raw_handle() as _),
                    WM_SETICON,
                    WPARAM(ICON_BIG as _),
                    LPARAM(big.0 as _),
                );
                SendMessageW(
                    HWND(handle.raw_handle() as _),
                    WM_SETICON,
                    WPARAM(ICON_SMALL as _),
                    LPARAM(small.0 as _),
                );
            }
            if self.enabled_ime {
                window.handle.ime(self.enabled_ime);
            }
            #[cfg(feature = "raw_input")]
            raw_input::register_devices(&window.handle, self.raw_input_window_state);
            push_window(hwnd, window);
            Ok(handle)
        }
    }
}

/// The object to build a window into the parent window.
pub struct InnerWindowBuilder<W = (), P = (), S = ()> {
    parent: W,
    position: P,
    size: S,
    visibility: bool,
    visible_ime_composition_window: bool,
    visible_ime_candidate_window: bool,
    accept_drag_files: bool,
    #[cfg(feature = "raw_input")]
    raw_input_window_state: raw_input::WindowState,
}

impl InnerWindowBuilder<(), (), ()> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> InnerWindowBuilder<(), LogicalPosition<f32>, ()> {
        InnerWindowBuilder {
            parent: (),
            position: LogicalPosition::new(0.0, 0.0),
            size: (),
            visibility: true,
            visible_ime_composition_window: true,
            visible_ime_candidate_window: true,
            accept_drag_files: false,
            #[cfg(feature = "raw_input")]
            raw_input_window_state: raw_input::WindowState::Foreground,
        }
    }
}

impl<W, P, S> InnerWindowBuilder<W, P, S> {
    pub fn parent(self, parent: &Window) -> InnerWindowBuilder<Window, P, S> {
        InnerWindowBuilder {
            parent: parent.clone(),
            position: self.position,
            size: self.size,
            visibility: self.visibility,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            accept_drag_files: self.accept_drag_files,
            #[cfg(feature = "raw_input")]
            raw_input_window_state: self.raw_input_window_state,
        }
    }

    pub fn position<T>(self, position: T) -> InnerWindowBuilder<W, T, S> {
        InnerWindowBuilder {
            parent: self.parent,
            position,
            size: self.size,
            visibility: self.visibility,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            accept_drag_files: self.accept_drag_files,
            #[cfg(feature = "raw_input")]
            raw_input_window_state: self.raw_input_window_state,
        }
    }

    pub fn size<T>(self, size: T) -> InnerWindowBuilder<W, P, T> {
        InnerWindowBuilder {
            parent: self.parent,
            position: self.position,
            size,
            visibility: self.visibility,
            visible_ime_composition_window: self.visible_ime_composition_window,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            accept_drag_files: self.accept_drag_files,
            #[cfg(feature = "raw_input")]
            raw_input_window_state: self.raw_input_window_state,
        }
    }

    pub fn visible(mut self, visibility: bool) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn accept_drag_files(mut self) -> Self {
        self.accept_drag_files = true;
        self
    }
}

impl<P, S> InnerWindowBuilder<Window, P, S>
where
    P: ToPhysicalPosition<i32>,
    S: ToPhysicalSize<u32>,
{
    pub fn build(self) -> Result<Window, ApiError> {
        unsafe {
            let dpi = self.parent.dpi();
            let position = self.position.to_physical(dpi as i32);
            let size = self.size.to_physical(dpi);
            let rc = adjust_window_rect(size, WS_CHILD.0, 0, dpi);
            let hinst = GetModuleHandleW(PWSTR::NULL);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                WINDOW_CLASS_NAME,
                PWSTR::NULL,
                WS_CHILD,
                position.x,
                position.y,
                (rc.right - rc.left) as i32,
                (rc.bottom - rc.top) as i32,
                HWND(self.parent.raw_handle() as _),
                HMENU::NULL,
                hinst,
                std::ptr::null_mut(),
            );
            if hwnd == HWND::NULL {
                return Err(ApiError::new());
            }
            let window = LocalWindow::new(
                hwnd,
                WindowState {
                    title: String::new(),
                    style: WS_CHILD.0,
                    set_position: (position.x, position.y),
                    set_inner_size: size,
                    enabled_ime: self.parent.is_enabled_ime(),
                    visible_ime_composition_window: self.visible_ime_composition_window,
                    visible_ime_candidate_window: self.visible_ime_candidate_window,
                    ime_position: PhysicalPosition::new(0, 0),
                    children: vec![],
                    closed: false,
                },
            );
            let handle = window.handle.clone();
            if self.visibility {
                window.handle.show();
            }
            if self.accept_drag_files {
                DragAcceptFiles(hwnd, TRUE);
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
    pub style: u32,
    pub set_position: (i32, i32),
    pub set_inner_size: PhysicalSize<u32>,
    pub enabled_ime: bool,
    pub visible_ime_composition_window: bool,
    pub visible_ime_candidate_window: bool,
    pub ime_position: PhysicalPosition<i32>,
    pub children: Vec<Window>,
    pub closed: bool,
}

#[derive(Clone)]
pub(crate) struct LocalWindow {
    pub handle: Window,
    pub ime_context: Rc<RefCell<ime::ImmContext>>,
}

impl LocalWindow {
    pub(crate) fn new(hwnd: HWND, state: WindowState) -> Self {
        Self {
            handle: Window {
                hwnd: WindowHandle(hwnd),
                state: Arc::new(RwLock::new(state)),
            },
            ime_context: Rc::new(RefCell::new(ime::ImmContext::new(hwnd))),
        }
    }
}

/// Represents a window.
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
            PostMessageW(
                self.hwnd.0,
                WM_USER,
                WPARAM(UserMessage::SetTitle as _),
                LPARAM(0),
            );
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
            state.set_position = (position.x, position.y);
            PostMessageW(
                self.hwnd.0,
                WM_USER,
                WPARAM(UserMessage::SetPosition as _),
                LPARAM(0),
            );
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
            PostMessageW(
                self.hwnd.0,
                WM_USER,
                WPARAM(UserMessage::SetInnerSize as _),
                LPARAM(0),
            );
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
            ShowWindowAsync(self.hwnd.0, SW_SHOW.0 as _);
        }
    }

    pub fn hide(&self) {
        unsafe {
            ShowWindowAsync(self.hwnd.0, SW_HIDE.0 as _);
        }
    }

    pub fn redraw(&self) {
        unsafe {
            RedrawWindow(self.hwnd.0, std::ptr::null(), HRGN::NULL, RDW_INTERNALPAINT);
        }
    }

    pub fn is_closed(&self) -> bool {
        let state = self.state.read().unwrap();
        state.closed
    }

    pub fn close(&self) {
        unsafe {
            if !self.is_closed() {
                PostMessageW(self.hwnd.0, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }

    pub fn ime_position(&self) -> PhysicalPosition<i32> {
        let state = self.state.read().unwrap();
        PhysicalPosition::new(state.ime_position.x, state.ime_position.y)
    }

    pub fn ime(&self, enable: bool) {
        unsafe {
            if enable {
                PostMessageW(
                    self.hwnd.0,
                    WM_USER,
                    WPARAM(UserMessage::EnableIme as _),
                    LPARAM(0),
                );
            } else {
                PostMessageW(
                    self.hwnd.0,
                    WM_USER,
                    WPARAM(UserMessage::DisableIme as _),
                    LPARAM(0),
                );
            }
        }
        let mut state = self.state.write().unwrap();
        state.enabled_ime = enable;
    }

    pub fn set_ime_position(&self, position: impl ToPhysicalPosition<i32>) {
        let mut state = self.state.write().unwrap();
        let position = position.to_physical(self.dpi() as i32);
        state.ime_position.x = position.x;
        state.ime_position.y = position.y;
    }

    pub fn is_enabled_ime(&self) -> bool {
        let state = self.state.read().unwrap();
        state.enabled_ime
    }

    pub fn style(&self) -> WindowStyle {
        let state = self.state.read().unwrap();
        WindowStyle(state.style)
    }

    pub fn set_style(&self, style: impl Style) {
        unsafe {
            let mut state = self.state.write().unwrap();
            state.style = style.value();
            PostMessageW(
                self.hwnd.0,
                WM_USER,
                WPARAM(UserMessage::SetStyle as _),
                LPARAM(0),
            );
        }
    }

    pub fn accept_drag_files(&self, enabled: bool) {
        unsafe {
            PostMessageW(
                self.hwnd.0,
                WM_USER,
                WPARAM(UserMessage::AcceptDragFiles as _),
                LPARAM(if enabled { 1 } else { 0 }),
            );
        }
    }

    pub fn raw_handle(&self) -> *mut std::ffi::c_void {
        self.hwnd.0 .0 as _
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
            hinstance: unsafe { GetModuleHandleW(PWSTR::NULL).0 as _ },
            hwnd: self.hwnd.0 .0 as _,
            ..WindowsHandle::empty()
        })
    }
}
