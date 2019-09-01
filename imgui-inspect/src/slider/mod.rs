
mod slider_f32;

pub use super::*;

/// Options for rendering a values as a slider
#[derive(Debug, Default)]
pub struct InspectArgsSlider {
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
}

/// Renders a value as a slider
pub trait InspectRenderSlider<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsSlider);
    fn render_mut(
        data: &mut [&mut T],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsSlider,
    ) -> bool;
}