
use super::*;

impl<T: InspectRenderDefault<T, C>, C> InspectRenderDefault<Option<T>, C> for Option<T> {
    fn render(data: &[&Option<T>], context: C, label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        if data.len() == 0 {
            ui.text(&imgui::im_str!("{}: None", label));
            return;
        }

        let d = data[0];
        match d {
            Some(value) => <T as InspectRenderDefault<T, C>>::render(&[value], context, label, ui, args),
            None => ui.text(&imgui::im_str!("{}: None", label)),
        };
    }

    fn render_mut(
        data: &mut [&mut Option<T>],
        context: C,
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
                <T as InspectRenderDefault<T, C>>::render_mut(&mut [value], context, label, ui, args)
            }
            None => {
                ui.text(&imgui::im_str!("{}: None", label));
                return false;
            }
        }
    }
}
