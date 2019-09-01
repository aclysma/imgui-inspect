
use super::*;

impl InspectRenderDefault<f32> for f32 {
    fn render(data: &[&f32], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsDefault) {
        if data.len() == 0 {
            // Values are inconsistent
            let _style_token = ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);
            ui.text(&imgui::im_str!("{}: ", label));
            return;
        }

        match get_same_or_none(data) {
            Some(_v) => {
                // Values are consistent
                ui.text(&imgui::im_str!("{}: {}", label, data[0]))
            }
            None => {
                // Values are inconsistent
                let _style_token =
                    ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]);
                ui.text(&imgui::im_str!("{}: ", label));
            }
        }
    }

    fn render_mut(
        data: &mut [&mut f32],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let same_or_none_value = get_same_or_none_mut(data);

        let mut value = match same_or_none_value {
            Some(v) => v,
            None => 0.0, // Some reasonable default
        };

        let _style_token = if same_or_none_value.is_none() {
            // If values are inconsistent, push a style
            Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        } else {
            None
        };

        let mut changed = false;
        if ui
            .input_float(&imgui::im_str!("{}", label), &mut value)
            .build()
        {
            for d in data {
                **d = value;
                changed = true;
            }
        }

        changed
    }
}
