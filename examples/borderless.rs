struct Application;

impl Application {
    fn new() -> Self {
        wita::WindowBuilder::new()
            .style(wita::WindowStyle::borderless())
            .build();
        Self
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
            match code.vkey {
                wita::VirtualKey::Char('Q') => window.close(),
                wita::VirtualKey::Char('T') => {
                    if window.style().is_borderless() {
                        window.set_style(wita::WindowStyle::default());
                    } else {
                        window.set_style(wita::WindowStyle::borderless());
                    }
                }
                _ => (),
            }
        }
    }
}

fn main() {
    wita::initialize::<Application>();
    wita::run(wita::RunType::Wait, Application::new());
}
