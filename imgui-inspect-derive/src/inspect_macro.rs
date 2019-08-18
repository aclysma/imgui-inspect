
use syn::{parse_macro_input, Data, DeriveInput, Fields};
use darling::{FromField, FromDeriveInput};
use quote::quote;
use syn::export::ToTokens;

// Utility function to convert an Option<T> to tokens
fn expand_to_tokens<T : quote::ToTokens>(input: &Option<T>) -> proc_macro2::TokenStream {
    match input {
        Some(value) => quote!(Some(#value)),
        None => quote!(None)
    }
}

// Metadata from the struct's type annotation
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inspect))]
struct InspectStructArgs {
    ident: syn::Ident
}

// We support multiple distinct inspect annotations (i.e. inspect_slider, inspect_text)
// Each distinct type will have a struct for capturing the metadata. These metadata structs
// must implement this trait
trait InspectFieldArgs {
    fn ident(&self) -> &Option<syn::Ident>;
    fn ty(&self) -> &syn::Type;
    fn render_trait(&self) -> &Option<syn::Path>;
    fn wrapping_type(&self) -> &Option<syn::Path>;
}

#[derive(Debug, FromField, Clone)]
#[darling(attributes(inspect))]
struct InspectFieldArgsDefault {

    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    render_trait: Option<syn::Path>,

    #[darling(default)]
    wrapping_type: Option<syn::Path>,

    #[darling(default)]
    skip: bool,

    #[darling(default)]
    min_value: Option<f32>,

    #[darling(default)]
    max_value: Option<f32>,

    #[darling(default)]
    step: Option<f32>,
}

impl InspectFieldArgs for InspectFieldArgsDefault {
    fn ident(&self) -> &Option<syn::Ident> { &self.ident }
    fn ty(&self) -> &syn::Type { &self.ty }
    fn render_trait(&self) -> &Option<syn::Path> { &self.render_trait }
    fn wrapping_type(&self) -> &Option<syn::Path> { &self.wrapping_type }
}

impl quote::ToTokens for InspectArgsDefault {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let min_value = expand_to_tokens(&self.min_value);
        let max_value = expand_to_tokens(&self.max_value);
        let step = expand_to_tokens(&self.step);

        use quote::TokenStreamExt;
        tokens.append_all(quote!(
            InspectArgsDefault {
                min_value: #min_value,
                max_value: #max_value,
                step: #step,
            }
        ));
    }
}

#[derive(Debug)]
struct InspectArgsDefault {
    min_value: Option<f32>,
    max_value: Option<f32>,
    step: Option<f32>,
}

impl From<InspectFieldArgsDefault> for InspectArgsDefault {
    fn from(field_args: InspectFieldArgsDefault) -> Self {
        Self {
            min_value: field_args.min_value,
            max_value: field_args.max_value,
            step: field_args.step,
        }
    }
}

#[derive(Debug, FromField, Clone)]
#[darling(attributes(inspect_slider))]
struct InspectFieldArgsSlider {

    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    render_trait: Option<syn::Path>,

    #[darling(default)]
    wrapping_type: Option<syn::Path>,

    #[darling(default)]
    min_value: Option<f32>,

    #[darling(default)]
    max_value: Option<f32>,
}

impl InspectFieldArgs for InspectFieldArgsSlider {
    fn ident(&self) -> &Option<syn::Ident> { &self.ident }
    fn ty(&self) -> &syn::Type { &self.ty }
    fn render_trait(&self) -> &Option<syn::Path> { &self.render_trait }
    fn wrapping_type(&self) -> &Option<syn::Path> { &self.wrapping_type }
}

impl quote::ToTokens for InspectArgsSlider {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let min_value = expand_to_tokens(&self.min_value);
        let max_value = expand_to_tokens(&self.max_value);

        use quote::TokenStreamExt;
        tokens.append_all(quote!(
            InspectArgsSlider {
                min_value: #min_value,
                max_value: #max_value,
            }
        ));
    }
}

#[derive(Debug)]
struct InspectArgsSlider {
    min_value: Option<f32>,
    max_value: Option<f32>
}

impl From<InspectFieldArgsSlider> for InspectArgsSlider {
    fn from(field_args: InspectFieldArgsSlider) -> Self {
        Self {
            min_value: field_args.min_value,
            max_value: field_args.max_value,
        }
    }
}

pub fn impl_inspect_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_args = InspectStructArgs::from_derive_input(&input).unwrap();
    let field_args = parse_field_args(&input);
    generate(&input, struct_args, field_args)
}

struct ParsedField {
    render: proc_macro2::TokenStream,
    render_mut: proc_macro2::TokenStream
}

fn parse_field_args(input: &syn::DeriveInput) -> Vec<ParsedField> {

    // Effectively, these are constants. We support only one of these on a member at a time
    #[allow(non_snake_case)]
    let INSPECT_DEFAULT_PATH = syn::parse2::<syn::Path>(quote!(inspect)).unwrap();
    #[allow(non_snake_case)]
    let INSPECT_SLIDER_PATH = syn::parse2::<syn::Path>(quote!(inspect_slider)).unwrap();

    match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Parse the fields
                    let parsed_fields : Vec<_> = fields.named.iter().map(|f| {

                        let mut parsed_field : Option<ParsedField> = None;

                        try_handle_inspect_type::<InspectFieldArgsSlider, InspectArgsSlider>(&mut parsed_field, &f, &INSPECT_SLIDER_PATH, quote!(InspectRenderSlider), quote!(InspectArgsSlider));
                        try_handle_inspect_type::<InspectFieldArgsDefault, InspectArgsDefault>(&mut parsed_field, &f, &INSPECT_DEFAULT_PATH, quote!(InspectRenderDefault), quote!(InspectArgsDefault));

                        if parsed_field.is_none() {
                            handle_inspect_type::<InspectFieldArgsDefault, InspectArgsDefault>(&mut parsed_field, &f, quote!(InspectRenderDefault), quote!(InspectArgsDefault));
                        }

                        parsed_field.unwrap()

                    }).collect();

                    parsed_fields
                }
                //Fields::Unit => ,
                _ => unimplemented!(),
            }
        },
        _ => unimplemented!(),
    }
}

