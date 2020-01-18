# imgui_inspect

Oversimplified, this is an implementation of a property editor using [`ImGui`](https://github.com/ocornut/imgui) and [`Rust`](https://rust-lang.org).

More specifically, this crate aims to be a `serde` for inspecting with imgui. It defines a common interface for getting values into imgui and back out.

There are some default implementations for certain types to be drawn as certain widgets, but the primary goal is to make it easy to implement your own preferred way of rendering values.
* There is a trait for each widget type (i.e. `InspectRenderSlider`)
* There is an impl for each value type (i.e. `f32`)

[![Build Status](https://travis-ci.org/aclysma/imgui-inspect.svg?branch=master)](https://travis-ci.org/aclysma/imgui-inspect)
![Crates.io](https://img.shields.io/crates/v/imgui-inspect)

## Usage

For default rendering behavior, derive Inspect on your struct

```rust
#[derive(Inspect)]
pub struct MyStruct {
    pub first_value: f32,
    pub second_value: f32,
}
```

To draw, Call it with the UI window and a reference to an instance of your struct:

```rust
    // ....
    ui.text(im_str!("This...is...imgui!"));
    ui.separator();
    let my_struct = MyStruct::default(); // example, maybe get it from somewhere else instead
    <MyStruct as InspectRenderDefault<MyStruct>>::render(
        &[&my_struct], 
        &"my_struct_test", 
        ui, 
        &InspectArgsDefault::default()
    );
```

The reason my_struct is being passed as &\[&my_struct\] is because it's common to select multiple objects. In this case,
the rendering code could compare if the values are consistent across all selected items, or in the case of rendering
mutably, apply the change to all selected values.

### Simple Customization

You can get slightly different behavior by marking up the struct members. This can choose what widgets to draw and tweak their settings.

```rust
#[derive(Inspect)]
pub struct MyStruct {    
    // Use a slider widget with the given min/max values
    #[inspect_slider(min_value = 5.0, max_value = 53.0)]
    pub sliding_value: f32,
}
```

### Advanced Customization

Internally, deriving Inspect implements `InspectRenderDefault` for MyStruct. But you can implement it manually if you need to do something custom.

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

You can either wrap the type yourself (i.e. `struct MyVec2(glm::Vec2)`) or you can use a dummy type and override the `proxy_type`.

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
    #[inspect(proxy_type = "ImGlmVec2")]
    pub position: glm::Vec2,
}
```

This type is never instantiated. It's just used to resolve the function that should be called: `<ImGlmVec2 as InspectRenderDefault>::render(...)`

## Adding a default widget implementation for a value type

**Remember you can always use a proxy type if you don't want to upstream changes, or if you dislike the default implementation!**

This is easy to do and only requires changing `imgui-inspect`

* Many examples are located in [imgui-inspect/src](tree/master/imgui-inspect/src)
* Add a new module `[WIDGET]/[WIDGET]_[TYPE]`
    * EXAMPLE: `slider/slider_f32.rs`
* Add an implementation: `impl InspectRender[WIDGET]<[TYPE]> for [TYPE]`
    * EXAMPLE: `impl InspectRenderSlider<f32> for f32`

## Adding a new widget type

In summary, we need to add a trait for the widget, implement the trait for some types, and add widget/config options to the proc macro

* Add the trait
    * Add `InspectArgs[WIDGET]` and `InspectRender[WIDGET]` to `imgui-inspect/src/[WIDGET]/mod.rs`
    * EXAMPLE: `InspectArgsSlider` and `InspectRenderSlider` in `imgui-inspect/src/slider`
    * The values in `InspectArgs[WIDGET]` are the options that can be fed into imgui
    * The implementations for `InspectArgs[WIDGET]` should pipe these values into the imgui widget
* Implement the trait for types by following the above instructions "Adding a default widget implementation for a value type"
* Add the widget to the proc macro
    * Add `InspectFieldArgs[WIDGET]` and `InspectArgs[WIDGET]` to `imgui-inspect-derive/src/inspect_macro/args/[WIDGET]_args.rs`
        * EXAMPLE: `InspectFieldArgsSlider` and `InspectFieldArgsSlider` in `slider_args.rs`
        * `InspectArgs[WIDGET]` is a copy-paste of the same struct in inspect-imgui. (It's duplicated because a proc_macro crate cannot export a type)
        * `InspectArgs[WIDGET]` are the values unique to the widget, and `InspectFieldArgs[WIDGET]` is every single property that can be changed via the macro
    * This structs `InspectArgsDefault` and `InspectFieldArgsDefault` should be a superset of all args possible for any widget. Update:
        * `imgui-inspect-derive/src/inspect_macro/args/default_args.rs`
        * `imgui-inspect/src/default/mod.rs`
    * Update `handle_inspect_types()` in  `imgui-inspect-derive/src/inspect_macro/mod.rs`
    * Add widget type to proc_macro_derive in `imgui-inspect-derive/src/lib.rs`

## Making imgui Optional

Generally, you don't want to ship imgui in your end product. However, conditionally including the proc_macro is awkward.
Rust will complain about any macros that it isn't told about, so removing the imgui-inspect-derive dependency from a
project conditionally requires noisy markup.

To solve this, the imgui-inspect-derive macro uses a feature "generate_code." Disabling
default features will prevent code from being generated.

Steps:
 * make imgui-inspect optional
     * Example: `imgui-inspect = { version = "...", optional = true }`
 * Turn off imgui-inspect-derive default features
     * Example: `imgui-inspect-derive = { version = "...", default-features = false }`
 
imgui-inspect-derive generates boilerplate code but doesn't actually depend on imgui. Disabling 
default features means the generate_code feature will be disabled, causing the
macros to be parsed, but no code to be emitted.

## Example

A simple example is located in imgui-inspect-demo

You could also refer to [this project](https://github.com/aclysma/minimum/tree/master/minimum-framework/src/inspect). 
The property editor in [this video](https://www.youtube.com/watch?v=BON_RvVFiWY&t=30s) shows this library being used within it.

Many of the more complex use-cases, such as custom types and handling multiple selected values, are implemented there.

## Status

The overall design of this crate is unlikely to change, but very few imgui widget types and value types are implemented.

Most of the future work will be:
* Add traits for each imgui widget type
* Define structs to represent valid options for that imgui widget 
* Implement sensible defaults for std types

It's a fairly straightforward process and basic examples exist, but it does take time to add them.

I'll be extending this as I need support for more types, but if you need something that's missing, PR it! There are detailed 
instructions below.

## Contribution

All contributions are assumed to be dual-licensed under MIT/Apache-2.

## License

Distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
