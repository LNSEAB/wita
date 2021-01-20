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
//!     fn new() -> Self {
//!         wita::WindowBuilder::new()
//!             .title("hello, world!")
//!             .build();
//!         Self
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
//!     wita::initialize::<Application>();
//!     wita::run(wita::RunType::Wait, Application);
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
//!     fn new() -> Self {
//!         // 
//!         Self {}
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
//! wita::initialize::<Foo>();
//! wita::run(wita::RunType::Wait, Foo::new());
//! ```
//!
//! # Drawing on the window
//! There are directly no any methods for drawing on a [`Window`] in `wita`.
//! However, a [`Window`] provides the [`raw_handle`] that return a pointer which is `HWND`.
//! You can create a drawing context by using the [`raw_handle`] such as DirectX, Vulkan, etc.
//!
//! [`raw_handle`]: struct.Window.html#method.raw_handle
//!

mod api;
mod context;
mod device;
mod event;
mod geometry;
pub mod ime;
mod monitor;
mod procedure;
mod window;

pub use context::RunType;
pub use device::*;
pub use event::*;
pub use geometry::*;
pub use monitor::*;
pub use window::*;

use context::*;
use std::ptr::null_mut;
use winapi::um::winuser::*;

pub const DEFAULT_DPI: i32 = 96;

/// Initialize `wita`.
pub fn initialize<T: EventHandler + 'static>() {
    api::enable_dpi_awareness();
    window::register_class::<T>();
    api::enable_gui_thread();
    context::create_context();
}

/// Run the event loop.
pub fn run<T: EventHandler + 'static>(run_type: RunType, handler: T) {
    set_event_handler(handler);
    let mut msg = MSG::default();
    match run_type {
        RunType::Idle => unsafe {
            while msg.message != WM_QUIT {
                if PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else {
                    call_handler(|eh: &mut T, _| eh.idle());
                }
                maybe_resume_unwind();
            }
        },
        RunType::Wait => unsafe {
            loop {
                let ret = GetMessageW(&mut msg, null_mut(), 0, 0);
                if ret == 0 || ret == -1 {
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
                maybe_resume_unwind();
            }
        },
    }
    destroy_context();
}
