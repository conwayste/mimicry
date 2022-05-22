use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};

// ARBITRARY: bound to ensure type lettering contiguously iterates from `A` to `Z` in Unicode
const MAX_RECURSIVE_DEPTH: usize = 26;

pub fn define_mimic_arg_n(n: usize) -> Vec<TokenStream2> {
    let mut parts: Vec<TokenStream2> = vec![];

    if n > MAX_RECURSIVE_DEPTH {
        panic!("An enum variant cannot exceed a hard-coded limit of 26 members. PR's are always welcome.");
    }

    let name_arg0 = format!("MimicArg0");
    let name_arg0_ident = Ident::new(name_arg0.as_str(), Span::call_site());

    // Create the Arg0 struct, TryFrom, and FromStr implementations
    parts.push(quote! {
        use std::{fmt::Debug, str::FromStr};

        #[derive(Default)]
        pub struct #name_arg0_ident {}

        impl TryFrom<Vec<String>> for #name_arg0_ident {
            type Error = &'static str;
            fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
                if value.len() != 0usize {
                    return Err("Non-zero list-length provided to MimicArg0 for conversion TypeFrom<Vec<String>>");
                }
                return Ok(#name_arg0_ident::default());
            }
        }

        impl FromStr for #name_arg0_ident {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if !s.is_empty() {
                    return Err("FromStr for MimicArg0 expects no arguments");
                }
                return Ok(#name_arg0_ident::default());
            }
        }
    });

    // Create the Arg1 through ArgN structs, TryFrom, and FromStr implementations
    (1..=n).for_each(|i| {
        let name = format!("MimicArg{}", i);
        let name_ident = Ident::new(name.as_str(), Span::call_site());
        let args_error_msg = format!("Insufficient list length provided to {} for conversion TryFrom<Vec<String>>", name);

        let generics_as_chars = (0..i)
            .map(|j| {
                // Unwraps are safe because 'A'+i is always convertible to base 10 in unicode with a limit on n of 26
                char::from_u32('A' as u32 + j as u32).unwrap()
            })
            .collect::<Vec<char>>();

        let generics_as_markers = generics_as_chars
            .iter()
            .map(|c| Ident::new(c.to_string().as_str(), Span::call_site()))
            .collect::<Vec<Ident>>();

        let item_list = (0..i)
            .map(|j| Ident::new(format!("f{}", j).as_str(), Span::call_site()))
            .collect::<Vec<Ident>>();

        let nested_tryfrom_parsing_calls = generate_recursive_tryfrom_parsing_calls(i, i, &item_list, &generics_as_chars, &generics_as_markers, &name);
        let listed_fromstr_parsing_calls = generate_recursive_fromstr_parsing_calls( i, &item_list, &generics_as_chars, &generics_as_markers);

        let no_csv_error_msg = format!(
            "No comma-separated list found in argument to `from_str` for MimicArg{}",
            i,
        );
        let no_csv_error_msg_str = no_csv_error_msg.as_str();

        let invalid_argn_error_msg = format!(
            "Invalid number of CSV arguments provided to `from_str` for MimicArg{}",
            i,
        );
        let invalid_argn_error_msg_str = invalid_argn_error_msg.as_str();

        parts.push(quote! {
            #[derive(Default)]
            pub struct #name_ident<#(#generics_as_markers),*> {
                #(pub #item_list: #generics_as_markers),*
            }

            impl<#(#generics_as_markers),*> TryFrom<Vec<String>> for #name_ident<#(#generics_as_markers),*>
            where
                #(#generics_as_markers: Debug + FromStr),*
            {
                type Error = &'static str;

                fn try_from(input: Vec<String>) -> Result<Self, Self::Error> {
                    if input.len() != #i {
                        return Err(#args_error_msg);
                    }

                    #(#nested_tryfrom_parsing_calls)*
                }
            }

            impl<#(#generics_as_markers),*> FromStr for #name_ident<#(#generics_as_markers),*>
            where
                #(#generics_as_markers: Debug + FromStr),*
            {
                type Err = &'static str;

                fn from_str(input: &str) -> Result<Self, Self::Err> {
                    let csv = input.split(",")
                    .filter_map(|s: &str| {
                        let t = s.trim();

                        if !t.is_empty() {
                            Some(t)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<&str>>();
                    if csv.is_empty() {
                        return Err(#no_csv_error_msg_str);
                    }
                    if csv.len() != #i {
                        return Err(#invalid_argn_error_msg_str);
                    }
                    Ok(#name_ident {
                        #(#listed_fromstr_parsing_calls)*
                    })
                }
            }
        });
    });

    parts
}

fn generate_recursive_tryfrom_parsing_calls(
    total: usize,
    n: usize,
    item_list: &Vec<Ident>,
    generics_as_chars: &Vec<char>,
    generics_as_markers: &Vec<Ident>,
    struct_name: &String,
) -> Vec<TokenStream2> {
    let mut output_stream2 = vec![];
    if item_list.len() < total {
        return output_stream2;
    }

    if n == 0 {
        fn base_case(struct_name_ident: &Ident, item_list: &Vec<Ident>) -> Vec<TokenStream2> {
            vec![quote! {
                return Ok(#struct_name_ident {#(#item_list),*});
            }]
        }

        let struct_name_ident = Ident::new(struct_name.as_str(), Span::call_site());

        let instantiate_arg_token_stream = base_case(&struct_name_ident, item_list);
        output_stream2.extend(instantiate_arg_token_stream)
    } else if n >= 1 {
        let list_index = total - n;
        let item_in_list = &item_list[list_index];
        let generic_in_list = &generics_as_markers[list_index];

        let generics_list = generics_as_chars
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        let generics_csv = generics_list.join(",");
        let error_msg = format!(
            "Failed to parse `{}` in {}<{}>",
            generics_list[list_index], struct_name, generics_csv
        );

        let nested_token_stream = generate_recursive_tryfrom_parsing_calls(
            total,
            n - 1,
            item_list,
            generics_as_chars,
            generics_as_markers,
            struct_name,
        );

        output_stream2.push(quote! {
            if let Ok(#item_in_list) = input[#list_index].parse::<#generic_in_list>() {
                #(#nested_token_stream)*
            }
            return Err(#error_msg);
        });
    }

    output_stream2
}

fn generate_recursive_fromstr_parsing_calls(
    i: usize,
    item_list: &Vec<Ident>,
    generics_as_chars: &Vec<char>,
    generics_as_markers: &Vec<Ident>,
) -> Vec<TokenStream2> {
    let mut output_stream2: Vec<TokenStream2> = vec![];
    if i == 0 {
        return output_stream2;
    }

    if i > item_list.len() || i > generics_as_markers.len() {
        panic!(
            "FromStr generation unexpected list lengths i={} argn={} num_markers={}",
            i,
            item_list.len(),
            generics_as_markers.len()
        );
    }

    let index = i - 1;
    let argname = &item_list[index];
    let marker = &generics_as_markers[index];

    output_stream2.extend(generate_recursive_fromstr_parsing_calls(
        i - 1,
        item_list,
        generics_as_chars,
        generics_as_markers,
    ));

    let error_msg = format!(
        "Failed to parse argument {} as generic type {} in MimicArg{}",
        index,
        generics_as_chars[index],
        item_list.len()
    );
    let error_msg_str = error_msg.as_str();

    output_stream2.push(quote! {
        #argname: csv[#index].parse::<#marker>().map_err(|_| #error_msg_str)?,
    });

    output_stream2
}
