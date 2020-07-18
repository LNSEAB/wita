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
