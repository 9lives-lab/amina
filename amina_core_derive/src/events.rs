use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Meta;
use syn::Lit;

pub fn impl_event(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let attr = ast
        .attrs
        .iter()
        .find_map(|a| {
            let a = a.parse_meta();
            match a {
                Ok(meta) => {
                    if meta.path().is_ident("key") {
                        Some(meta)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .unwrap();

    let key = match attr {
        Meta::NameValue(value) => {
            //println!("{}", value.path.get_ident().unwrap());
            match value.lit {
                Lit::Str(str_value) => {
                    str_value.value()
                }
                _ => {panic!()}
            }
        }
        _ => {panic!()}
    };

    let a = quote! {
        impl Event for #name {
            fn get_key() -> &'static str {
                #key
            }
        }
    };
    a.into()
}
