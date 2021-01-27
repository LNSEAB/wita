struct Application;

impl Application {
    fn new() -> Self {
        wita::WindowBuilder::new()
            .title("icon")
            .icon(wita::Icon::from_path("examples/icon.ico"))
            .build();
        Self
    }
}

impl wita::EventHandler for Application {}

fn main() {
    wita::initialize::<Application>();
    wita::run(wita::RunType::Wait, Application::new());
}
