struct Application;

impl Application {
    fn new() -> Self {
        wita::WindowBuilder::new()
            .title("wita window params")
            .build();
        Self
    }
}

impl wita::EventHandler for Application {
    fn resized(&mut self, _: &wita::Window, size: wita::PhysicalSize<f32>) {
        println!("resized: {:?}", size);
    }

    fn moved(&mut self, _: &wita::Window, pos: wita::ScreenPosition) {
        println!("moved: {:?}", pos);
    }

    fn key_input(&mut self, window: &wita::Window, code: wita::KeyCode, state: wita::KeyState) {
        if state == wita::KeyState::Pressed {
            match code.vkey {
                wita::VirtualKey::Char('M') => {
                    window.set_position(wita::ScreenPosition::new(100, 100))
                }
                wita::VirtualKey::Char('R') => window.set_position(wita::ScreenPosition::new(0, 0)),
                wita::VirtualKey::Char('S') => {
                    window.set_inner_size(wita::LogicalSize::new(256.0, 256.0))
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
