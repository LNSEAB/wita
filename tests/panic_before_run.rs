struct Application;

impl Application {
    fn new() -> anyhow::Result<Self> {
        wita::WindowBuilder::new().build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {}

#[test]
#[should_panic]
pub fn before_run() {
    wita::WindowBuilder::new().build().unwrap();
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
