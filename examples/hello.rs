struct Application;

impl Application {
    fn new() -> Self {
        wita::WindowBuilder::new().title("hello, world!").build();
        Self
    }
}

impl wita::EventHandler for Application {
    fn closed(&mut self, _: &wita::Window) {
        println!("closed");
    }
}

fn main() {
    wita::initialize::<Application>();
    wita::run(wita::RunType::Wait, Application::new());
}
