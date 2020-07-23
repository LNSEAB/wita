struct Application;

impl wita::EventHandler for Application {
    fn closed(&mut self, window: &wita::Window) {
        println!("closed {}", window.title());
    }
}

fn main() {
    let context = wita::Context::new();
    let mut windows = Vec::new();
    for i in 0..3 {
        let window = wita::WindowBuilder::new()
            .title(format!("wita window {}", i))
            .position(wita::ScreenPosition::new(30 * i, 30 * i));
        let window = if i > 0 {
            window.parent(&windows[i as usize - 1])
        } else {
            window
        };
        windows.push(window.build(&context));
    }
    context.run(wita::RunType::Wait, Application);
}