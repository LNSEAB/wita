use crate::{device::*, event::EventHandler, window::Window};
use std::any::Any;
use std::cell::RefCell;
use std::panic::resume_unwind;
use winapi::shared::windef::*;

/// Describes event loop types.
pub enum RunType {
    Idle,
    Wait,
}

pub(crate) struct ContextState {
    pub mouse_buttons: Vec<MouseButton>,
    pub entered_window: Option<Window>,
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
    window_table: Vec<(HWND, Window)>,
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

    pub fn state_mut(&mut self) -> &mut ContextState {
        &mut self.state
    }
}

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

#[inline]
pub fn create_context() {
    CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(Context::new());
    });
}

#[inline]
pub(crate) fn context_ref<F, R>(f: F) -> R
where
    F: FnOnce(&Context) -> R,
{
    CONTEXT.with(|ctx| f(ctx.borrow().as_ref().unwrap()))
}

#[inline]
pub(crate) fn context_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Context) -> R,
{
    CONTEXT.with(|ctx| f(ctx.borrow_mut().as_mut().unwrap()))
}

#[inline]
pub fn push_window(hwnd: HWND, wnd: Window) {
    context_mut(|ctx| ctx.window_table.push((hwnd, wnd)))
}

#[inline]
pub fn find_window(hwnd: HWND) -> Option<Window> {
    context_ref(|ctx| {
        ctx.window_table.iter().find_map(
            |(h, wnd)| {
                if *h == hwnd {
                    Some(wnd.clone())
                } else {
                    None
                }
            },
        )
    })
}

#[inline]
pub fn remove_window(hwnd: HWND) {
    context_mut(|ctx| {
        ctx.window_table.remove(
            ctx.window_table
                .iter()
                .position(|(h, _)| *h == hwnd)
                .unwrap(),
        );
    })
}

#[inline]
pub fn window_table_is_empty() -> bool {
    context_ref(|ctx| ctx.window_table.is_empty())
}

#[inline]
pub fn set_event_handler(eh: impl EventHandler + 'static) {
    context_mut(|ctx| ctx.event_handler = Some(Box::new(eh)));
}

#[inline]
pub(crate) fn call_handler<F, T>(f: F)
where
    F: FnOnce(&mut T, &mut ContextState),
    T: EventHandler + 'static,
{
    context_mut(|ctx| {
        let event_handler = ctx
            .event_handler
            .as_mut()
            .unwrap()
            .downcast_mut::<T>()
            .unwrap();
        f(event_handler, &mut ctx.state);
    });
}

#[inline]
pub fn set_unwind(e: Box<dyn Any + Send>) {
    context_mut(|ctx| ctx.unwind = Some(e));
}

#[inline]
pub fn maybe_resume_unwind() {
    context_mut(|ctx| {
        if let Some(e) = ctx.unwind.take() {
            resume_unwind(e);
        }
    });
}

#[inline]
pub fn destroy_context() {
    CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    })
}