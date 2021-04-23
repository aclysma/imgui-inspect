use imgui_inspect_derive::Inspect;

use imgui_inspect::InspectArgsStruct;

mod color;
use color::Color;

mod renderer;
use renderer::Renderer;

mod imgui_support;
use imgui_support::ImguiManager;
use crate::color::Color4f;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestEnum {
    A,
    B,
    C,
}

// This struct is a simple example of something that can be inspected
#[derive(Inspect)]
pub struct ExampleInspectTarget {
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
                a: 1.0,
            }),
            text: "".to_string(),
        }
    }
}

fn draw_imgui(
    imgui_manager: &ImguiManager,
    example_inspect_target: &mut ExampleInspectTarget,
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
                    let selected = vec![&*example_inspect_target];
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
                    let mut selected_mut = vec![example_inspect_target];
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
    let renderer = Renderer::new(&window, imgui_manager.font_atlas_texture());

    // Check if there were errors setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    // This is the thing we will inspect
    let mut example_inspect_target = ExampleInspectTarget::default();

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
                // Queue a RedrawRequested event.
                window.request_redraw();
            }

            //
            // Redraw
            //
            winit::event::Event::RedrawRequested(_window_id) => {
                imgui_manager.begin_frame(&window);
                draw_imgui(&imgui_manager, &mut example_inspect_target);
                imgui_manager.render(&window);
                if let Err(e) =
                    renderer.draw(&window, imgui_manager.draw_data(), &example_inspect_target)
                {
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
