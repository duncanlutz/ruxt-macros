use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, token::Colon, visit_mut::VisitMut, ItemFn};

pub(crate) fn process_route(item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(item as ItemFn);

    println!("{:#?}", input);

    let mut sig = input.sig.clone();

    let layout_arg = get_layout_arg();

    sig.inputs.push(layout_arg);

    input.sig = sig;

    // Add a test input

    quote!(#input).into()
}

fn get_layout_arg() -> syn::FnArg {
    syn::FnArg::Typed(syn::PatType {
        attrs: vec![],
        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: syn::Ident::new("layout", proc_macro2::Span::call_site()),
            subpat: None,
        })),
        colon_token: Colon::default(),
        ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: vec![
                    syn::PathSegment {
                        ident: syn::Ident::new("actix_web", proc_macro2::Span::call_site()),
                        arguments: syn::PathArguments::None,
                    },
                    syn::PathSegment {
                        ident: syn::Ident::new("web", proc_macro2::Span::call_site()),
                        arguments: syn::PathArguments::None,
                    },
                    syn::PathSegment {
                        ident: syn::Ident::new("Data", proc_macro2::Span::call_site()),
                        arguments: syn::PathArguments::Parenthesized(
                            syn::ParenthesizedGenericArguments {
                                output: syn::ReturnType::Default,
                                paren_token: syn::token::Paren::default(),
                                inputs: vec![syn::Type::Path(syn::TypePath {
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: vec![syn::PathSegment {
                                            ident: syn::Ident::new(
                                                "Option",
                                                proc_macro2::Span::call_site(),
                                            ),
                                            arguments: syn::PathArguments::AngleBracketed(
                                                syn::AngleBracketedGenericArguments {
                                                    colon2_token: None,
                                                    lt_token: syn::token::Lt::default(),
                                                    gt_token: syn::token::Gt::default(),
                                                    args: vec![syn::GenericArgument::Type(
                                                        syn::Type::Path(syn::TypePath {
                                                            qself: None,
                                                            path: syn::Path {
                                                                leading_colon: None,
                                                                segments: vec![
                                            syn::PathSegment {
                                                ident: syn::Ident::new(
                                                    "ruxt",
                                                    proc_macro2::Span::call_site(),
                                                ),
                                                arguments: syn::PathArguments::None,
                                            },
                                            syn::PathSegment {
                                                ident: syn::Ident::new(
                                                    "Layout",
                                                    proc_macro2::Span::call_site(),
                                                ),
                                                arguments: syn::PathArguments::None,
                                            },
                                        ]
                                                                .into_iter()
                                                                .collect(),
                                                            },
                                                        }),
                                                    )]
                                                    .into_iter()
                                                    .collect(),
                                                },
                                            ),
                                        }]
                                        .into_iter()
                                        .collect(),
                                    },
                                })]
                                .into_iter()
                                .collect(),
                            },
                        ),
                    },
                ]
                .into_iter()
                .collect(),
            },
        })),
    })
}
