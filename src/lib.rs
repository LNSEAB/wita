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
//! impl wita::EventHandler for Application {
//!     fn closed(&mut self, _: &wita::Window) {
//!         println!("closed");
//!     }
//! }
//!
//! fn main() {
//!     let context = wita::Context::new();
//!     let _window = wita::WindowBuilder::new().title("hello, world!").build(&context);
//!     context.run(wita::RunType::Wait, Application);
//! }
//! ```
//!
//! # Event handling
//!
//! You must implement [`EventHandler`] for the your defined object, and can handle events in the `impl EventHandler`.
//!
//! ```no_run
//! struct Foo {}
//!
//! impl Foo {
//!     fn new() -> Self {
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
//! Next, pass the your defined object to [`Context::run`].
//!
//! ```ignore
//! let context = Context::new();
//!
//! // build the window here.
//!
//! context.run(wita::RunType::Wait, Foo::new());
//! ```
//!
//! # Drawing on the window
//! There are directly no any methods for drawing on a [`Window`] in 'wita'.
//! However, a [`Window`] provides the [`raw_handle`] that return a pointer which is `HWND`.
//! You can create a drawing context by using the [`raw_handle`] such as DirectX, Vulkan, etc.
//!
//! [`EventHandler`]: trait.EventHandler.html
//! [`Context::run`]: struct.Context.html#method.run
//! [`Window`]: struct.Window.html
//! [`raw_handle`]: struct.Window.html#method.raw_handle
//!

mod api;
mod context;
mod device;
mod event;
mod geometry;
pub mod ime;
mod procedure;
mod window;

pub use context::{Context, RunType};
pub use device::*;
pub use event::*;
pub use geometry::*;
pub use window::*;
