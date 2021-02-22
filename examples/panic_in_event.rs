struct Application;

impl Application {
    fn new() -> anyhow::Result<Self> {
        wita::WindowBuilder::new().title("wita panic").build()?;
        Ok(Self)
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
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
