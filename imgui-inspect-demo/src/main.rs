use imgui_inspect_derive::Inspect;

use imgui_inspect::InspectArgsStruct;

mod color;
use color::Color;

mod renderer;
use renderer::Renderer;

mod time_state;
use time_state::TimeState;

mod imgui_support;
use imgui_support::ImguiManager;
use rafx::api::RafxExtents2D;
use crate::color::Color4f;

// This struct is a simple example of something that can be inspected
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
    text: String,
}

impl Default for ExampleInspectTarget {
    fn default() -> Self {
        ExampleInspectTarget {
            x_position: 300.0,
            y_position: 250.0,
            radius: 50.0,
            color: Color(Color4f {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0
            }),
            text: "".to_string(),
        }
    }
}

// This encapsulates the demo logic
struct ExampleApp {
    last_fps_text_change: Option<std::time::Instant>,
    fps_text: String,
    example_inspect_target: ExampleInspectTarget,
}

impl ExampleApp {
    pub fn new() -> Self {
        ExampleApp {
            last_fps_text_change: None,
            fps_text: "".to_string(),
            example_inspect_target: Default::default(),
        }
    }

    fn update(
        &mut self,
        time_state: &TimeState,
    ) {
        let now = time_state.current_instant();

        //
        // Update FPS once a second
        //
        let update_text_string = match self.last_fps_text_change {
            Some(last_update_instant) => (now - last_update_instant).as_secs_f32() >= 1.0,
            None => true,
        };

        //
        // Refresh FPS text
        //
        if update_text_string {
            let fps = time_state.updates_per_second();
            self.fps_text = format!("Fps: {:.1}", fps);
            self.last_fps_text_change = Some(now);
        }
    }

    fn draw(
        &mut self,
        imgui_manager: &ImguiManager,
    ) {
        //
        //Draw an inspect window for the example struct
        //
        {
            imgui_manager.with_ui(|ui: &mut imgui::Ui| {
                imgui::Window::new(imgui::im_str!("Inspect Demo"))
                    .position([550.0, 100.0], imgui::Condition::Once)
                    .size([300.0, 400.0], imgui::Condition::Once)
                    .build(ui, || {
                        // Add read-only widgets. We pass a slice of refs. Using a slice means we
                        // can implement multiple selection
                        let selected = vec![&self.example_inspect_target];
                        <ExampleInspectTarget as imgui_inspect::InspectRenderStruct<
                            ExampleInspectTarget,
                        >>::render(
                            &selected,
                            "Example Struct - Read Only",
                            ui,
                            &InspectArgsStruct::default(),
                        );

                        // Now add writable UI widgets. This again takes a slice to handle multiple
                        // selection
                        let mut selected_mut = vec![&mut self.example_inspect_target];
                        <ExampleInspectTarget as imgui_inspect::InspectRenderStruct<
                            ExampleInspectTarget,
                        >>::render_mut(
                            &mut selected_mut,
                            "Example Struct - Writable",
                            ui,
                            &InspectArgsStruct::default(),
                        );
                    });
            });
        }

        // //
        // // Generally would want to clear data every time we draw
        // //
        // canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));
        //
        // //
        // // Make a color to draw with
        // //
        // let mut paint = skia_safe::Paint::new(self.example_inspect_target.color.0.clone(), None);
        // paint.set_anti_alias(true);
        // paint.set_style(skia_safe::paint::Style::StrokeAndFill);
        // paint.set_stroke_width(1.0);
        //
        // //
        // // Draw the circle that the user can manipulate
        // //
        // canvas.draw_circle(
        //     skia_safe::Point::new(
        //         self.example_inspect_target.x_position,
        //         self.example_inspect_target.y_position,
        //     ),
        //     self.example_inspect_target.radius,
        //     &paint,
        // );
        //
        // //
        // // Draw FPS text
        // //
        // let mut text_paint =
        //     skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 1.0, 0.0, 1.0), None);
        // text_paint.set_anti_alias(true);
        // text_paint.set_style(skia_safe::paint::Style::StrokeAndFill);
        // text_paint.set_stroke_width(1.0);
        //
        // //
        // // Draw user's custom string
        // //
        // let mut font = skia_safe::Font::default();
        // font.set_size(20.0);
        // canvas.draw_str(self.fps_text.clone(), (50, 50), &font, &text_paint);
        // canvas.draw_str(
        //     imgui::im_str!("{}", self.example_inspect_target.text),
        //     (50, 100),
        //     &font,
        //     &text_paint,
        // );
    }
}

// Creates a window and runs the event loop.
fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = winit::dpi::LogicalSize::new(900.0, 600.0);

    // Create a single window
    let window = winit::window::WindowBuilder::new()
        .with_title("Skulpin")
        .with_inner_size(logical_size)
        .build(&event_loop)
        .expect("Failed to create window");

    // Initialize imgui
    let imgui_manager = imgui_support::init_imgui_manager(&window);

    // Create the renderer, which will draw to the window
    let renderer = Renderer::new(&window);

    // Check if there were errors setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    let mut app = ExampleApp::new();
    let mut time_state = TimeState::new();

    // Start the window event loop. Winit will not return once run is called. We will get notified
    // when important events happen.
    event_loop.run(move |event, _window_target, control_flow| {
        imgui_manager.handle_event(&window, &event);

        match event {
            //
            // Halt if the user requests to close the window
            //
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,

            //
            // Close if the escape key is hit
            //
            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,

            //
            // Request a redraw any time we finish processing events
            //
            winit::event::Event::MainEventsCleared => {
                time_state.update();

                app.update(&time_state);

                // Queue a RedrawRequested event.
                window.request_redraw();
            }

            //
            // Redraw
            //
            winit::event::Event::RedrawRequested(_window_id) => {
                if let Err(e) = renderer.draw(&window, |command_buffer| {
                    imgui_manager.begin_frame(&window);

                    //app.draw(canvas, &coordinate_system_helper, &imgui_manager);

                    imgui_manager.render(&window);
                }) {
                    println!("Error during draw: {:?}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            }

            //
            // Ignore all other events
            //
            _ => {}
        }
    });
}
