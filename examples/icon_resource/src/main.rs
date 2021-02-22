struct Application;

impl Application {
    fn new() -> anyhow::Result<Self> {
        wita::WindowBuilder::new()
            .title("wita icon_resource")
            .icon(wita::Icon::Resource(111))
            .build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
