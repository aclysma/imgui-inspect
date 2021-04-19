use darling::FromDeriveInput;
use syn::{parse_macro_input, Data, DeriveInput};

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
            for _variant in &data.variants {
                //
            }
            todo!()
        }
        Data::Union(ref _data) => {
            unimplemented!("union is not supported");
        }
    }
}
