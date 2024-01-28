use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

extern crate proc_macro;

#[proc_macro_attribute]
pub fn croissant(_: TokenStream, input: TokenStream) -> TokenStream {
    // let attr = proc_macro2::TokenStream::from(a);
    // let item = proc_macro2::TokenStream::from(i);
    //
    // let t: Fn = syn::parse(item.into()).unwrap();
    // println!("{}", t);
    // item.into()
    // Parse the input as a function
    let input = parse_macro_input!(input as ItemFn);

    // Extract the function name and input type
    let fn_name = &input.sig.ident;
    // let input_type = &input.sig.inputs;

    let wrapper_name = proc_macro2::Ident::new(
        &(fn_name.to_string() + "_croissant"),
        Span::call_site(),
    );
    let expanded = quote! {
        #input

        fn #wrapper_name(c: C) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = ()> + Send + Sync>> {
            Box::pin(#fn_name(c))
        }
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

