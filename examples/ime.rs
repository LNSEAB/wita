use wita::*;

struct Application;

impl EventHandler for Application {
    fn ime_start_composition(&mut self, _: &Window) {
        println!("ime start composition");
    }

    fn ime_composition(&mut self, _: &Window, s: &str) {
        println!("ime composition: {}", s);
    }

    fn ime_end_composition(&mut self, _: &Window, s: Option<&str>) {
        println!("ime end composition: {:?}", s);
    }
}

fn main() {
    let context = Context::new();
    let window = WindowBuilder::new()
        .title("wita ime")
        .visible_composition_window(true)
        .visible_candidate_window(true)
        .build(&context);
    window.set_ime_position(LogicalPosition::new(100.0, 100.0));
    context.run(RunType::Wait, Application);
}