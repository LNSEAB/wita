//! A window library in Rust for Windows.
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
//! struct Foo {};
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
//! context.run(wita::RunType::Wait, Foo::new());
//! ```
//!
//! [`EventHandler`]: trait.EventHandler.html
//! [`Context::run`]: struct.Context.html#method.run
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
