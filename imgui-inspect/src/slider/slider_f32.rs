
use super::*;

impl InspectRenderSlider<f32> for f32 {
    fn render(data: &[&Self], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsSlider) {
        if data.len() == 0 {
            ui.text(&imgui::im_str!("{}: None", label));
            return;
        }

        ui.text(&imgui::im_str!("{}: {}", label, data[0]));
    }

    fn render_mut(
        data: &mut [&mut Self],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsSlider,
    ) -> bool {
        if data.len() == 0 {
            ui.text(&imgui::im_str!("{}: None", label));
            return false;
        }

        let mut min = -100.0;
        let mut max = 100.0;
        if let Some(min_value) = args.min_value {
            min = min_value;
        }

        if let Some(max_value) = args.max_value {
            max = max_value;
        }

        ui.slider_float(&imgui::im_str!("{}", label), data[0], min, max)
            .build()
    }
}
