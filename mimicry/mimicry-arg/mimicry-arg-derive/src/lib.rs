extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::LitInt;

mod args;

#[proc_macro_attribute]
pub fn mimic_arg_n(attr: TokenStream, _item: TokenStream) -> TokenStream {
    let depth_litint = syn::parse_macro_input!(attr as LitInt);

    let depth = depth_litint.base10_parse().unwrap();

    // Define a type-accurate mimic scheme for N arguments.
    //
    // To represent an enum variant that has two arguments, it will define:
    //
    //  pub struct MimicArg2<R, S> {
    //      f0: R
    //      f1: S
    //  }
    //
    //  where R and S are the individual types of the variant's fields, respectively.
    //
    // It will also define two trait implementations by default:
    //      1. `impl TryFrom<Vec<String>>`
    //          where each item in the Vector is the string-to-be-parsed into `f0` and `f1`.
    //          The vector size on TryFrom is checked during run-time and will return a parsing error on misuse.
    //
    //      2. `impl FromStr`
    //
    let tks2_mimic_arg_n = args::define_mimic_arg_n(depth);

    let final_token_stream: TokenStream = quote! {
        #(#tks2_mimic_arg_n)*
    }
    .into();

    final_token_stream
}
