#[derive(Debug, Default)]
pub struct InspectArgsDefault {
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub step: Option<f32>,
}

#[derive(Debug, Default)]
pub struct InspectArgsSlider {
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
}

pub trait InspectRenderDefault<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault);
    fn render_mut(
        data: &mut [&mut T],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool;
}

pub trait InspectRenderSlider<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsSlider);
    fn render_mut(
        data: &mut [&mut T],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsSlider,
    ) -> bool;
}

fn get_same_or_none<T: PartialEq + Clone>(data: &[&T]) -> Option<T> {
    if data.len() == 0 {
        return None;
    }

    let first = data[0].clone();
    for d in data {
        if **d != first {
            return None;
        }
    }

    Some(first)
}

fn get_same_or_none_mut<T: PartialEq + Clone>(data: &mut [&mut T]) -> Option<T> {
    if data.len() == 0 {
        return None;
    }

    let first = data[0].clone();
    for d in data {
        if **d != first {
            return None;
        }
    }

    Some(first)
}

impl InspectRenderDefault<bool> for bool {
    fn render(data: &[&bool], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsDefault) {
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
        data: &mut [&mut bool],
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let same_or_none_value = get_same_or_none_mut(data);

        let mut value = match same_or_none_value {
            Some(v) => v,
            None => false, // Some reasonable default
        };

        let _style_token = if same_or_none_value.is_none() {
            // If values are inconsistent, push a style
            Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        } else {
            None
        };

        let mut changed = false;
        if ui.checkbox(&imgui::im_str!("{}", label), &mut value) {
            for d in data {
                **d = value;
                changed = true;
            }
        }

        changed
    }
}

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

impl<T: InspectRenderDefault<T>> InspectRenderDefault<Option<T>> for Option<T> {
    fn render(data: &[&Option<T>], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        if data.len() == 0 {
            ui.text(&imgui::im_str!("{}: None", label));
            return;
        }

        let d = data[0];
        match d {
            Some(value) => <T as InspectRenderDefault<T>>::render(&[value], label, ui, args),
            None => ui.text(&imgui::im_str!("{}: None", label)),
        };
    }

    fn render_mut(
        data: &mut [&mut Option<T>],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() == 0 {
            ui.text(&imgui::im_str!("{}: None", label));
            return false;
        }

        let d = &mut data[0];
        match d {
            Some(value) => {
                <T as InspectRenderDefault<T>>::render_mut(&mut [value], label, ui, args)
            }
            None => {
                ui.text(&imgui::im_str!("{}: None", label));
                return false;
            }
        }
    }
}

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
