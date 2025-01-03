use quote::quote;
use syn::{Error, ExprClosure, Path};

static LDTK_INT_CELL_ATTRIBUTE_NAME: &str = "ldtk_int_cell";
static FROM_INT_GRID_CELL_ATTRIBUTE_NAME: &str = "from_int_grid_cell";
static WITH_ATTRIBUTE_NAME: &str = "with";
static DEFAULT_ATTRIBUTE_NAME: &str = "default";

pub fn expand_ldtk_int_cell_derive(ast: syn::DeriveInput) -> proc_macro::TokenStream {
    let struct_name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("Expected a struct with named fields."),
    };

    let mut field_constructions = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let ldtk_int_cell = field
            .attrs
            .iter()
            .find(|a| *a.path().get_ident().as_ref().unwrap() == LDTK_INT_CELL_ATTRIBUTE_NAME);
        if let Some(attribute) = ldtk_int_cell {
            field_constructions.push(expand_ldtk_int_cell_attribute(
                attribute, field_name, field_type,
            ));
            continue;
        }

        let from_int_grid_cell = field
            .attrs
            .iter()
            .find(|a| *a.path().get_ident().as_ref().unwrap() == FROM_INT_GRID_CELL_ATTRIBUTE_NAME);
        if let Some(attribute) = from_int_grid_cell {
            field_constructions.push(expand_from_int_grid_cell_attribute(
                attribute, field_name, field_type,
            ));
            continue;
        }

        let with = field
            .attrs
            .iter()
            .find(|a| *a.path().get_ident().as_ref().unwrap() == WITH_ATTRIBUTE_NAME);
        if let Some(attribute) = with {
            match expand_with_attribute(attribute, field_name, field_type) {
                Ok(attr) => field_constructions.push(attr),
                Err(e) => return e.into(),
            }
            continue;
        }

        let default = field
            .attrs
            .iter()
            .find(|a| *a.path().get_ident().as_ref().unwrap() == DEFAULT_ATTRIBUTE_NAME);
        if let Some(attribute) = default {
            field_constructions.push(expand_default_attribute(attribute, field_name, field_type));
            continue;
        }
    }

    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let struct_update = if field_constructions.len() < fields.len() {
        quote! { ..<Self as std::default::Default>::default() }
    } else {
        quote! {}
    };

    let gen = quote! {
        impl #impl_generics bevy_ecs_ldtk::prelude::LdtkIntCell for #struct_name #ty_generics #where_clause {
            fn bundle_int_cell(
                int_grid_cell: bevy_ecs_ldtk::prelude::IntGridCell,
                layer_instance: &bevy_ecs_ldtk::prelude::LayerInstance,
            ) -> Self {
                Self {
                    #(#field_constructions)*
                    #struct_update
                }
            }
        }
    };
    gen.into()
}

fn expand_ldtk_int_cell_attribute(
    attribute: &syn::Attribute,
    field_name: &syn::Ident,
    field_type: &syn::Type,
) -> proc_macro2::TokenStream {
    match attribute.meta {
        syn::Meta::Path(_) => {
            quote! {
                #field_name: <#field_type as bevy_ecs_ldtk::prelude::LdtkIntCell>::bundle_int_cell(int_grid_cell, layer_instance),
            }
        }
        _ => panic!("#[ldtk_int_cell] attribute should take the form #[ldtk_int_cell]"),
    }
}

fn expand_from_int_grid_cell_attribute(
    attribute: &syn::Attribute,
    field_name: &syn::Ident,
    field_type: &syn::Type,
) -> proc_macro2::TokenStream {
    match attribute.meta {
        syn::Meta::Path(_) => {
            quote! {
                #field_name: <#field_type as From<bevy_ecs_ldtk::prelude::IntGridCell>>::from(int_grid_cell),
            }
        }
        _ => {
            panic!("#[from_int_grid_cell] attribute should take the form #[from_int_grid_cell]")
        }
    }
}

fn expand_with_attribute(
    attribute: &syn::Attribute,
    field_name: &syn::Ident,
    _: &syn::Type,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    if let syn::Meta::List(syn::MetaList { ref tokens, .. }) = attribute.meta {
        if let Ok(path) = syn::parse2::<Path>(tokens.clone()) {
            return Ok(quote! {
                #field_name: #path(int_grid_cell),
            });
        }
        if let Ok(closure) = syn::parse2::<ExprClosure>(tokens.clone()) {
            return Ok(quote! {
                #field_name: (#closure)(int_grid_cell),
            });
        }
    }
    Err(Error::new_spanned(
        attribute,
        "#[with...] attribute should take the form #[with(function_name) or #[with(|_| {..})]]",
    )
    .into_compile_error())
}

fn expand_default_attribute(
    attribute: &syn::Attribute,
    field_name: &syn::Ident,
    _: &syn::Type,
) -> proc_macro2::TokenStream {
    match attribute.meta {
        syn::Meta::Path(_) => {
            quote! {
                #field_name: Default::default(),
            }
        }
        _ => panic!("#[default] attribute should take the form #[default]"),
    }
}
