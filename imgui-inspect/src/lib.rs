
#[derive(Debug, Default)]
pub struct InspectArgsDefault {
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub step: Option<f32>,
}

#[derive(Debug, Default)]
pub struct InspectArgsSlider {
    pub min_value: Option<f32>,
    pub max_value: Option<f32>
}

pub trait InspectRenderDefault<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault);
    fn render_mut(data: &mut [&mut T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault);
}

pub trait InspectRenderSlider<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsSlider);
    fn render_mut(data: &mut [&mut T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsSlider);
}

fn get_same_or_none<T : PartialEq + Clone>(data: &[&T]) -> Option<T> {
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

fn get_same_or_none_mut<T : PartialEq + Clone>(data: &mut [&mut T]) -> Option<T> {
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

impl InspectRenderDefault<f32> for f32 {
    fn render(data: &[&f32], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsDefault) {
        match get_same_or_none(data) {
            Some(_v) => {
                // Values are consistent
                ui.text(&imgui::im_str!("{}: {}", label, data[0]))
            },
            None => {
                // Values are inconsistent
                let _style_token = ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]);
                ui.text(&imgui::im_str!("{}: ", label));
            }
        }
    }

    fn render_mut(data: &mut [&mut f32], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsDefault) {
        let same_or_none_value = get_same_or_none_mut(data);

        let mut value = match same_or_none_value {
            Some(v) => v,
            None => 0.0 // Some reasonable default
        };

        let _style_token = if same_or_none_value.is_none() {
            // If values are inconsistent, push a style
            Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        } else {
            None
        };

        if ui.input_float(&imgui::im_str!("{}", label), &mut value).build() {
            for d in data {
                **d = value;
            }
        }
    }
}

impl<T : InspectRenderDefault<T>> InspectRenderDefault<Option<T>> for Option<T> {
    fn render(data: &[&Option<T>], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        let d = data[0];
        match d {
            Some(value) => <T as InspectRenderDefault<T>>::render(&[value], label, ui, args),
            None => ui.text(&imgui::im_str!("{}: None", label)),
        };
    }

    fn render_mut(data: &mut [&mut Option<T>], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        let d = &mut data[0];
        match d {
            Some(value) => <T as InspectRenderDefault<T>>::render_mut(&mut [value], label, ui, args),
            None => ui.text(&imgui::im_str!("{}: None", label))
        }
    }
}

impl InspectRenderSlider<f32> for f32 {
    fn render(data: &[&Self], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsSlider) {
        ui.text(&imgui::im_str!("{}: {}", label, data[0]));
    }

    fn render_mut(data: &mut [&mut Self], label: &'static str, ui: &imgui::Ui, args: &InspectArgsSlider) {
        println!("{:#?}", args);
        let mut min = -100.0;
        let mut max = 100.0;
        if let Some(min_value) = args.min_value {
            min = min_value;
        }

        if let Some(max_value) = args.max_value {
            max = max_value;
        }

        let _slider = ui.slider_float(&imgui::im_str!("{}", label), data[0], min, max).build();
    }
}




/*
#[derive(imgui_inspect_derive::Inspect, Clone)]
pub struct MyStruct {
    pub a: f32,
    pub b: f32,
    //pub c: glm::Vec2,
    //pub d: glm::Vec3
}

#[derive(imgui_inspect_derive::Inspect, Clone)]
pub struct MyStruct2 {

    #[inspect(min_value = 5.0, max_value = 42.0)]
    pub a: f32,
    #[inspect_slider(wrapping_type = "Testingf32", min_value = 5.0, max_value = 53.0)]
    pub b: f32,
    //#[inspect(wrapping_type = "TestingVec2")]
    //pub c: glm::Vec2,
    //#[inspect(min_value = 100.0)]
    //pub d: glm::Vec3,

    pub s: MyStruct
}
*/
/*
struct Testingf32;
impl InspectRenderSlider<f32> for Testingf32 {
    fn render(data: &[&f32], label: &'static str, ui: &imgui::Ui, args: &InspectArgsSlider) {
        <f32 as InspectRenderSlider<f32>>::render(data, label, ui, args);
    }
    fn render_mut(data: &mut [&mut f32], label: &'static str, ui: &imgui::Ui, args: &InspectArgsSlider) {
        <f32 as InspectRenderSlider<f32>>::render_mut(data, label, ui, args);
    }
}
*/
//struct TestingVec2;
//impl InspectRenderDefault<glm::Vec2> for TestingVec2 {
//    fn render(data: &[&glm::Vec2], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
//        <glm::Vec2 as InspectRenderDefault<glm::Vec2>>::render(data, label, ui, args);
//    }
//    fn render_mut(data: &mut [&mut glm::Vec2], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
//        <glm::Vec2 as InspectRenderDefault<glm::Vec2>>::render_mut(data, label, ui, args);
//    }
//}
