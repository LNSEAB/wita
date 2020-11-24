struct Application;

impl Application {
    fn new() -> Self {
        let monitor = wita::monitor_from_point(wita::ScreenPosition::new(0, 0)).unwrap();
        wita::WindowBuilder::new()
            .inner_size(monitor.size)
            .style(wita::WindowStyle::borderless())
            .build();
        Self
    }
}

impl wita::EventHandler for Application {
    fn key_input(&mut self, window: &wita::Window, code: wita::KeyCode, state: wita::KeyState) {
        if state == wita::KeyState::Pressed {
            if let wita::VirtualKey::Char(_) = code.vkey {
                window.close();
            }
        }
    }
}

fn main() {
    wita::initialize::<Application>();
    wita::run(wita::RunType::Wait, Application::new());
}
