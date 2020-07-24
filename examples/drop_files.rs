use std::path::Path;

struct Application;

impl wita::EventHandler for Application {
    fn drop_files(&mut self, _: &wita::Window, paths: &[&Path], position: wita::LogicalPosition<f32>) {
        println!("drop files: {:?}, {:?}", paths, position);
    }
}

fn main() {
    let context = wita::Context::new();
    let _window = wita::WindowBuilder::new()
        .title("wita drop files")
        .accept_drag_files(true)
        .build(&context);
    context.run(wita::RunType::Wait, Application);
}