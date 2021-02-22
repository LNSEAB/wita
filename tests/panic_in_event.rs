struct Application;

impl Application {
    fn new() -> anyhow::Result<Self> {
        wita::WindowBuilder::new().build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {
    fn activated(&mut self, _: &wita::Window) {
        panic!("success");
    }
}

#[test]
#[should_panic]
pub fn panic_in_event() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
