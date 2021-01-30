struct Application;

impl Application {
    fn new() -> Self {
        wita::WindowBuilder::new().title("wita panic").build();
        Self
    }
}

impl wita::EventHandler for Application {
    fn activated(&mut self, _: &wita::Window) { 
        panic!("awawawawa");
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        eprintln!("drop Application");
    }
}

fn main() {
    wita::initialize::<Application>();
    wita::run(wita::RunType::Wait, Application::new());
}
