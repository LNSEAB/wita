struct Application;

impl Application {
    fn new() -> Result<Self, wita::ApiError> {
        wita::WindowBuilder::new().title("hello, world!").build()?;
        Ok(Self)
    }
}

impl wita::EventHandler for Application {
    fn key_input(
        &mut self,
        window: &wita::Window,
        key_code: wita::KeyCode,
        state: wita::KeyState,
        _prev_pressed: bool,
    ) {
        if state == wita::KeyState::Pressed {
            let cursor = match key_code.vkey {
                wita::VirtualKey::Char('D') => wita::Cursor::Arrow,
                wita::VirtualKey::Char('H') => wita::Cursor::Hand,
                wita::VirtualKey::Char('I') => wita::Cursor::IBeam,
                wita::VirtualKey::Char('W') => wita::Cursor::Wait,
                _ => return,
            };
            window.set_cursor(cursor);
        }
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
