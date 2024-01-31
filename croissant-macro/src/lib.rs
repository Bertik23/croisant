use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Pat};

extern crate proc_macro;

/// Creates a wrapper function, that can be Boxed and passed to
/// Croissant scheduler
/// ```
///
/// #[croissant]
/// async fn hello(msg: String) {
///     println!("{}", msg);
/// }
///
/// fn main() {
///     let mut croissant = Croissant::new();
///     croissant
///         .add_async_job("Hello World".to_string(), Box::new(hello_croissant));
///     croissant.run_every(Duration::from_secs(5));
///     croissant.run_at(NaiveTime::from_hms(12, 0, 0));
///     thread::sleep(Duration::from_secs(100));
/// }
/// ```
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
    let visibility = &input.vis;
    let fn_name = &input.sig.ident;
    let input_type = &input.sig.inputs;

    let _pnk = input_type.iter().fold("".to_string(), |y, x| {
        y + (format!("L: {:?}", x.to_token_stream())).as_str()
    });
    // panic!("{}", pnk);
    let params: Vec<_> = input_type
        .iter()
        .map(|param| {
            if let syn::FnArg::Typed(pat_type) = param {
                if let Pat::Ident(ident) = &*pat_type.pat {
                    // if let Type::Path(type_path) = &*pat_type.ty {
                    let param_name = &ident.ident;
                    // let param_type = &type_path.path;

                    // You can now use param_name and param_type as needed

                    param_name
                    // } else {
                    //     panic!("Unsupported parameter type");
                    // }
                } else {
                    panic!("Unsupported parameter pattern");
                }
            } else {
                panic!("Unsupported function argument");
            }
        })
        .collect();

    // panic!("{:?}", param_name);

    let wrapper_name = proc_macro2::Ident::new(
        &(fn_name.to_string() + "_croissant"),
        Span::call_site(),
    );
    let expanded = quote! {
        #input

        #visibility fn #wrapper_name(#input_type) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = ()> + Send + Sync>> {
            Box::pin(#fn_name(#(#params),*))
        }
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}
