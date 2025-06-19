use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, parse_macro_input};

#[derive(FromDeriveInput)]
#[darling(attributes(input_action))]
struct InputActionOpts {
    output: Ident,
    #[darling(default)]
    accumulation: Option<Ident>,
    #[darling(default)]
    consume_input: Option<bool>,
    #[darling(default)]
    require_reset: Option<bool>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(input_context))]
struct InputContextOpts {
    #[darling(default)]
    schedule: Option<Ident>,
    #[darling(default)]
    priority: Option<usize>,
}

#[proc_macro_derive(InputAction, attributes(input_action))]
pub fn input_action_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    #[expect(non_snake_case, reason = "item shortcuts")]
    let (Accumulation, InputAction) = (
        quote! { ::bevy_enhanced_input::prelude::Accumulation },
        quote! { ::bevy_enhanced_input::prelude::InputAction },
    );

    let opts = match InputActionOpts::from_derive_input(&input) {
        Ok(value) => value,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let struct_name = input.ident;
    let output = opts.output;
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
    let require_reset = if let Some(reset) = opts.require_reset {
        quote! {
            const REQUIRE_RESET: bool = #reset;
        }
    } else {
        Default::default()
    };

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #InputAction for #struct_name #type_generics #where_clause {
            type Output = #output;
            #accumulation
            #consume_input
            #require_reset
        }
    })
}

#[proc_macro_derive(InputContext, attributes(input_context))]
pub fn input_context_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    #[expect(non_snake_case, reason = "item shortcuts")]
    let InputContext = quote! { ::bevy_enhanced_input::prelude::InputContext };

    let opts = match InputContextOpts::from_derive_input(&input) {
        Ok(value) => value,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let struct_name = input.ident;
    let priority = if let Some(priority) = opts.priority {
        quote! {
            const PRIORITY: usize = #priority;
        }
    } else {
        Default::default()
    };
    let schedule = if let Some(schedule) = opts.schedule {
        quote! { #schedule }
    } else {
        quote! { ::bevy::app::PreUpdate }
    };

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #InputContext for #struct_name #type_generics #where_clause {
            type Schedule = #schedule;
            #priority
        }
    })
}
