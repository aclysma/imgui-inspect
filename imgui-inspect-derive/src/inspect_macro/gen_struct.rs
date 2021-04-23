use quote::{quote, ToTokens};
use syn::{DataStruct, Fields};

use crate::inspect_macro::{args::*, ParsedField};

pub fn parse_field_args(data: &DataStruct) -> Vec<ParsedField> {
    match data.fields {
        Fields::Named(ref fields) => fields
            .named
            .iter()
            .map(|f| handle_inspect_types(f).unwrap())
            .collect(),
        Fields::Unnamed(ref _fields) => {
            unimplemented!("#[derive(Inspect)] is only allowed on structs with named fields.")
        }
        Fields::Unit => vec![],
    }
}

// Provide a way to early out and generate no code. It's going to be a common case for
// downstream users to want to only conditionally generate code, and it's easier to do this
// by adding an early-out here that can be configured via a cargo feature, than having to
// mark up all the downstream code with conditional compile directives.
#[cfg(not(feature = "generate_code"))]
pub fn generate(
    input: &syn::DeriveInput,
    struct_args: InspectStructArgs,
    parsed_fields: Vec<ParsedField>,
) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! {})
}

#[cfg(feature = "generate_code")]
pub fn generate(
    input: &syn::DeriveInput,
    struct_args: InspectStructArgs,
    parsed_fields: Vec<ParsedField>,
) -> proc_macro::TokenStream {
    let struct_name = &struct_args.ident;

    let (render_impls, render_mut_impls): (Vec<_>, Vec<_>) = parsed_fields
        .into_iter()
        .map(|f| (f.render, f.render_mut))
        .unzip();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let default_impl = quote! {
        impl #impl_generics imgui_inspect::InspectRenderDefault<#struct_name>
            for #struct_name #ty_generics #where_clause
        {
            fn render(
                data: &[&Self],
                label: &'static str,
                ui: &imgui::Ui,
                args: &imgui_inspect::InspectArgsDefault,
            ) {
                <#struct_name as imgui_inspect::InspectRenderStruct<#struct_name>>::render(
                    data,
                    label,
                    ui,
                    &imgui_inspect::InspectArgsStruct {
                        header: args.header,
                        indent_children: args.indent_children,
                    },
                )
            }

            fn render_mut(
                data: &mut [&mut Self],
                label: &'static str,
                ui: &imgui::Ui,
                args: &imgui_inspect::InspectArgsDefault,
            ) -> bool {
                <#struct_name as imgui_inspect::InspectRenderStruct<#struct_name>>::render_mut(
                    data,
                    label,
                    ui,
                    &imgui_inspect::InspectArgsStruct {
                        header: args.header,
                        indent_children: args.indent_children,
                    },
                )
            }
        }
    };

    let struct_impl = quote! {
        impl #impl_generics imgui_inspect::InspectRenderStruct<#struct_name> for #struct_name #ty_generics #where_clause {
             fn render(
                 data: &[&Self],
                 label: &'static str,
                 ui: &imgui::Ui,
                 args: &imgui_inspect::InspectArgsStruct,
             ) {
                let header_name = stringify!(#struct_name);

                let header = args.header.unwrap_or(false);
                let indent_children = args.indent_children.unwrap_or(false);

                if header {
                    if !imgui::CollapsingHeader::new(&imgui::im_str!("{}", header_name))
                        .default_open(true)
                        .build(&ui)
                    {
                        return;
                    }
                }

                let id_token = ui.push_id(label);

                if indent_children { ui.indent(); }
                #(#render_impls;)*
                if indent_children { ui.unindent(); }

                id_token.pop(ui);
            }

            fn render_mut(
                data: &mut [&mut Self],
                label: &'static str,
                ui: &imgui::Ui,
                args: &imgui_inspect::InspectArgsStruct,
            ) -> bool {
                let header_name = stringify!(#struct_name);

                let header = args.header.unwrap_or(false);
                let indent_children = args.indent_children.unwrap_or(false);

                if header {
                    if !imgui::CollapsingHeader::new(&imgui::im_str!("{}", header_name))
                        .default_open(true)
                        .build(&ui)
                    {
                        return false;
                    }
                }

                let id_token = ui.push_id(label);

                if indent_children { ui.indent(); }
                let mut any_field_changed = false;
                #(any_field_changed |= #render_mut_impls ;)*
                if indent_children { ui.unindent(); }

                id_token.pop(ui);

                any_field_changed
            }
        }
    };

    proc_macro::TokenStream::from(quote! {
        #default_impl
        #struct_impl
    })
}

/// Named parameters for creating render methods
struct RenderCall<'a, T: ToTokens> {
    field_name: &'a syn::Ident,
    field_type: &'a syn::Type,
    render_trait: &'a syn::Path,
    proxy_type: &'a Option<syn::Path>,
    arg_type: &'a syn::Type,
    args: &'a T,
}

