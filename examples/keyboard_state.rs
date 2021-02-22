struct Application;

impl Application {
    fn new() -> anyhow::Result<Self> {
        wita::WindowBuilder::new()
            .title("wita keyboard state")
            .build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {
    fn key_input(&mut self, _: &wita::Window, _: wita::KeyCode, state: wita::KeyState, _: bool) {
        if state == wita::KeyState::Pressed {
            let ks = wita::keyboard_state();
            println!("{:?}", ks);
        }
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
