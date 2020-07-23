# wita

A window library in Rust for Windows

## Hello, world!
```rust
struct Application;

impl wita::EventHandler for Application {
    fn closed(&mut self, _: &wita::Window) {
        println!("closed");
    }
}

fn main() {
    let context = wita::Context::new();
    let _window = wita::WindowBuilder::new()
        .title("hello, world!")
        .build(&context);
    context.run(wita::RunType::Wait, Application);
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
