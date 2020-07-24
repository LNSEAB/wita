use std::path::Path;

struct Application {
    accept_drag_files: bool,
}

impl Application {
    fn new() -> Self {
        Self {
            accept_drag_files: true,
        }
    }
}

impl wita::EventHandler for Application {
    fn drop_files(&mut self, _: &wita::Window, paths: &[&Path], position: wita::LogicalPosition<f32>) {
        println!("drop files: {:?}, {:?}", paths, position);
    }

    fn key_input(&mut self, window: &wita::Window, code: wita::KeyCode, state: wita::KeyState) {
        if state == wita::KeyState::Pressed {
            if code.vkey == wita::VirtualKey::Char('T') {
                self.accept_drag_files = !self.accept_drag_files;
                window.accept_drag_files(self.accept_drag_files);
                if self.accept_drag_files {
                    println!("enabled accept_drag_files");
                } else {
                    println!("disabled accept_drag_files");
                }
            }
        }
    }
}

fn main() {
    let context = wita::Context::new();
    let _window = wita::WindowBuilder::new()
        .title("wita drop files")
        .accept_drag_files(true)
        .build(&context);
    context.run(wita::RunType::Wait, Application::new());
}