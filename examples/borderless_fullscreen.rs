struct Application;

impl Application {
    fn new() -> anyhow::Result<Self> {
        let monitor = wita::monitor_from_point(wita::ScreenPosition::new(0, 0)).unwrap();
        wita::WindowBuilder::new()
            .inner_size(monitor.size)
            .style(wita::WindowStyle::borderless())
            .build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {
    fn key_input(
        &mut self,
        window: &wita::Window,
        code: wita::KeyCode,
        state: wita::KeyState,
        _: bool,
    ) {
        if state == wita::KeyState::Pressed {
            if let wita::VirtualKey::Char(_) = code.vkey {
                window.close();
            }
        }
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
