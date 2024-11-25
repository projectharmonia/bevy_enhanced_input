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

    #[expect(non_snake_case, reason = "item shortcuts")]
    let (Accumulation, InputAction, ActionValueDim) = (
        quote! { bevy_enhanced_input::prelude::Accumulation },
        quote! { bevy_enhanced_input::prelude::InputAction },
        quote! { bevy_enhanced_input::prelude::ActionValueDim },
    );

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
            const ACCUMULATION: #Accumulation = #Accumulation::#accumulation;
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

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #InputAction for #struct_name #type_generics #where_clause {
            const DIM: #ActionValueDim = #ActionValueDim::#dim;
            #accumulation
            #consume_input
        }
    })
}
