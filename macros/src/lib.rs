use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[derive(FromDeriveInput)]
#[darling(attributes(input_action))]
struct InputActionOpts {
    dim: Ident,
    #[darling(default)]
    accumulation: Option<Ident>,
    #[darling(default)]
    consume_input: Option<bool>,
}

#[proc_macro_derive(InputAction, attributes(input_action))]
pub fn input_action_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let opts = match InputActionOpts::from_derive_input(&input) {
        Ok(value) => value,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let struct_name = input.ident;
    let dim = opts.dim;
    let accumulation = if let Some(accumulation) = opts.accumulation {
        quote! {
            const ACCUMULATION: Accumulation = Accumulation::#accumulation;
        }
    } else {
        Default::default()
    };
    let consume_input = if let Some(consume) = opts.consume_input {
        quote! {
            const CONSUME_INPUT: bool = #consume;
        }
    } else {
        Default::default()
    };

    TokenStream::from(quote! {
        impl InputAction for #struct_name {
            const DIM: ActionValueDim = ActionValueDim::#dim;
            #accumulation
            #consume_input
        }
    })
}
