struct Application {
    count: u64,
}

impl Application {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl wita::EventHandler for Application {
    fn mouse_input(
        &mut self,
        _: &wita::Window,
        button: wita::MouseButton,
        button_state: wita::KeyState,
        _: wita::MouseState,
    ) {
        if button == wita::MouseButton::Left && button_state == wita::KeyState::Pressed {
            self.count += 1;
            println!("count = {}", self.count);
        }
    }
}

fn main() {
    let context = wita::Context::new();
    let _window = wita::WindowBuilder::new()
        .title("wita count")
        .inner_size(wita::LogicalSize::new(256.0, 256.0))
        .style(
            wita::WindowStyle::default()
                .resizable(false)
                .has_minimize_box(false)
                .has_maximize_box(false),
        )
        .build(&context);
    context.run(wita::RunType::Wait, Application::new());
}
