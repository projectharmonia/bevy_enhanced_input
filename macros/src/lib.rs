use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Ident, Path, parse_macro_input};

#[derive(FromDeriveInput)]
#[darling(attributes(input_context))]
struct InputContextOpts {
    #[darling(default)]
    schedule: Option<Ident>,
    #[darling(default)]
    priority: Option<usize>,
}

#[proc_macro_derive(InputAction, attributes(action_output))]
pub fn input_action_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let Some(attr) = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("action_output"))
    else {
        return Error::new_spanned(&input, "Missing #[action_output(Type)] attribute")
            .to_compile_error()
            .into();
    };

    let output_ty = match attr.parse_args::<Path>() {
        Ok(output_ty) => output_ty,
        Err(e) => return e.to_compile_error().into(),
    };

    let trait_name = quote! { ::bevy_enhanced_input::prelude::InputAction };
    let struct_name = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #trait_name for #struct_name #type_generics #where_clause {
            type Output = #output_ty;
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
    let struct_name = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #InputContext for #struct_name #type_generics #where_clause {
            type Schedule = #schedule;
            #priority
        }
    })
}
