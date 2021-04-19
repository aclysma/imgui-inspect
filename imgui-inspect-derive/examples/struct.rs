use imgui_inspect_derive::Inspect;

#[derive(Inspect)]
pub struct Circle {
    #[inspect_slider(min_value = 100.0, max_value = 500.0)]
    x: f32,
    #[inspect_slider(min_value = 100.0, max_value = 400.0)]
    y: f32,
    #[inspect_slider(min_value = 20.0, max_value = 100.0)]
    radius: f32,
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            x: 300.0,
            y: 250.0,
            radius: 50.0,
        }
    }
}

fn main() {}
