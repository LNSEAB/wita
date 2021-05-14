use crate::bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use crate::{device::*, event::EventHandler, window::LocalWindow};
use std::any::Any;
use std::cell::RefCell;
use std::panic::resume_unwind;

/// Describes event loop types.
pub enum RunType {
    Idle,
    Wait,
}

pub(crate) struct ContextState {
    pub mouse_buttons: Vec<MouseButton>,
    pub entered_window: Option<LocalWindow>,
    pub resizing: bool,
}

impl ContextState {
    fn new() -> Self {
        Self {
            mouse_buttons: Vec::with_capacity(5),
            entered_window: None,
            resizing: false,
        }
    }
}

pub(crate) struct Context {
    state: ContextState,
    window_table: Vec<(HWND, LocalWindow)>,
    event_handler: Option<Box<dyn Any>>,
    unwind: Option<Box<dyn Any + Send>>,
}

impl Context {
    fn new() -> Self {
        Self {
            state: ContextState::new(),
            window_table: Vec::new(),
            event_handler: None,
            unwind: None,
        }
    }
}

thread_local! {
    static CONTEXT: RefCell<*mut Context> = RefCell::new(std::ptr::null_mut());
}

#[inline]
pub fn create_context() {
    CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Box::into_raw(Box::new(Context::new()));
    });
}

#[inline]
pub fn is_context_null() -> bool {
    CONTEXT.with(|ctx| ctx.borrow().is_null())
}

#[inline]
pub(crate) fn push_window(hwnd: HWND, wnd: LocalWindow) {
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &mut *p;
        ctx.window_table.push((hwnd, wnd));
    }
}

#[inline]
pub(crate) fn find_window(hwnd: HWND) -> Option<LocalWindow> {
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &*p;
        ctx.window_table.iter().find_map(
            |(h, wnd)| {
                if *h == hwnd {
                    Some(wnd.clone())
                } else {
                    None
                }
            },
        )
    }
}

#[inline]
pub fn remove_window(hwnd: HWND) {
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &mut *p;
        ctx.window_table.remove(
            ctx.window_table
                .iter()
                .position(|(h, _)| *h == hwnd)
                .unwrap(),
        );
    }
}

#[inline]
pub fn window_table_is_empty() -> bool {
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &*p;
        ctx.window_table.is_empty()
    }
}

#[inline]
pub fn set_resizing(state: bool) {
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &mut *p;
        ctx.state.resizing = state;
    }
}

#[inline]
pub fn set_event_handler(eh: impl EventHandler + 'static) {
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &mut *p;
        ctx.event_handler = Some(Box::new(eh));
    }
}

#[inline]
pub(crate) fn call_handler<F, T>(f: F)
where
    F: FnOnce(&mut T, &mut ContextState),
    T: EventHandler + 'static,
{
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &mut *p;
        if ctx.event_handler.is_some() {
            let event_handler = ctx
                .event_handler
                .as_mut()
                .unwrap()
                .downcast_mut::<T>()
                .unwrap();
            f(event_handler, &mut ctx.state);
        }
    }
}

#[inline]
pub fn set_unwind(e: Box<dyn Any + Send>) {
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &mut *p;
        ctx.event_handler = None;
        ctx.unwind = Some(e);
    }
}

#[inline]
pub fn maybe_resume_unwind() {
    let p = CONTEXT.with(|ctx| *ctx.borrow());
    unsafe {
        let ctx = &mut *p;
        if let Some(e) = ctx.unwind.take() {
            resume_unwind(e);
        }
    }
}

#[inline]
pub fn destroy_context() {
    CONTEXT.with(|ctx| unsafe {
        let mut p = ctx.borrow_mut();
        Box::from_raw(*p);
        *p = std::ptr::null_mut();
    });
}
