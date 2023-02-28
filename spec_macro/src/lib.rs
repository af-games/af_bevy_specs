use proc_macro::TokenStream;

use quote::quote;
use syn::punctuated::Punctuated;

use std::result::Result::{Err, Ok};
use syn::{AngleBracketedGenericArguments, Data, Field, Fields, GenericArgument, Ident, Token};

#[proc_macro_derive(Spec)]
pub fn spec_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_spec(ast).unwrap_or_else(to_compile_errors).into()
}

fn impl_spec(
    ast: syn::DeriveInput,
) -> std::result::Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    let name = &ast.ident;
    let mut named_handle_field_names: Vec<String> = vec![];
    let mut named_handle_field_idents: Vec<Ident> = vec![];
    let mut named_handle_field_args: Vec<Punctuated<GenericArgument, Token![,]>> = vec![];

    let mut named_handle_vec_names: Vec<String> = vec![];
    let mut named_handle_vec_idents: Vec<Ident> = vec![];
    let mut named_handle_vec_args: Vec<Punctuated<GenericArgument, Token![,]>> = vec![];

    if let Data::Struct(ref data_struct) = ast.data {
        if let Fields::Named(ref named_fields) = data_struct.fields {
            for field in named_fields.named.iter() {
                let maybe_parsed_field = parse_field(field);
                match maybe_parsed_field {
                    Ok(parsed_field) => match parsed_field {
                        SpecHandleField::NamedHandle {
                            field_name,
                            ident,
                            args,
                        } => {
                            named_handle_field_names.push(field_name);
                            named_handle_field_idents.push(ident.clone());
                            named_handle_field_args.push(args.args);
                        }
                        SpecHandleField::NamedHandleVec {
                            field_name,
                            ident,
                            args,
                        } => {
                            named_handle_vec_names.push(field_name);
                            named_handle_vec_idents.push(ident.clone());
                            named_handle_vec_args.push(args.args);
                        }
                        SpecHandleField::OtherField => {}
                    },
                    Err(_) => {}
                }
            }
        }
    }
    let moo = quote! {
        impl #name {
            pub fn hey() {}
        }
        impl PopulateHandles for #name {
            fn populate_handles(&mut self, mut ass: &mut AssetServer) {
                #(
                    self.#named_handle_field_idents.maybe_handle =
                        Some(Arc::new(ass.load::<#named_handle_field_args, String>(self.#named_handle_field_idents.name.clone())));
                )*
                #(
                    for mut v in self.#named_handle_vec_idents.iter_mut() {
                        v.maybe_handle = Some(Arc::new(ass.load::<#named_handle_vec_args, String>(v.name.clone())));
                    }
                )*
            }
        }
    };
    Ok(moo)
}

#[derive(Debug)]
enum ParseFieldError {}

enum SpecHandleField {
    NamedHandle {
        field_name: String,
        ident: Ident,
        args: AngleBracketedGenericArguments,
    },
    NamedHandleVec {
        field_name: String,
        ident: Ident,
        args: AngleBracketedGenericArguments,
    },
    OtherField,
}

fn parse_field(field: &Field) -> std::result::Result<SpecHandleField, Vec<ParseFieldError>> {
    let field_ident = field.clone().ident.unwrap();
    let ident = field_ident.clone();
    match field.ty.clone() {
        syn::Type::Path(type_path) => {
            let path = type_path.path;

            for seg in path.segments.iter() {
                // dbg!(&seg.ident.to_string());
                match seg.ident.to_string().as_str() {
                    "NamedHandle" => match &seg.arguments {
                        syn::PathArguments::AngleBracketed(abgas) => {
                            return Ok(SpecHandleField::NamedHandle {
                                field_name: field_ident.to_string(),
                                ident,
                                args: abgas.clone(),
                            });
                        }
                        _ => {}
                    },
                    "Vec" => {
                        match &seg.arguments {
                            syn::PathArguments::AngleBracketed(abgas) => {
                                if abgas.args.len() == 1 {
                                    let first_arg: &GenericArgument = abgas.args.first().unwrap();
                                    match first_arg {
                                        GenericArgument::Type(ty) => match ty {
                                            syn::Type::Path(inner_type_path) => {
                                                let inner_path = &inner_type_path.path;
                                                for inner_seg in inner_path.segments.iter() {
                                                    match inner_seg.ident.to_string().as_str() {
                                                        "NamedHandle" => {
                                                            match &inner_seg.arguments {
                                                                syn::PathArguments::AngleBracketed(abgas2) => {
                                                                    return Ok(SpecHandleField::NamedHandleVec {
                                                                        field_name: field_ident.to_string(),
                                                                        ident,
                                                                        args: abgas2.clone(),
                                                                    });
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                    break;
                                                }
                                            }
                                            _ => {}
                                        },
                                        _ => {}
                                    }
                                } else {
                                    panic!();
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                break;
            }
        }
        _ => {}
    }
    Ok(SpecHandleField::OtherField)
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
