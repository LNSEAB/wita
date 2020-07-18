use crate::{
    api::*,
    device::*,
    event::EventHandler,
    geometry::*,
    procedure::window_proc,
    window::{Window, WindowBuilder},
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Once;
use winapi::shared::{minwindef::*, windef::*, winerror::S_OK};
use winapi::um::{
    libloaderapi::GetModuleHandleW,
    shellscalingapi::*,
    wingdi::{GetStockObject, WHITE_BRUSH},
    winuser::*,
};

pub(crate) const DEFAULT_DPI: f32 = 96.0;

thread_local! {
    static CONTEXT: RefCell<Option<Rc<ContextImpl>>> = RefCell::new(None);
}

pub enum RunType {
    Idle,
    Wait,
}

pub(crate) struct ContextState {
    pub mouse_buttons: Vec<MouseButton>,
}

impl ContextState {
    fn new() -> Self {
        Self {
            mouse_buttons: Vec::with_capacity(5),
        }
    }
}

pub(crate) struct ContextImpl {
    event_handler: RefCell<Option<Box<dyn EventHandler>>>,
    window_table: RefCell<Vec<(HWND, Window)>>,
    state: RefCell<ContextState>,
}

impl ContextImpl {
    fn new() -> Self {
        Self {
            event_handler: RefCell::new(None),
            window_table: RefCell::new(Vec::new()),
            state: RefCell::new(ContextState::new()),
        }
    }

    fn set_event_handler(&self, event_handler: impl EventHandler + 'static) {
        *self.event_handler.borrow_mut() = Some(Box::new(event_handler));
    }

    fn call_handler(&self, f: impl FnOnce(&mut Box<dyn EventHandler>, &mut ContextState)) {
        f(
            self.event_handler.borrow_mut().as_mut().unwrap(),
            &mut *self.state.borrow_mut(),
        )
    }
}

fn register_class() -> &'static Vec<u16> {
    static mut WINDOW_CLASS_NAME: Vec<u16> = Vec::new();
    static REGISTER: Once = Once::new();
    unsafe {
        REGISTER.call_once(|| {
            let class_name = "curun_window_class"
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
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

fn enable_dpi_awareness() {
    static ENABLE_DPI_AWARENESS: Once = Once::new();
    unsafe {
        ENABLE_DPI_AWARENESS.call_once(|| {
            if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == TRUE {
                return;
            } else if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE) == TRUE
            {
                return;
            } else if SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) == S_OK {
                return;
            }
        });
    }
}

pub(crate) fn find_window(hwnd: HWND) -> Option<Window> {
    CONTEXT.with(|context| {
        let context = context.borrow();
        let window_table = context.as_ref().unwrap().window_table.borrow();
        window_table.iter().find_map(|(h, window)| {
            if *h == hwnd {
                Some(window.clone())
            } else {
                None
            }
        })
    })
}

pub(crate) fn call_handler(f: impl FnOnce(&mut Box<dyn EventHandler>, &mut ContextState)) {
    CONTEXT.with(|context| {
        let context = context.borrow();
        context.as_ref().unwrap().call_handler(f);
    })
}

pub(crate) fn root_window() -> Option<Window> {
    CONTEXT.with(|context| {
        let context = context.borrow();
        let window_table = context.as_ref().unwrap().window_table.borrow();
        window_table.first().map(|elem| elem.1.clone())
    })
}

pub struct Context(Rc<ContextImpl>);

impl Context {
    pub fn new() -> Self {
        enable_dpi_awareness();
        CONTEXT.with(move |context| {
            let new_context = Rc::new(ContextImpl::new());
            *context.borrow_mut() = Some(new_context.clone());
            Self(new_context)
        })
    }

    pub fn run(self, run_type: RunType, event_handler: impl EventHandler + 'static) {
        self.0.set_event_handler(event_handler);
        let mut msg = MSG::default();
        match run_type {
            RunType::Idle => unsafe {
                while msg.message != WM_QUIT {
                    if PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    } else {
                        call_handler(|eh, _| eh.idle())
                    }
                }
            },
            RunType::Wait => unsafe {
                loop {
                    let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                    if ret == 0 || ret == -1 {
                        break;
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            },
        }
    }

    pub(crate) fn create_window<Ti, S>(&self, builder: WindowBuilder<Ti, S>) -> Window
    where
        Ti: AsRef<str>,
        S: ToPhysicalSize<f32>,
    {
        let class_name = register_class();
        let title = builder
            .title
            .as_ref()
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let mut style = WS_OVERLAPPED | WS_SYSMENU | WS_CAPTION | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
        if builder.resizable {
            style |= WS_THICKFRAME;
        }
        if builder.visibility {
            style |= WS_VISIBLE;
        }
        unsafe {
            let dpi = get_dpi_from_point(builder.position.clone());
            let rc = adjust_window_rect(
                builder.inner_size.to_physical(dpi as f32 / DEFAULT_DPI),
                WS_OVERLAPPEDWINDOW,
                0,
                dpi,
            );
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                style,
                builder.position.x,
                builder.position.y,
                (rc.right - rc.left) as i32,
                (rc.bottom - rc.top) as i32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null_mut()),
                std::ptr::null_mut(),
            );
            let window = Window(hwnd);
            self.0
                .window_table
                .borrow_mut()
                .push((hwnd, window.clone()));
            window
        }
    }
}
