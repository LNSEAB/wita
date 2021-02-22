# wita

[![wita at crates.io](https://img.shields.io/crates/v/wita.svg)](https://crates.io/crates/wita)
[![wita at docs.rs](https://docs.rs/wita/badge.svg)](https://docs.rs/wita)

A window library in Rust for Windows

## Hello, world!
```rust
struct Application;

impl Application {
    fn new() -> Result<Self, wita::ApiError> {
        wita::WindowBuilder::new()
            .title("hello, world!")
            .build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {
    fn closed(&mut self, _: &wita::Window) {
        println!("closed");
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([licenses/LICENSE-APACHE](licenses/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([licenses/LICENSE-MIT](licenses/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
