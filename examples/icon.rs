struct Application;

impl Application {
    fn new() -> anyhow::Result<Self> {
        wita::WindowBuilder::new()
            .title("wita icon")
            .icon(wita::Icon::from_path("examples/icon.ico"))
            .build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
