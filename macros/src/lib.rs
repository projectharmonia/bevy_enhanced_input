use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Ident, Path};

#[derive(FromDeriveInput)]
#[darling(attributes(input_action))]
struct InputActionOpts {
    output: Ident,
    #[darling(default)]
    accumulation: Option<Ident>,
    #[darling(default)]
    consume_input: Option<bool>,
}

#[proc_macro_derive(InputAction, attributes(input_action))]
pub fn input_action_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    #[expect(non_snake_case, reason = "item shortcuts")]
    let (Accumulation, InputAction) = (
        quote! { ::bevy_enhanced_input::prelude::Accumulation },
        quote! { ::bevy_enhanced_input::prelude::InputAction },
    );

    let InputActionOpts {
        output,
        accumulation,
        consume_input,
    } = match InputActionOpts::from_derive_input(&input) {
        Ok(value) => value,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let accumulation = accumulation.map_or_else(Default::default, |accumulation| {
        quote! {
            const ACCUMULATION: #Accumulation = #Accumulation::#accumulation;

        }
    });
    let consume_input = consume_input.map_or_else(Default::default, |consume| {
        quote! {
            const CONSUME_INPUT: bool = #consume;
        }
    });

    let struct_name = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #InputAction for #struct_name #type_generics #where_clause {
            type Output = #output;
            #accumulation
            #consume_input
        }
    })
}

#[derive(FromDeriveInput)]
#[darling(attributes(input_context))]
struct InputContextOpts {
    instance_system: Path,
    #[darling(default)]
    mode: Option<Ident>,
    #[darling(default)]
    priority: Option<Expr>,
}

#[proc_macro_derive(InputContext, attributes(input_context))]
pub fn input_context_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    #[expect(non_snake_case, reason = "item shortcuts")]
    let (InputContext, ContextInstance, ContextMode, Entity, IntoSystem, ReadOnlySystem) = (
        quote! { ::bevy_enhanced_input::prelude::InputContext },
        quote! { ::bevy_enhanced_input::prelude::ContextInstance },
        quote! { ContextMode },
        // TODO
        quote! { Entity },
        quote! { IntoSystem },
        quote! { ReadOnlySystem },
    );

    let InputContextOpts {
        instance_system,
        mode,
        priority,
    } = match InputContextOpts::from_derive_input(&input) {
        Ok(value) => value,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let mode = mode.map_or_else(Default::default, |mode| {
        quote! {
            const MODE: #ContextMode = #ContextMode::#mode;
        }
    });
    let priority = priority.map_or_else(Default::default, |priority| {
        quote! {
            const PRIORITY: isize = #priority;
        }
    });

    let struct_name = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #InputContext for #struct_name #type_generics #where_clause {
            fn instance_system() -> impl #ReadOnlySystem<In = #Entity, Out = #ContextInstance> {
                #IntoSystem::into_system(#instance_system)
            }
            #mode
            #priority
        }
    })
}
