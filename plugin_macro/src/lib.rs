use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Ident, Token};
struct Input {
    head: Ident,
    items: Vec<Ident>,
}
impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        let head = input.parse()?;
        input.parse::<Token![,]>()?;
        while !input.is_empty() {
            items.push(input.parse()?);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Input { head, items })
    }
}

#[proc_macro]
pub fn bevy_plugin_group(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let snake_items = &input.items;
    let head = &input.head;

    let plugin_items = snake_items.iter().map(|ident| {
        Ident::new(
            &(String::from(&ident.to_string().to_case(Case::Pascal)) + "Plugin"),
            ident.span(),
        )
    });

    let plugin_group_name = Ident::new(
        &(String::from(head.to_string().to_case(Case::Pascal)) + "Plugins"),
        head.span(),
    );
    TokenStream::from(quote! {
        #(
            pub mod #snake_items;
            pub use #snake_items::*;
        )*

        pub struct #plugin_group_name;
        use bevy::{app::PluginGroupBuilder, app::PluginGroup};
        impl PluginGroup for #plugin_group_name {
            fn build(self) -> PluginGroupBuilder {
                PluginGroupBuilder::start::<Self>()
                    #(
                        .add(#snake_items::#plugin_items)
                    )*
            }
        }

    })
}