fn try_handle_inspect_type<FieldArgsT : darling::FromField + InspectFieldArgs + Clone, ArgsT : From<FieldArgsT> + ToTokens>(
    parsed_field: &mut Option<ParsedField>,
    f: &syn::Field,
    path: &syn::Path,
    default_render_trait: proc_macro2::TokenStream,
    arg_type: proc_macro2::TokenStream
) {
    if f.attrs.iter().find(|x| x.path == *path).is_some() {
        handle_inspect_type::<FieldArgsT, ArgsT>(parsed_field, &f, default_render_trait, arg_type);
    }
}

// Does common data gathering and error checking, then calls create_render_call and create_render_mut_call to emit
// code for inspecting.
fn handle_inspect_type<FieldArgsT : darling::FromField + InspectFieldArgs + Clone, ArgsT : From<FieldArgsT> + ToTokens>(
    parsed_field: &mut Option<ParsedField>,
    f: &syn::Field,
    default_render_trait: proc_macro2::TokenStream,
    arg_type: proc_macro2::TokenStream
) {
    //TODO: Improve error message
    if parsed_field.is_some() {
        panic!("Too many inspect attributes on a single member {:?}", f.ident);
    }

    let field_args = FieldArgsT::from_field(&f).unwrap();

    let render_trait = match field_args.render_trait() {
        Some(t) => t.clone(),
        None => syn::parse2::<syn::Path>(default_render_trait).unwrap()
    };

    let arg_type = syn::parse2::<syn::Type>(arg_type).unwrap();
    let args : ArgsT = field_args.clone().into();

    let render = create_render_call(
        field_args.ident().as_ref().unwrap(),
        field_args.ty(),
        &render_trait,
        field_args.wrapping_type(),
        &arg_type,
        &args);

    let render_mut = create_render_mut_call(
        field_args.ident().as_ref().unwrap(),
        field_args.ty(),
        &render_trait,
        field_args.wrapping_type(),
        &arg_type,
        &args);

    *parsed_field = Some(ParsedField {
        render,
        render_mut
    });
}

fn create_render_call<T : ToTokens>(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    render_trait: &syn::Path,
    wrapping_type: &Option<syn::Path>,
    arg_type: &syn::Type,
    args: &T,
) -> proc_macro2::TokenStream {

    use quote::format_ident;
    let args_name1 = format_ident!("_inspect_args_{}", field_name);
    let args_name2 = args_name1.clone();

    let field_name1 = field_name.clone();
    let field_name2 = field_name.clone();

    let source_type = if let Some(w) = wrapping_type {
        quote!(#w)
    } else {
        quote!(#field_type)
    };

    quote! {{
        #[allow(non_upper_case_globals)]
        const #args_name1 : #arg_type = #args;
        <#source_type as #render_trait<#field_type>>::render(&[&data[0].#field_name1], stringify!(#field_name2), ui, &#args_name2);
    }}
}

fn create_render_mut_call<T : ToTokens>(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    render_trait: &syn::Path,
    wrapping_type: &Option<syn::Path>,
    arg_type: &syn::Type,
    args: &T,
) -> proc_macro2::TokenStream {

    use quote::format_ident;
    let args_name1 = format_ident!("_inspect_args_{}", field_name);
    let args_name2 = args_name1.clone();

    let field_name2 = field_name.clone();
    let field_name3 = field_name.clone();

    let source_type = if let Some(w) = wrapping_type {
        quote!(#w)
    } else {
        quote!(#field_type)
    };

    quote! {{
        #[allow(non_upper_case_globals)]
        const #args_name1 : #arg_type = #args;
        let mut values : Vec<_> = data.iter_mut().map(|x| &mut x.#field_name3).collect();
        <#source_type as #render_trait<#field_type>>::render_mut(&mut values.as_mut_slice(), stringify!(#field_name2), ui, &#args_name2);
    }}
}

fn generate(input: &syn::DeriveInput, struct_args: InspectStructArgs, parsed_fields: Vec<ParsedField>) -> proc_macro::TokenStream {

    let struct_name1 = &struct_args.ident;
    let struct_name2 = &struct_args.ident;
    let struct_name3 = &struct_args.ident;
    let struct_name4 = &struct_args.ident;

    let mut render_impls = vec![];
    let mut render_mut_impls = vec![];

    for parsed_field in parsed_fields {
        render_impls.push(parsed_field.render);
        render_mut_impls.push(parsed_field.render_mut);
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    proc_macro::TokenStream::from(quote! {

        impl #impl_generics InspectRenderDefault<#struct_name1> for #struct_name2 #ty_generics #where_clause {
            fn render(data: &[&Self], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
                let header_name = stringify!(#struct_name3);
                let header = ui.collapsing_header(&imgui::im_str!( "{}", header_name)).build();
                if header {
                    ui.push_id(label);
                    ui.indent();
                    #(
                        #render_impls
                    )*
                    ui.unindent();
                    ui.pop_id();
                }
            }

            fn render_mut(data: &mut [&mut Self], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
                let header_name = stringify!(#struct_name4);
                let header = ui.collapsing_header(&imgui::im_str!("{}", header_name)).build();
                if header {
                    ui.push_id(label);
                    ui.indent();
                    #(
                        #render_mut_impls
                    )*
                    ui.unindent();
                    ui.pop_id();
                }
            }
        }
    })
}
