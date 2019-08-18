# imgui_inspect

An implementation of a property editor using [`ImGui`](https://github.com/ocornut/imgui) and [`Rust`](https://rust-lang.org).

## Status

For now this is a demo/prototype, but I think it could become useful! I'll be extending it as I start using it more. If something is missing that you want, PR it!

## Usage

The intent is that you can get an imgui property editor with minimal code changes

```rust
#[derive(Inspect)]
pub struct MyStruct {
    pub first_value: f32,
    pub second_value: f32,
}
```
You can get slightly different behavior by marking up the struct members


```rust
#[derive(Inspect)]
pub struct MyStruct {    
    // Use a slider widget with the given min/max values
    #[inspect_slider(min_value = 5.0, max_value = 53.0)]
    pub sliding_value: f32,
}
```

Internally, this is implementing `InspectRenderDefault` for MyStruct. But you can implement it manually if you need to do something custom.

```rust
impl InspectRenderDefault<MyStruct> for MyStruct {
    fn render(data: &[&MyStruct], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        ui.text("custom rendering is easy!");
    }
    fn render_mut(data: &mut [&mut MyStruct], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        ui.text("custom rendering is easy!");
    }
}
```
![screenshot][logo]

[logo]: imgui_inspect.png "Screenshot"

The API allows mixing and matching Render traits and types that are being rendered. A single type can be renderered multiple ways by overriding the `render_trait`

```rust
pub trait InspectRenderMyCustomWidgetType<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault);
    fn render_mut(data: &mut [&mut T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault);
}

#[derive(Inspect)]
pub struct MyStruct {
    #[inspect(render_trait = "InspectRenderMyCustomWidgetType")]
    pub a_value: f32,
}
```

This will call `<f32 as InspectRenderMyCustomWidgetType>::render(...)`. This is useful for rendering the same type a few different ways.

Sometimes you want to implement your own rendering for a type in some other crate. But there's a problem.. traits can only
be implemented in the same crate as the trait or the type itself.

You can either wrap the type yourself (i.e. `struct MyVec2(glm::Vec2)`) or you can use a dummy type and override the `wrapping_type`.

```rust
struct ImGlmVec2;
impl InspectRenderDefault<glm::Vec2> for ImGlmVec2 {
    fn render(data: &[&glm::Vec2], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        // ...
    }

    fn render_mut(data: &mut [&mut glm::Vec2], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        // ...
    }
}

#[derive(Inspect, Clone)]
pub struct MyStruct {
    #[inspect(wrapping_type = "ImGlmVec2")]
    pub position: glm::Vec2,
}
```

This type is never instantiated. It's just used to resolve the function that should be called: `<ImGlmVec2 as InspectRenderDefault>::render(...)`

## Contribution

All contributions are assumed to be dual-licensed under MIT/Apache-2.

## License

Distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
