
mod default_bool;
mod default_f32;
mod default_option;
mod default_u32;
mod default_usize;

pub use super::*;

/// Options for using the default rendering style for the element. The options here are a superset
/// of all other options since "default" could be any of the widgets
#[derive(Debug, Default)]
pub struct InspectArgsDefault {
    pub header: Option<bool>,
    pub indent_children: Option<bool>,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub step: Option<f32>,
}

/// Renders a value using the default widget
pub trait InspectRenderDefault<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault);
    fn render_mut(
        data: &mut [&mut T],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool;
}