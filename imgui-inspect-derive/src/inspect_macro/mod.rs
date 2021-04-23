use darling::FromDeriveInput;
use syn::{parse_macro_input, Data, DeriveInput};
use quote::quote;

mod args;
use args::{InspectArgsDefault, InspectStructArgs};

mod gen_struct;

pub struct ParsedField {
    render: proc_macro2::TokenStream,
    render_mut: proc_macro2::TokenStream,
    //skip: bool
}

pub fn impl_inspect_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_args = InspectStructArgs::from_derive_input(&input)
        .expect("Unable to create InspectStructArgs from token stream");

    match input.data {
        Data::Struct(ref data) => {
            let field_args = gen_struct::parse_field_args(data);
            gen_struct::generate(&input, struct_args, field_args)
        }
        Data::Enum(ref data) => {
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

            let type_name = &input.ident;

            let combo_items = data
                .variants
                .iter()
                .map(|v| {
                    assert!(v.fields.is_empty(), "Only plain enum is supported (now)");
                    let variant_name = &v.ident;
                    quote! {
                        #variant_name
                    }
                })
                .collect::<Vec<_>>();

            proc_macro::TokenStream::from(quote! {
                impl #impl_generics imgui_inspect::InspectRenderDefault<#type_name> for #type_name #ty_generics #where_clause {
                    fn render(
                        data: &[&Self],
                        _label: &'static str,
                        ui: &imgui::Ui,
                        _args: &InspectArgsDefault,
                    ) {
                        let items = [#(#combo_items,)*];
                        ComboBox::new(imgui::im_str!("{}", #type_name)).build_simple_string(ui, &items);
                    }
                    fn render_mut(
                        data: &[&Self],
                        _label: &'static str,
                        ui: &imgui::Ui,
                        _args: &InspectArgsDefault,
                    ) {
                        let items = [#(#combo_items,)*];
                        ComboBox::new(imgui::im_str!("{}", #type_name)).build_simple_string(ui, &items);
                    }
                }
            })
        }
        Data::Union(ref _data) => {
            unimplemented!("union is not supported");
        }
    }
}
