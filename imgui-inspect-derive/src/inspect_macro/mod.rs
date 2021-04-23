use darling::FromDeriveInput;
use syn::{parse_macro_input, Data, DeriveInput};
use quote::quote;

mod args;
use args::InspectStructArgs;

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
            let type_name = {
                let type_name = &input.ident;
                stringify!(#type_name);
            };

            let combo = quote! {
                ComboBox::new(imgui::im_str!("{}", type_name))
            };

            let combo_items: Vec<proc_macro::TokenStream> = data.variants.iter().map(|v| {
                assert!(v.fields.is_empty(), "Only plain enum is supported (now)");
                let variant_name = &v.ident;
                quote! {
                    #variant_name
                }
            });

            proc_macro::TokenStream::from(quote! {
                let combo = #combo;
                #(#combo_items)*
                combo.build(ui);
            })
        }
        Data::Union(ref _data) => {
            unimplemented!("union is not supported");
        }
    }
}
