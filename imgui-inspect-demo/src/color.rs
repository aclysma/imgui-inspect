
use skulpin::skia_safe;
use skulpin::imgui;
use imgui_inspect::InspectRenderDefault;
use imgui_inspect::InspectArgsDefault;

use crate::SpecsWorld;

// This struct demonstrates how to wrap an existing type that you might be using from another crate
// and manually implement your own inspect handler
#[derive(Clone)]
pub struct Color(pub skia_safe::Color4f);

impl InspectRenderDefault<Color, &SpecsWorld> for Color {
    fn render(
        data: &[&Color],
        _context: &SpecsWorld,
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.len() == 0 {
            return;
        }

        imgui::ColorButton::new(
            &imgui::im_str!("{}", label),
            [data[0].0.r, data[0].0.g, data[0].0.b, data[0].0.a],
        )
            .build(ui);
    }

    fn render_mut(
        data: &mut [&mut Color],
        _context: &SpecsWorld,
        label: &'static str,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() == 0 {
            return false;
        }

        let mut changed = false;
        let mut val = [data[0].0.r, data[0].0.g, data[0].0.b, data[0].0.a];
        if imgui::ColorEdit::new(
            &imgui::im_str!("{}", label),
            imgui::EditableColor::from(&mut val),
        )
            .build(ui)
        {
            changed = true;
            for d in data {
                d.0.r = val[0];
                d.0.g = val[1];
                d.0.b = val[2];
                d.0.a = val[3];
            }
        }

        changed
    }
}