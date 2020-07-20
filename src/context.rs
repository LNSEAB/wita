use crate::{
    device::*,
    event::EventHandler,
    window::Window,
};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::sync::Once;
use std::panic::resume_unwind;
use winapi::shared::{minwindef::*, windef::*, winerror::S_OK};
use winapi::um::{
    shellscalingapi::*,
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
    pub entered_window: Option<Window>,
}

impl ContextState {
    fn new() -> Self {
        Self {
            mouse_buttons: Vec::with_capacity(5),
            entered_window: None,
        }
    }
}

pub(crate) struct ContextImpl {
    event_handler: RefCell<Option<Box<dyn EventHandler>>>,
    window_table: RefCell<Vec<(HWND, Window)>>,
    state: RefCell<ContextState>,
    unwind: RefCell<Option<Box<dyn std::any::Any + Send>>>,
}

impl ContextImpl {
    fn new() -> Self {
        Self {
            event_handler: RefCell::new(None),
            window_table: RefCell::new(Vec::new()),
            state: RefCell::new(ContextState::new()),
            unwind: RefCell::new(None),
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

fn enable_dpi_awareness() {
    static ENABLE_DPI_AWARENESS: Once = Once::new();
    unsafe {
        ENABLE_DPI_AWARENESS.call_once(|| {
            if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == TRUE {
                return;
            } else if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE) == TRUE {
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
        window_table
            .iter()
            .find_map(|(h, window)| if *h == hwnd { Some(window.clone()) } else { None })
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

pub(crate) fn set_unwind(payload: Box<dyn std::any::Any + Send>) {
    CONTEXT.with(|context| {
        let context = context.borrow();
        *context.as_ref().unwrap().unwind.borrow_mut() = Some(payload);
    });
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
                    if let Some(e) = self.0.unwind.borrow_mut().take() {
                        resume_unwind(e);
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
                    if let Some(e) = self.0.unwind.borrow_mut().take() {
                        resume_unwind(e);
                    }
                }
            },
        }

    }

    pub(crate) fn window_table(&self) -> RefMut<Vec<(HWND, Window)>> {
        self.0.window_table.borrow_mut()
    }
}
