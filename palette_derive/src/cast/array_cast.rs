use proc_macro::TokenStream;
use proc_macro2::Span;

use quote::{quote, ToTokens};
use syn::{Attribute, Data, DeriveInput, Fields, Ident, Type};

use crate::meta::{self, FieldAttributes, IdentOrIndex, TypeItemAttributes};
use crate::util;

pub fn derive(tokens: TokenStream) -> std::result::Result<TokenStream, Vec<syn::Error>> {
    let DeriveInput {
        ident,
        attrs,
        generics,
        data,
        ..
    } = syn::parse(tokens).map_err(|error| vec![error])?;

    let allowed_repr = is_allowed_repr(&attrs)?;
    let item_meta: TypeItemAttributes = meta::parse_namespaced_attributes(attrs)?;

    let mut number_of_channels = 0usize;
    let mut field_type: Option<Type> = None;

    let (all_fields, fields_meta) = match data {
        Data::Struct(struct_item) => {
            let fields_meta: FieldAttributes =
                meta::parse_field_attributes(struct_item.fields.clone())?;
            let all_fields = match struct_item.fields {
                Fields::Named(fields) => fields.named,
                Fields::Unnamed(fields) => fields.unnamed,
                Fields::Unit => Default::default(),
            };

            (all_fields, fields_meta)
        }
        Data::Enum(_) => {
            return Err(vec![syn::Error::new(
                Span::call_site(),
                "`ArrayCast` cannot be derived for enums, because of the discriminant",
            )]);
        }
        Data::Union(_) => {
            return Err(vec![syn::Error::new(
                Span::call_site(),
                "`ArrayCast` cannot be derived for unions",
            )]);
        }
    };

    let fields = all_fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            (
                field
                    .ident
                    .map(IdentOrIndex::Ident)
                    .unwrap_or_else(|| IdentOrIndex::Index(index.into())),
                field.ty,
            )
        })
        .filter(|&(ref field, _)| !fields_meta.zero_size_fields.contains(field));

    let mut errors = Vec::new();

    for (field, ty) in fields {
        let ty = fields_meta
            .type_substitutes
            .get(&field)
            .cloned()
            .unwrap_or(ty);
        number_of_channels += 1;

        if let Some(field_type) = field_type.clone() {
            if field_type != ty {
                errors.push(syn::Error::new_spanned(
                    &field,
                    format!(
                        "expected fields to have type `{}`",
                        field_type.into_token_stream()
                    ),
                ));
            }
        } else {
            field_type = Some(ty);
        }
    }

    if !allowed_repr {
        errors.push(syn::Error::new(
            Span::call_site(),
            format!(
                "a `#[repr(C)]` or `#[repr(transparent)]` attribute is required to give `{}` a fixed memory layout",
                ident
            ),
        ));
    }

    let array_cast_trait_path = util::path(&["cast", "ArrayCast"], item_meta.internal);

    let mut implementation = if let Some(field_type) = field_type {
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

        quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #array_cast_trait_path for #ident #type_generics #where_clause {
                type Array = [#field_type; #number_of_channels];
            }
        }
    } else {
        errors.push(syn::Error::new(
            Span::call_site(),
            "`ArrayCast` can only be derived for structs with one or more fields".to_string(),
        ));

        return Err(errors);
    };

    implementation.extend(errors.iter().map(syn::Error::to_compile_error));
    Ok(implementation.into())
}

fn is_allowed_repr(attributes: &[Attribute]) -> std::result::Result<bool, Vec<syn::Error>> {
    let mut errors = Vec::new();

    for attribute in attributes {
        let attribute_name = attribute.path.get_ident().map(ToString::to_string);

        if let Some("repr") = attribute_name.as_deref() {
            let items = match meta::parse_tuple_attribute(attribute.tokens.clone()) {
                Ok(items) => items,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };

            let contains_allowed_repr = items
                .iter()
                .any(|item: &Ident| item == "C" || item == "transparent");

            if contains_allowed_repr {
                return Ok(true);
            }
        }
    }

    if errors.is_empty() {
        Ok(false)
    } else {
        Err(errors)
    }
}
