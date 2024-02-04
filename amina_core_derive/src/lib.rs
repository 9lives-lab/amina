mod events;

use proc_macro::TokenStream;
use syn;

extern crate quote;

#[proc_macro_derive(Event, attributes(key))]
pub fn event_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    events::impl_event(&ast)
}
