// This example does a physics demo, because physics is fun :)

use skulpin::{AppHandler, AppUpdateArgs, AppDrawArgs};
use skulpin::VirtualKeyCode;
use skulpin::LogicalSize;
use skulpin::skia_safe;
use skulpin::imgui;
use imgui_inspect_derive::Inspect;

use std::ffi::CString;
use imgui_inspect::InspectArgsStruct;

mod color;
use color::Color;

// This struct is a simple example
#[derive(Inspect)]
struct ExampleInspectTarget {
    #[inspect_slider(min_value = 100.0, max_value = 500.0)]
    x_position: f32,

    #[inspect_slider(min_value = 100.0, max_value = 400.0)]
    y_position: f32,

    #[inspect_slider(min_value = 20.0, max_value = 100.0)]
    radius: f32,

    // This type has a custom handler, see color.rs
    color: Color,

    // String is supported as well
    text: String
}

impl Default for ExampleInspectTarget {
    fn default() -> Self {
        ExampleInspectTarget {
            x_position: 300.0,
            y_position: 250.0,
            radius: 50.0,
            color: Color(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0)),
            text: "".to_string()
        }
    }
}

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let example_app = ExampleApp::new();

    skulpin::AppBuilder::new()
        .app_name(CString::new("imgui-inspect demo").unwrap())
        .use_vulkan_debug_layer(true)
        .logical_size(LogicalSize::new(900.0, 600.0))
        .run(example_app);
}

struct ExampleApp {
    last_fps_text_change: Option<std::time::Instant>,
    fps_text: String,
    example_inspect_target: ExampleInspectTarget
}

impl ExampleApp {
    pub fn new() -> Self {
        ExampleApp {
            last_fps_text_change: None,
            fps_text: "".to_string(),
            example_inspect_target: Default::default()
        }
    }
}

impl AppHandler for ExampleApp {
    fn update(
        &mut self,
        update_args: AppUpdateArgs,
    ) {
        let time_state = update_args.time_state;
        let input_state = update_args.input_state;
        let app_control = update_args.app_control;

        let now = time_state.current_instant();

        //
        // Quit if user hits escape
        //
        if input_state.is_key_down(VirtualKeyCode::Escape) {
            app_control.enqueue_terminate_process();
        }

        //
        // Update FPS once a second
        //
        let update_text_string = match self.last_fps_text_change {
            Some(last_update_instant) => (now - last_update_instant).as_secs_f32() >= 1.0,
            None => true,
        };

        // Refresh FPS text
        if update_text_string {
            let fps = time_state.updates_per_second();
            self.fps_text = format!("Fps: {:.1}", fps);
            self.last_fps_text_change = Some(now);
        }
    }

    fn draw(
        &mut self,
        draw_args: AppDrawArgs,
    ) {
        let canvas = draw_args.canvas;

        // Draw an inspect window for the example struct
        {
            let imgui_manager = draw_args.imgui_manager;
            imgui_manager.with_ui(|ui: &mut imgui::Ui| {
                imgui::Window::new(imgui::im_str!("Inspect Demo"))
                    .position([550.0, 100.0], imgui::Condition::Once)
                    .size([300.0, 400.0], imgui::Condition::Once)
                    .build(ui, || {

                        // Add read-only widgets. We pass a slice of refs. Using a slice means we
                        // can implement multiple selection
                        let selected = vec![&self.example_inspect_target];
                        <ExampleInspectTarget as imgui_inspect::InspectRenderStruct::<ExampleInspectTarget>>::render(
                            &selected,
                            "Example Struct - Read Only",
                            ui,
                            &InspectArgsStruct::default());

                        // Now add writable UI widgets. This again takes a slice to handle multiple
                        // selection
                        let mut selected_mut = vec![&mut self.example_inspect_target];
                        <ExampleInspectTarget as imgui_inspect::InspectRenderStruct::<ExampleInspectTarget>>::render_mut(
                            &mut selected_mut,
                            "Example Struct - Writable",
                            ui,
                            &InspectArgsStruct::default());
                });
            });
        }

        // Generally would want to clear data every time we draw
        canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        // Make a color to draw with
        let mut paint = skia_safe::Paint::new(self.example_inspect_target.color.0.clone(), None);
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::paint::Style::StrokeAndFill);
        paint.set_stroke_width(1.0);

        //
        // Draw the circle that the user can manipulate
        //
        canvas.draw_circle(
            skia_safe::Point::new(self.example_inspect_target.x_position, self.example_inspect_target.y_position),
            self.example_inspect_target.radius,
            &paint,
        );

        //
        // Draw FPS text
        //
        let mut text_paint =
            skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 1.0, 0.0, 1.0), None);
        text_paint.set_anti_alias(true);
        text_paint.set_style(skia_safe::paint::Style::StrokeAndFill);
        text_paint.set_stroke_width(1.0);

        //
        // Draw user's custom string
        //
        let mut font = skia_safe::Font::default();
        font.set_size(20.0);
        canvas.draw_str(self.fps_text.clone(), (50, 50), &font, &text_paint);
        canvas.draw_str(imgui::im_str!("{}", self.example_inspect_target.text), (50, 100), &font, &text_paint);
    }

    fn fatal_error(
        &mut self,
        error: &skulpin::AppError,
    ) {
        println!("{}", error);
    }
}