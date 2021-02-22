struct Application;

impl Application {
    fn new() -> anyhow::Result<Self> {
        wita::WindowBuilder::new().build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {
    fn activated(&mut self, wnd: &wita::Window) {
        wnd.close();
    }
}

#[test]
pub fn build_window() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
