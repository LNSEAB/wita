struct Application;

impl Application {
    fn new() -> Result<Self, wita::ApiError> {
        wita::WindowBuilder::new().title("hello, world!").build()?;
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
