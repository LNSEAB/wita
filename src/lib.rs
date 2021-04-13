#![allow(clippy::needless_doctest_main)]

//! A window library in Rust for Windows.
//!
//! `wita` is a library that create a window and run an event loop.
//! It is only for Windows.
//!
//! # Example
//!
//! ```no_run
//! struct Application;
//!
//! impl Application {
//!     fn new() -> Result<Self, wita::ApiError> {
//!         wita::WindowBuilder::new()
//!             .title("hello, world!")
//!             .build()?;
//!         Ok(Self)
//!     }
//! }
//!
//! impl wita::EventHandler for Application {
//!     fn closed(&mut self, _: &wita::Window) {
//!         println!("closed");
//!     }
//! }
//!
//! fn main() {
//!     wita::run(wita::RunType::Wait, Application::new).unwrap();
//! }
//! ```
//!
//! # Event handling
//!
//! You must implement [`EventHandler`] for the your defined object, and can handle events in the `impl EventHandler`.
//!
//! ```ignore
//! struct Foo {}
//!
//! impl Foo {
//!     fn new() -> Result<Self, wita::ApiError> {
//!         wita::WindowBuilder::new().build()?;
//!         Ok(Self {})
//!     }
//! }
//!
//! impl wita::EventHandler for Foo {
//!     // You define handlers.
//!     // For example, handle the event that closed the window.
//!     fn closed(&mut self, _: &wita::Window) {
//!         // write handling codes
//!     }
//! }
//! ```
//! Next, pass the your defined object to [`run`].
//!
//! ```ignore
//! wita::run(wita::RunType::Wait, Foo::new).unwrap();
//! ```
//!
//! # Drawing on the window
//! There are directly no any methods for drawing on a [`Window`] in `wita`.
//! However, a [`Window`] provides the [`raw_handle`] that return a pointer which is `HWND`.
//! You can create a drawing context by using the [`raw_handle`] such as DirectX, Vulkan, etc.
//!
//! [`raw_handle`]: struct.Window.html#method.raw_handle
//!

#[allow(warnings)]
mod bindings {
    ::windows::include_bindings!();
}

mod api;
mod context;
mod device;
mod event;
mod geometry;
pub mod ime;
mod monitor;
mod procedure;
#[cfg(any(feature = "raw_input", doc))]
pub mod raw_input;
mod resource;
mod window;
#[macro_use]
pub mod error;

pub use context::RunType;
pub use device::*;
#[doc(inline)]
pub use error::ApiError;
pub use event::*;
pub use geometry::*;
pub use monitor::*;
pub use resource::*;
pub use window::*;

use bindings::Windows::Win32::{SystemServices::*, WindowsAndMessaging::*};
use context::*;

/// The value is an unit in logical coordinates.
pub const DEFAULT_DPI: i32 = 96;

/// Run the event loop.
pub fn run<F, T, E>(run_type: RunType, f: F) -> Result<(), E>
where
    F: FnOnce() -> Result<T, E>,
    T: EventHandler + 'static,
{
    api::enable_dpi_awareness();
    api::enable_gui_thread();
    window::register_class::<T>();
    context::create_context();
    let handler = f();
    match handler {
        Ok(handler) => set_event_handler(handler),
        Err(e) => return Err(e),
    }
    let mut msg = MSG::default();
    match run_type {
        RunType::Idle => unsafe {
            while msg.message != WM_QUIT {
                call_handler(|eh: &mut T, _| eh.pre_processing());
                if PeekMessageW(&mut msg, HWND(0), 0, 0, PeekMessage_wRemoveMsg::PM_REMOVE)
                    != BOOL(0)
                {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else {
                    call_handler(|eh: &mut T, _| eh.idle());
                }
                maybe_resume_unwind();
                call_handler(|eh: &mut T, _| eh.post_processing());
            }
        },
        RunType::Wait => unsafe {
            loop {
                let ret = GetMessageW(&mut msg, HWND(0), 0, 0);
                if ret == BOOL(0) || ret == BOOL(-1) {
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
                maybe_resume_unwind();
            }
        },
    }
    destroy_context();
    Ok(())
}
