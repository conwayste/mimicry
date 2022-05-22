extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::vec;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};

use syn::Variant;
use syn::{DataEnum, DeriveInput, Field, PathArguments::AngleBracketed};

fn extract_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let mut base_type = "".to_owned();
            // Path may be segmented like "a::<b::c>"
            for segment in &type_path.path.segments {
                base_type = segment.ident.to_string();

                // Does the type we found nest some other generic argument?
                if let AngleBracketed(abga) = &segment.arguments {
                    // arguments may be comma separated like "a<b,c>"
                    for a in &abga.args {
                        if let syn::GenericArgument::Type(nested_type_path) = &a {
                            let nested_type = extract_type(&nested_type_path);
                            base_type = format!("{}${}", base_type, nested_type);
                        }
                    }
                }
            }
            return base_type;
        }
        _ => {}
    }

    "".to_owned()
}

fn type_is_primitive(type_string: &String) -> bool {
    let primitives_str = "u128 i128 u64 i64 u32 i32 f32 u16 i16 u8 i8 bool String &str ()";
    for t in primitives_str.split_ascii_whitespace() {
        if t == type_string {
            return true;
        }

        let outer = type_string.split('|').next().unwrap();
        let inner = type_string.split('|').last().unwrap();
        if (outer == "Vec" || outer == "Option") && t == inner {
            return true;
        }
    }
    return false;
}

struct FieldParts {
    name: String,
    outer_type: String,
    inner_type_list: Vec<String>,
}

struct VariantMimic {
    name: String,
    fields: Vec<FieldParts>,
}

fn digest_field_into_parts(f: &Field) -> FieldParts {
    let mut inner_type_list = vec![];
    let mut type_string = extract_type(&f.ty);

    if !type_is_primitive(&type_string) {
        // HACK: Limited support covers single-depth nested types only
        // such as
        //      Vec<T> or Vec<T, U, V>
        // but not
        //      Vec<Option<T>>

        // Split off the first type in the list as the outer type with the remainder as a list of inner types
        inner_type_list = type_string
            .split('$')
            .map(|part| part.to_string())
            .collect();
        type_string = inner_type_list[0].clone();
        inner_type_list = inner_type_list.drain(1..).collect();
    }

    let field_name = f.ident.as_ref().unwrap().to_string();

    FieldParts {
        name: field_name,
        outer_type: type_string,
        inner_type_list,
    }
}

fn generate_mimic_from_variant(v: &Variant) -> VariantMimic {
    let variant_name = v.ident.to_string();
    let fields = v
        .fields
        .iter()
        .map(|f| digest_field_into_parts(f))
        .collect::<Vec<FieldParts>>();

    VariantMimic {
        name: variant_name,
        fields,
    }
}