fn handle_inspect_types(f: &syn::Field) -> Option<ParsedField> {
    let mut parsed_field: Option<ParsedField> = None;

    let inspect_default_path = syn::parse2::<syn::Path>(quote!(inspect)).unwrap();
    let inspect_slider_path = syn::parse2::<syn::Path>(quote!(inspect_slider)).unwrap();

    // We must check every trait
    try_handle_inspect_type::<InspectFieldArgsSlider, InspectArgsSlider>(
        &mut parsed_field,
        f,
        &inspect_slider_path,
        quote!(imgui_inspect::InspectRenderSlider),
        quote!(imgui_inspect::InspectArgsSlider),
    );

    try_handle_inspect_type::<InspectFieldArgsDefault, InspectArgsDefault>(
        &mut parsed_field,
        f,
        &inspect_default_path,
        quote!(imgui_inspect::InspectRenderDefault),
        quote!(imgui_inspect::InspectArgsDefault),
    );

    if parsed_field.is_none() {
        handle_inspect_type::<InspectFieldArgsDefault, InspectArgsDefault>(
            &mut parsed_field,
            &f,
            quote!(imgui_inspect::InspectRenderDefault),
            quote!(imgui_inspect::InspectArgsDefault),
        );
    }

    parsed_field
}

fn try_handle_inspect_type<
    FieldArgsT: darling::FromField + InspectFieldArgs + Clone,
    ArgsT: From<FieldArgsT> + ToTokens,
>(
    parsed_field: &mut Option<ParsedField>,
    f: &syn::Field,
    path: &syn::Path,
    default_render_trait: proc_macro2::TokenStream,
    arg_type: proc_macro2::TokenStream,
) {
    if f.attrs.iter().any(|x| x.path == *path) {
        handle_inspect_type::<FieldArgsT, ArgsT>(parsed_field, &f, default_render_trait, arg_type);
    }
}

// Does common data gathering and error checking, then creates `render` and `render_mut` methods
fn handle_inspect_type<
    FieldArgsT: darling::FromField + InspectFieldArgs + Clone,
    ArgsT: From<FieldArgsT> + ToTokens,
>(
    parsed_field: &mut Option<ParsedField>,
    f: &syn::Field,
    default_render_trait: proc_macro2::TokenStream,
    arg_type: proc_macro2::TokenStream,
) {
    //TODO: Improve error message
    if parsed_field.is_some() {
        panic!(
            "Too many inspect attributes on a single member {:?}",
            f.ident
        );
    }

    let field_args = FieldArgsT::from_field(&f).unwrap();

    if field_args.skip() {
        *parsed_field = Some(ParsedField {
            render: quote!(),
            render_mut: quote!(),
            //skip: true
        });

        return;
    }

    let render_trait = match field_args.render_trait() {
        Some(t) => t.clone(),
        None => syn::parse2::<syn::Path>(default_render_trait).unwrap(),
    };

    let arg_type = syn::parse2::<syn::Type>(arg_type).unwrap();
    let args: ArgsT = field_args.clone().into();

    let (render, render_mut) = RenderCall {
        field_name: field_args.ident().as_ref().unwrap(),
        field_type: field_args.ty(),
        render_trait: &render_trait,
        proxy_type: field_args.proxy_type(),
        arg_type: &arg_type,
        args: &args,
    }
    .create_calls(field_args.on_set());

    *parsed_field = Some(ParsedField {
        render,
        render_mut,
        //skip: false
    });
}

impl<'a, T: ToTokens> RenderCall<'a, T> {
    /// Returns (render, render_mut)
    pub fn create_calls(
        &self,
        on_set: &Option<syn::Ident>,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        (
            self.create_render_call(),
            self.create_render_mut_call(on_set),
        )
    }

    fn create_render_call(&self) -> proc_macro2::TokenStream {
        use quote::format_ident;
        let RenderCall {
            field_name,
            field_type,
            render_trait,
            proxy_type,
            arg_type,
            args,
        } = self;

        let args_name = format_ident!("_inspect_args_{}", field_name);

        let field_name = field_name.clone();

        let source_type = if let Some(w) = proxy_type {
            quote!(#w)
        } else {
            quote!(#field_type)
        };

        quote! {{
            #[allow(non_upper_case_globals)]
            const #args_name : #arg_type = #args;
            let values : Vec<_> = data.iter().map(|x| &x.#field_name).collect();
            if !data.is_empty() {
                <#source_type as #render_trait<#field_type>>::render(values.as_slice(), stringify!(#field_name), ui, &#args_name);
            }
        }}
    }

    fn create_render_mut_call(
        &self,
        on_set: &Option<syn::Ident>,
    ) -> proc_macro2::TokenStream {
        use quote::format_ident;
        let RenderCall {
            field_name,
            field_type,
            render_trait,
            proxy_type,
            arg_type,
            args,
        } = self;

        let args_name = format_ident!("_inspect_args_{}", field_name);

        let field_name = (*field_name).clone();

        let source_type = if let Some(w) = proxy_type {
            quote!(#w)
        } else {
            quote!(#field_type)
        };

        let on_set_callback_impl = match on_set {
            Some(ident) => quote! {{
               for d in data.iter_mut() {
                    d.#ident();
                }
            }},
            None => quote! {{}},
        };

        quote! {{
            #[allow(non_upper_case_globals)]
            const #args_name: #arg_type = #args;
            let mut values: Vec<_> = data.iter_mut().map(|x| &mut x.#field_name).collect();
            let mut changed = <#source_type as #render_trait<#field_type>>::render_mut(
                &mut values.as_mut_slice(),
                stringify!(#field_name),
                ui,
                &#args_name,
            );

            #on_set_callback_impl

            changed
        }}
    }
}
