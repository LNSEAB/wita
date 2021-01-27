struct Application;

impl Application {
    fn new() -> Self {
        wita::WindowBuilder::new()
            .title("icon_resource")
            .icon(wita::Icon::Resource(111))
            .build();
        Self
    }
}

impl wita::EventHandler for Application {}

fn main() {
    wita::initialize::<Application>();
    wita::run(wita::RunType::Wait, Application::new());
}