fn build_mimic_field_from_parts(mimic_fields: &Vec<FieldParts>) -> Vec<TokenStream2> {
    let mut ifd_streams = vec![];
    for field in mimic_fields {
        let field_name = &field.name;
        let field_outer_type = &field.outer_type;
        let field_inner_types = &field.inner_type_list;
        let ifd_token_stream = quote! {
             MimicFieldData {
                 name: #field_name,
                 type_: #field_outer_type,
                type_arguments: vec![#(#field_inner_types,)*],
             }
        };
        ifd_streams.push(ifd_token_stream);
    }
    ifd_streams
}

fn generate_mimic_struct_for_each_variant(
    mimics: &Vec<VariantMimic>,
    input_enum_name_ident: &Ident,
) -> Vec<TokenStream2> {
    let mut parts: Vec<TokenStream2> = vec![];

    for mimic in mimics {
        let input_enum_name = input_enum_name_ident.to_string();
        let input_variant_name = mimic.name.clone();
        let concatenated_name = format!("{}{}", input_enum_name, input_variant_name);
        let mimic_name_ident = Ident::new(concatenated_name.as_str(), Span::call_site());

        let field_count = mimic.fields.len();
        let field_type_list_ident = mimic
            .fields
            .iter()
            .map(|f| Ident::new(f.outer_type.as_str(), Span::call_site()))
            .collect::<Vec<Ident>>();

        let mimicry_arg_ident = Ident::new(
            format!("MimicArg{}", field_count).as_str(),
            Span::call_site(),
        );
        let mimic_fields = build_mimic_field_from_parts(&mimic.fields);

        let item_list = (0..field_count)
            .map(|i| Ident::new(format!("f{}", i).as_str(), Span::call_site()))
            .collect::<Vec<Ident>>();

        // There is some repetition in this code between the three possibilities, but it's for the sake of explicitness.
        if field_count == 0 {
            let part = quote! {
            pub struct #mimic_name_ident {
                pub meta: MimicMetadata,
                pub instance: #mimicry_arg_ident,
            }
            impl Default for #mimic_name_ident {
                fn default() -> Self {
                    #mimic_name_ident {
                        meta: MimicMetadata {
                            name: #input_variant_name,
                            fields: vec![
                                #(#mimic_fields, )*
                            ],
                        },
                        instance: #mimicry_arg_ident {
                            #(#item_list: "".into()),*
                        }
                    }
                }
            }
            };
            parts.push(part);
        } else if field_count == 1 {
            let only_type = &field_type_list_ident[0];
            let part = quote! {
            pub struct #mimic_name_ident {
                pub meta: MimicMetadata,
                pub instance: #mimicry_arg_ident<#only_type>,
            }
            impl Default for #mimic_name_ident {
                fn default() -> Self {
                    #mimic_name_ident {
                        meta: MimicMetadata {
                            name: #input_variant_name,
                            fields: vec![
                                #(#mimic_fields, )*
                            ],
                        },
                        instance: #mimicry_arg_ident::<#only_type> {
                            f0: #only_type::from_str("0").unwrap()
                        },
                    }
                }
            }
            };
            parts.push(part);
        } else {
            let part = quote! {
            pub struct #mimic_name_ident {
                pub meta: MimicMetadata,
                pub instance: #mimicry_arg_ident<#(#field_type_list_ident),*>,
            }
                impl Default for #mimic_name_ident {
                    fn default() -> Self {
                        #mimic_name_ident {
                            meta: MimicMetadata {
                                name: #input_variant_name,
                                fields: vec![
                                    #(#mimic_fields, )*
                                ],
                            },
                            instance: #mimicry_arg_ident::< #(#field_type_list_ident),* > {
                                #(#item_list: "".into()),*
                            }
                        }
                    }
                }
            };
            parts.push(part);
        }
    }

    parts
}

fn generate_mimic_enum(
    mimics: &Vec<VariantMimic>,
    input_enum_name_ident: &Ident,
) -> Vec<TokenStream2> {
    let mut parts: Vec<TokenStream2> = vec![];

    let variant_name = input_enum_name_ident.to_string();
    let mimic_enum_name = format!("Mimic{}", variant_name);
    let mimic_enum_name_ident = Ident::new(mimic_enum_name.as_str(), Span::call_site());

    let mut variant_mimic_stream: Vec<TokenStream2> = vec![];
    for mimic in mimics {
        let mimic_name = mimic.name.clone();
        let concatenated_name = format!("{}{}", variant_name, mimic_name);
        let mimic_name_ident = Ident::new(concatenated_name.as_str(), Span::call_site());

        let field_count = mimic.fields.len();

        if field_count == 0 {
            variant_mimic_stream.push(quote! {
                #mimic_name_ident,
            });
        } else {
            variant_mimic_stream.push(quote! {
                #mimic_name_ident {
                    inner: #mimic_name_ident,
                },
            });
        }
    }

    parts.push(quote! {
        #[derive(Default)]
        pub enum #mimic_enum_name_ident {
            #[default]
            #(#variant_mimic_stream)*
        }
    });

    parts
}

