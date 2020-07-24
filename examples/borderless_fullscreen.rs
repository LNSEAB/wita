struct Application;

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
    let context = wita::Context::new();
    let monitor = wita::monitor_from_point(wita::ScreenPosition::new(0, 0)).unwrap();
    let _window = wita::WindowBuilder::new()
        .inner_size(monitor.size)
        .style(wita::WindowStyle::borderless())
        .build(&context);
    context.run(wita::RunType::Wait, Application);
}
