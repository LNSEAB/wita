struct Application;

impl wita::EventHandler for Application {
    fn key_input(&mut self, window: &wita::Window, code: wita::KeyCode, state: wita::KeyState) {
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
    let context = wita::Context::new();
    let _window = wita::WindowBuilder::new()
        .style(wita::WindowStyle::borderless())
        .build(&context);
    context.run(wita::RunType::Wait, Application);
}