/// Implement `TryFrom<Vec<String>>` for each mimic structure. This is so we can turn a vector of user input as
/// strings into  each individual mimic struct, assuming it parses cleanly.
///
///  impl TryFrom<Vec<String>> for FooA {
///      type Error = &'static str;
///
///      fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
///          let mut foo_b = FooB::default();
///          foo_b.instance = MimicArg2::<usize, String>::try_from(value)?;
///          Ok(foo_b)
///      }
/// }
///
fn generate_mimic_try_from(
    mimics: &Vec<VariantMimic>,
    input_enum_name_ident: &Ident,
) -> Vec<TokenStream2> {
    let mut parts = vec![];
    for mimic in mimics {
        let input_enum_name = input_enum_name_ident.to_string();
        let input_variant_name = mimic.name.clone();
        let concatenated_name = format!("{}{}", input_enum_name, input_variant_name);
        let mimic_name_ident = Ident::new(concatenated_name.as_str(), Span::call_site());

        let field_count = mimic.fields.len();
        let field_type_list_ident = mimic
            .fields
            .iter()
            .map(|f| Ident::new(f.outer_type.as_str(), Span::call_site()))
            .collect::<Vec<Ident>>();

        let mimicry_arg_ident = Ident::new(
            format!("MimicArg{}", field_count).as_str(),
            Span::call_site(),
        );

        parts.push(quote!{
            impl TryFrom<Vec<String>> for #mimic_name_ident {
                type Error = &'static str;

                fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
                    let mut mimic_arg = #mimic_name_ident::default();
                    mimic_arg.instance = #mimicry_arg_ident::<#(#field_type_list_ident),*>::try_from(value)?;
                    Ok(mimic_arg)
                }
            }
        });
    }
    parts
}

fn impl_mimic_for_enum(ast: &DeriveInput) -> TokenStream {
    let input_enum_name_ident = &ast.ident;
    let data = &ast.data;

    let mut mimics: Vec<VariantMimic> = vec![];

    // Iterate over the Enum's variants and create a mimic representation of each one
    match data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            for v in variants {
                let mimic = generate_mimic_from_variant(v);
                mimics.push(mimic);
            }
        }
        _ => unimplemented!(),
    };

    let mut all_tks2s: Vec<TokenStream2> = vec![];

    // Take an enum such as:
    //
    //  #[derive(Mimic)]
    // pub enum Foo {
    //     A,
    //     B {b0: usize, b1: String}
    // }
    //
    // and define a new struct for each variant containing metadata about the variant, and a type-accurate
    // mimic of it. Also `impl Default` for it. Bear in mind with the pseudo-code:
    //
    //  pub struct FooA {
    //       metadata: {name: "A", types: vec![]},
    //       inner: MimicArg0 {}
    //  }
    //
    //  impl Default for FooA {
    //      FooA {}
    //  }
    //
    //  pub struct FooB {
    //       metadata: {name: "B", types: {"usize", "String"}},
    //       inner: MimicArg2<usize, String>,
    //  }
    //
    //  impl Default for FooB {
    //      FooB {
    //          metadata: MimicMetadata { ... }
    //          instance: MimicArg2<usize, String> { ... }
    //      }
    //  }

    let tks2_mimic_structs =
        generate_mimic_struct_for_each_variant(&mimics, input_enum_name_ident);
    all_tks2s.extend(tks2_mimic_structs);

    // Take the collection of all generated mimic structures into an enumeration for matching upon by the library user.
    //
    // pub enum MimicFoo {
    //     FooA { inner: FooA }
    //     FooB { inner: FooB }
    // }
    //
    let tks2_mimic_enum = generate_mimic_enum(&mimics, input_enum_name_ident);
    all_tks2s.extend(tks2_mimic_enum);

    let tks2_try_froms = generate_mimic_try_from(&mimics, input_enum_name_ident);
    all_tks2s.extend(tks2_try_froms);

    let final_token_stream: TokenStream = quote! {
        #(#all_tks2s)*
    }
    .into();

    final_token_stream
}

#[proc_macro_derive(Mimic)]
pub fn generate_mimic_for(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input);

    // Build the implementations for the enumeration
    let gen = impl_mimic_for_enum(&ast);

    // Return the generated impl
    gen
}
