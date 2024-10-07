use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta};

#[proc_macro_derive(InputAction, attributes(action_dim))]
pub fn input_action_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    const ATTR_NAME: &str = "action_dim";
    let mut dim = None;
    for attr in input.attrs {
        let Meta::List(list) = attr.meta else {
            continue;
        };

        if list
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == ATTR_NAME)
        {
            assert!(dim.is_none(), "`{ATTR_NAME}` can be defined only once");

            let mut token_iter = list.tokens.into_iter();
            let token = token_iter
                .next()
                .unwrap_or_else(|| panic!("`{ATTR_NAME}` should have argument"));

            let TokenTree::Ident(indent) = token else {
                panic!("`{token}` is invalid argument for `{ATTR_NAME}`");
            };

            dim = Some(indent);

            assert!(
                token_iter.next().is_none(),
                "`{ATTR_NAME}` should have only a single argument"
            );
        }
    }

    let dim = dim.unwrap_or_else(|| panic!("`InputAction` should have `{ATTR_NAME}` attribute"));
    let struct_name = input.ident;
    TokenStream::from(quote! {
        impl InputAction for #struct_name {
            const DIM: ActionValueDim = ActionValueDim::#dim;
        }
    })
}
