//! buon appetito, hope you like italian
//!
//! cuz this is pure spaghetti

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Write,
};

use aptos_rest_client::{
    aptos_api_types::{Address, MoveFunctionVisibility, MoveStruct, MoveStructTag, MoveType},
    Client,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Ident, ItemFn, ItemStruct, Type};

type Bde = Box<dyn Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Bde> {
    let [base_url, module, outfile] = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|v| format!("expected `[base_url, module, outfile]`, but found `{v:?}`"))?;

    let module: Address = module.parse()?;

    let client = Client::new(base_url.parse()?);

    let abis = client
        .get_account_modules(module.into())
        .await?
        .into_inner()
        .into_iter()
        .flat_map(|m| m.try_parse_abi().unwrap().abi)
        .collect::<Vec<_>>();

    let mut mod_map = HashMap::<_, HashMap<_, _>>::new();

    let mut referenced_structs = HashSet::new();

    let mut already_found_structs = HashSet::new();

    let mk_struct = |_mod_name: &Ident, t: &MoveStruct| -> (ItemStruct, Vec<_>) {
        let ident = format_ident!("{}", t.name.to_string());
        println!("{ident}");

        let generics = t
            .generic_type_params
            .iter()
            .enumerate()
            .map(|(i, _)| format_ident!("T{i}"));

        let (fields, contained_struct_types): (Vec<_>, Vec<_>) = t
            .fields
            .iter()
            .map(|f| {
                let ident = syn::parse_str::<Ident>(&format!("{}", f.name))
                    .unwrap_or_else(|_| format_ident!("r#{}", f.name.to_string()));
                // println!("  {ident}: {:?}", f.typ);

                let (field_ty, contained_struct_types) = move_type_to_output_type(&f.typ);

                (
                    quote! {
                        pub #ident: #field_ty,
                    },
                    contained_struct_types,
                )
            })
            .unzip();

        (
            parse_quote! {
                #[derive(
                    Debug,
                    Clone,
                    PartialEq,
                    Eq,
                    PartialOrd,
                    Ord,
                    ::move_bindgen::serde::Serialize,
                    ::move_bindgen::serde::Deserialize,
                    ::move_bindgen::MoveOutputType,
                )]
                #[serde(crate = "::move_bindgen::serde")]
                // #[move_output_type(module = #mod_name)]
                pub struct #ident<#(#generics)*> {
                    #(#fields)*
                }
            },
            contained_struct_types.into_iter().flatten().collect(),
        )
    };

    // resolve events
    for abi in &abis {
        assert_eq!(abi.address, module);

        let mod_name = format_ident!("{}", abi.name.to_string());

        let events = abi
            .structs
            .iter()
            .filter(|t| t.is_event)
            .map(|t| mk_struct(&mod_name, t));

        for (event, rs) in events {
            referenced_structs.extend(rs);
            already_found_structs.insert((mod_name.clone(), event.ident.clone()));

            mod_map
                .entry(mod_name.clone())
                .or_default()
                .insert(event.ident.clone(), event);
        }
    }

    let mut output_ts = HashMap::<_, TokenStream>::new();

    // resolve fns
    for abi in &abis {
        assert_eq!(abi.address, module);

        let mod_name = format_ident!("{}", abi.name.to_string());

        let (methods, new_referenced_structs): (Vec<_>, Vec<_>) = abi
            .exposed_functions
            .iter()
            .filter(|f| {
                matches!(f.visibility, MoveFunctionVisibility::Public) && (f.is_entry || f.is_view)
            })
            .map(|f| -> _ {
                let ident = format_ident!("{}", f.name.to_string());
                println!("found function {ident}");

                let (return_type, return_type_referenced_structs): (Vec<_>, Vec<_>) = f
                    .return_
                    .iter()
                    .map(move_type_to_output_type)
                    .collect::<Vec<_>>().into_iter().unzip();

                let params = f.params.iter()
                    .filter(|move_type| !is_signer(move_type)).collect::<Vec<_>>();

                let param_types_referenced_structs = params.iter().flat_map(|ty| move_type_to_output_type(ty).1);

                let (idents, param_types): (Vec<_>, Vec<_>) = params
                    .iter()
                    .filter(|move_type| !is_signer(move_type))
                    .enumerate()
                    .map(|(i, p)| {
                        (format_ident!("_{i}"), move_type_to_param_type(p))
                    }).unzip();

                let raw_ret = quote!((#(<#return_type as ::move_bindgen::MoveOutputType>::Raw,)*));

                let raw_return_value_idents = return_type.iter().enumerate().map(|(i, _)| format_ident!("ret_{i}")).collect::<Vec<_>>();

                // unwrap output tuple if there's only one output value
                let (return_type, return_expr) = if return_type.len() == 1 {
                    let ret = &return_type[0];
                    (
                        quote!(#ret),
                        quote!((#(<#return_type as ::move_bindgen::MoveOutputType>::from_raw(ret_0))*,) .0)
                    )
                } else {
                    (
                        quote!((#(#return_type,)*)),
                        quote!((#(
                            <#return_type as ::move_bindgen::MoveOutputType>::from_raw(#raw_return_value_idents),
                        )*))
                    )
                };

                let params = if param_types.is_empty() {
                    quote!()
                } else {
                    quote!((#(#idents,)*): (#(#param_types,)*),)
                };

                let generic_type_params = f.generic_type_params
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format_ident!("t{i}"));

                let generic_type_params_args = if f.generic_type_params.is_empty() {
                    quote!()
                } else {
                    let params = f.generic_type_params.iter().map(|_| quote!(impl Into<::move_bindgen::move_core_types::language_storage::TypeTag>));
                    let generic_type_params = generic_type_params.clone();
                    quote!((#(#generic_type_params,)*): (#(#params,)*),)
                };

                let tracing_instrument_fields = if param_types.is_empty() {
                    quote!()
                } else {
                    quote!(#(?#idents,)*)
                };

                let item_fn: ItemFn = if f.is_view {
                    parse_quote! {
                        #[::move_bindgen::tracing::instrument(
                            skip_all,
                            fields(
                                %contract_address,
                                ?ledger_version,
                                #tracing_instrument_fields,
                            )
                        )]
                        async fn #ident(
                            &self,
                            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
                            ledger_version: Option<u64>,
                            #params
                            #generic_type_params_args
                        ) -> ::core::result::Result<#return_type, ::move_bindgen::aptos_rest_client::error::RestError>
                        {
                            let response = self
                                .client()
                                .view(
                                    &::move_bindgen::aptos_rest_client::aptos_api_types::ViewRequest {
                                        function: ::move_bindgen::aptos_rest_client::aptos_api_types::EntryFunctionId {
                                            module:
                                                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                                    address: contract_address.into(),
                                                    name: stringify!(#mod_name).parse().unwrap(),
                                                },
                                            name: stringify!(#ident).parse().unwrap(),
                                        },
                                        type_arguments: vec![
                                            #(#generic_type_params.into().into()),*
                                        ],
                                        arguments: vec![#(
                                            ::move_bindgen::serde_json::to_value(
                                                &<#param_types as ::move_bindgen::MoveOutputType>::into_raw(
                                                    #idents
                                                )
                                            )
                                            .unwrap(),
                                        )*],
                                    },
                                    ledger_version,
                                )
                                .await?
                                .into_inner();

                            let value = ::move_bindgen::serde_json::Value::from(response);

                            ::move_bindgen::tracing::debug!(%value, "fetched response");

                            let (#(#raw_return_value_idents,)*) = ::move_bindgen::serde_json::from_value::<#raw_ret>(value)?;

                            Ok(#return_expr)
                        }
                    }
                } else {
                    parse_quote! {
                        fn #ident(
                            &self,
                            contract_address: ::move_bindgen::aptos_types::account_address::AccountAddress,
                            #params
                            #generic_type_params_args
                        ) -> ::move_bindgen::aptos_types::transaction::EntryFunction
                        {
                            ::move_bindgen::aptos_types::transaction::EntryFunction::new(
                                ::move_bindgen::aptos_rest_client::aptos_api_types::MoveModuleId {
                                    address: contract_address.into(),
                                    name: stringify!(#mod_name).parse().unwrap(),
                                }.into(),
                                stringify!(#ident).parse().unwrap(),
                                vec![#(#generic_type_params.into().into()),*],
                                vec![#(::move_bindgen::bcs::to_bytes(&#idents).unwrap(),)*],
                            )
                        }
                    }
                };

                (
                    item_fn,
                    return_type_referenced_structs
                        .into_iter()
                        .flatten()
                        .chain(param_types_referenced_structs)
                        .collect::<Vec<_>>()
                )
            })
            .unzip();

        referenced_structs.extend(new_referenced_structs.into_iter().flatten());

        // dbg!(&mod_map);

        if !methods.is_empty() {
            output_ts.entry(mod_name).or_default().extend(quote! {
                pub trait ClientExt {
                    fn client(&self) -> &::move_bindgen::aptos_rest_client::Client;

                    #(#methods)*
                }
            });
        };
    }

    let mut output = String::new();

    output += "
#![allow(
    async_fn_in_trait,
    non_snake_case,
    clippy::type_complexity,
    clippy::needless_borrows_for_generic_args,
    clippy::useless_conversion,
    clippy::unused_unit,
    clippy::too_many_arguments
)]
";

    dbg!(&referenced_structs);

    // panic!();

    // resolve types referenced in events and fns
    while !referenced_structs.is_empty() {
        dbg!(&referenced_structs, &already_found_structs);

        for abi in &abis {
            let mod_name = format_ident!("{}", abi.name.to_string());
            println!("{mod_name}");

            let struct_results = abi
                .structs
                .iter()
                .filter(|t| {
                    // dbg!(&t);
                    let key = (mod_name.clone(), format_ident!("{}", t.name.to_string()));
                    // not an event, referenced, but not already found
                    !t.is_event
                        && referenced_structs.contains(&key)
                        && !already_found_structs.contains(&key)
                })
                .map(|t| mk_struct(&mod_name, t))
                .collect::<Vec<_>>();

            for (s, rs) in struct_results {
                let key = (mod_name.clone(), s.ident.clone());
                referenced_structs.remove(&key);
                already_found_structs.insert(key);

                // dbg!(&rs);
                for r in rs {
                    if !already_found_structs.contains(&r) {
                        referenced_structs.insert(r);
                    }
                }

                mod_map
                    .entry(mod_name.clone())
                    .or_default()
                    .insert(s.ident.clone(), s);
            }
        }
    }

    for (k, v) in mod_map {
        let structs = v.into_iter().map(|m| {
            dbg!(&m);

            m.1
        });

        output_ts.entry(k).or_default().extend(quote!(#(#structs)*));
    }

    for (k, v) in output_ts {
        writeln!(&mut output).unwrap();

        output += &prettyplease::unparse(&syn::parse_quote! {
            pub mod #k {
                #v
            }
        });
    }

    std::fs::write(outfile, &output)?;

    Ok(())
}

// fn find_struct_types_in_type(ty: &Type) -> Vec<(Ident, Ident)> {
//     match ty {
//         Type::Path(ty) => {
//             if ty.path.segments.first().unwrap().ident == "super" {
//                 vec![(
//                     ty.path.segments.get(1).unwrap().ident.clone(),
//                     ty.path.segments.get(2).unwrap().ident.clone(),
//                 )]
//             } else if let PathArguments::AngleBracketed(args) =
//                 &ty.path.segments.last().unwrap().arguments
//             {
//                 args.args
//                     .iter()
//                     .flat_map(|t| {
//                         if let GenericArgument::Type(ty) = t {
//                             find_struct_types_in_type(ty)
//                         } else {
//                             vec![]
//                         }
//                     })
//                     .collect()
//             } else {
//                 vec![]
//             }
//             // TODO: Recurse into the generics here to find all types
//         }
//         _ => vec![],
//     }
// }

// /// (param type, field type)
// fn move_type_to_rust_type(this_module: Address, typ: &MoveType) -> Result<(Type, Type), Bde> {
//     let is_string = |mt: &MoveStructTag| {
//         mt.address == "0x1".parse().unwrap()
//             && mt.module == "string".parse().unwrap()
//             && mt.name == "String".parse().unwrap()
//             && mt.generic_type_params.is_empty()
//     };

//     let is_option = |mt: &MoveStructTag| {
//         mt.address == "0x1".parse().unwrap()
//             && mt.module == "option".parse().unwrap()
//             && mt.name == "Option".parse().unwrap()
//             && mt.generic_type_params.len() == 1
//     };

//     let is_simple_map = |mt: &MoveStructTag| {
//         mt.address == "0x1".parse().unwrap()
//             && mt.module == "simple_map".parse().unwrap()
//             && mt.name == "SimpleMap".parse().unwrap()
//             && mt.generic_type_params.len() == 2
//     };

//     Ok(match typ {
//         MoveType::Bool => (parse_quote!(bool), parse_quote!(bool)),
//         MoveType::U8 => (parse_quote!(u8), parse_quote!(u8)),
//         MoveType::U16 => (
//             parse_quote!(u16),
//             parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::U16),
//         ),
//         MoveType::U32 => (
//             parse_quote!(u32),
//             parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::U32),
//         ),
//         MoveType::U64 => (
//             parse_quote!(u64),
//             parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::U64),
//         ),
//         MoveType::U128 => (
//             parse_quote!(u128),
//             parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::U128),
//         ),
//         MoveType::U256 => (
//             parse_quote!(U256),
//             parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::U256),
//         ),
//         MoveType::Address => (
//             parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::Address),
//             parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::Address),
//         ),
//         MoveType::Signer => (parse_quote!(Signer), parse_quote!(Signer)),
//         MoveType::Vector { items } => {
//             if **items == MoveType::U8 {
//                 let (_param, field) = move_type_to_rust_type(this_module, items)?;
//                 (
//                     parse_quote!(
//                         ::move_bindgen::aptos_rest_client::aptos_api_types::HexEncodedBytes
//                     ),
//                     parse_quote!(Vec<u8>),
//                 )
//             } else {
//                 let (param, field) = move_type_to_rust_type(this_module, items)?;
//                 (parse_quote!(Vec<#param>), parse_quote!(Vec<#field>))
//             }
//         }
//         MoveType::Struct(s) if is_string(s) => (parse_quote!(String), parse_quote!(String)),
//         MoveType::Struct(s) if is_option(s) => {
//             let (param, field) = move_type_to_rust_type(this_module, &s.generic_type_params[0])?;
//             (
//                 parse_quote!(::move_bindgen::MoveOption<#param>),
//                 parse_quote!(Option<#field>),
//             )
//         }
//         MoveType::Struct(s) if is_simple_map(s) => {
//             let (key_param, key_field) =
//                 move_type_to_rust_type(this_module, &s.generic_type_params[1])?;
//             let (value_param, value_field) =
//                 move_type_to_rust_type(this_module, &s.generic_type_params[1])?;
//             (
//                 parse_quote!(::move_bindgen::SimpleMap<#key_param, #value_param>),
//                 parse_quote!(::std::collections::BTreeMap<#key_field, #value_field>),
//             )
//         }
//         // MoveType::Struct(s) if is_smart_table(&s) => {
//         //     let t0 = move_type_to_rust_type(this_module, &s.generic_type_params[0])?;
//         //     let t1 = move_type_to_rust_type(this_module, &s.generic_type_params[1])?;

//         //     parse_quote!(SmartTable<#t0, #t1>)
//         // }
//         // MoveType::Struct(s) if is_table(&s) => {
//         //     let t0 = move_type_to_rust_type(this_module, &s.generic_type_params[0])?;
//         //     let t1 = move_type_to_rust_type(this_module, &s.generic_type_params[1])?;

//         //     parse_quote!(Table<#t0, #t1>)
//         // }
//         MoveType::Struct(MoveStructTag {
//             address,
//             module,
//             name,
//             generic_type_params,
//         }) => {
//             if address != &this_module {
//                 return Err(format!("no clue where this is coming from ({address}::{module}::{name}<{generic_type_params:?}>)").into());
//             }

//             let module = format_ident!("{}", module.to_string());
//             let ident = format_ident!("{}", name.to_string());
//             let (param, field): (Vec<_>, Vec<_>) = generic_type_params
//                 .iter()
//                 .map(|m| move_type_to_rust_type(this_module, m))
//                 .collect::<Result<Vec<_>, _>>()?
//                 .into_iter()
//                 .unzip();

//             (
//                 parse_quote!(super::#module::#ident<#(#param,)*>),
//                 parse_quote!(super::#module::#ident<#(#field,)*>),
//             )
//         }
//         MoveType::GenericTypeParam { index } => {
//             let ident = format_ident!("T{index}");

//             (parse_quote!(#ident), parse_quote!(#ident))
//         }
//         MoveType::Reference { mutable: _, to } => move_type_to_rust_type(this_module, to)?,
//         MoveType::Unparsable(_) => todo!(),
//     })
// }

#[track_caller]
fn move_type_to_param_type(typ: &MoveType) -> Type {
    match typ {
        MoveType::Bool => parse_quote!(bool),
        MoveType::U8 => parse_quote!(u8),
        MoveType::U16 => parse_quote!(u16),
        MoveType::U32 => parse_quote!(u32),
        MoveType::U64 => parse_quote!(u64),
        MoveType::U128 => parse_quote!(u128),
        MoveType::Address => {
            parse_quote!(::move_bindgen::aptos_types::account_address::AccountAddress)
        }
        MoveType::Vector { items } => {
            let param = move_type_to_param_type(items);
            parse_quote!(Vec<#param>)
        }
        MoveType::Struct(s) if is_string(s) => parse_quote!(String),
        MoveType::Struct(s) if is_option(s) => {
            let t = move_type_to_param_type(&s.generic_type_params[0]);

            parse_quote!(Option<#t>)
        }
        MoveType::Struct(s) if is_type_info(s) => {
            parse_quote!(::move_bindgen::TypeInfo)
        }
        MoveType::Struct(s) if is_fungible_asset_metadata(s) => {
            parse_quote!(::move_bindgen::fungible_asset::Metadata)
        }
        MoveType::Struct(s) if is_object_object(s) => {
            let t = move_type_to_param_type(&s.generic_type_params[0]);

            parse_quote!(::move_bindgen::object::Object<#t>)
        }
        MoveType::GenericTypeParam { index } => {
            let ident = format_ident!("T{index}");
            parse_quote!(#ident)
        }
        typ => panic!("unsupported param type: {typ:?}"),
    }
}

// (type, contained struct type identifiers)
#[track_caller]
fn move_type_to_output_type(typ: &MoveType) -> (Type, Vec<(Ident, Ident)>) {
    match typ {
        MoveType::Bool => (parse_quote!(bool), vec![]),
        MoveType::U8 => (parse_quote!(u8), vec![]),
        MoveType::U16 => (parse_quote!(u16), vec![]),
        MoveType::U32 => (parse_quote!(u32), vec![]),
        MoveType::U64 => (parse_quote!(u64), vec![]),
        MoveType::U128 => (parse_quote!(u128), vec![]),
        MoveType::Address => (
            parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::Address),
            vec![],
        ),
        MoveType::Vector { items } => {
            let (items, contained_struct_types) = move_type_to_output_type(items);

            (parse_quote!(Vec<#items>), contained_struct_types)
        }
        MoveType::Struct(s) if is_string(s) => (parse_quote!(String), vec![]),
        MoveType::Struct(s) if is_option(s) => {
            let (param, contained_struct_types) =
                move_type_to_output_type(&s.generic_type_params[0]);

            (parse_quote!(Option<#param>), contained_struct_types)
        }
        MoveType::Struct(s) if is_type_info(s) => (parse_quote!(::move_bindgen::TypeInfo), vec![]),
        MoveType::Struct(s) if is_fungible_asset_metadata(s) => (
            parse_quote!(::move_bindgen::fungible_asset::Metadata),
            vec![],
        ),
        MoveType::Struct(s) if is_object_object(s) => {
            let (param, contained_struct_types) =
                move_type_to_output_type(&s.generic_type_params[0]);

            (
                parse_quote!(::move_bindgen::object::Object<#param>),
                contained_struct_types,
            )
        }
        MoveType::Struct(s) if is_simple_map(s) => {
            let (key, key_contained_struct_types) =
                move_type_to_output_type(&s.generic_type_params[0]);
            let (value, value_contained_struct_types) =
                move_type_to_output_type(&s.generic_type_params[1]);

            (
                parse_quote!(::std::collections::BTreeMap<#key, #value>),
                key_contained_struct_types
                    .into_iter()
                    .chain(value_contained_struct_types)
                    .collect(),
            )
        }
        MoveType::Struct(MoveStructTag {
            address: _,
            module,
            name,
            generic_type_params,
        }) => {
            let module = format_ident!("{}", module.to_string());
            let name = format_ident!("{}", name.to_string());
            let (generic_type_params, struct_type_identifiers): (Vec<_>, Vec<_>) =
                generic_type_params
                    .iter()
                    .map(move_type_to_output_type)
                    .unzip();

            (
                parse_quote!(super::#module::#name<#(#generic_type_params)*>),
                [(module, name)]
                    .into_iter()
                    .chain(struct_type_identifiers.into_iter().flatten())
                    .collect(),
            )
        }
        MoveType::GenericTypeParam { index } => {
            let ident = format_ident!("T{index}");
            (parse_quote!(#ident), vec![])
        }
        // MoveType::GenericTypeParam { index } => {
        //     parse_quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::GenericTypeParam { index: #index })
        // }
        typ => panic!("unsupported output type: {typ:?}"),
    }
}

#[allow(dead_code)]
fn move_type_to_type_expression(typ: &MoveType) -> proc_macro2::TokenStream {
    match typ {
        MoveType::Bool => {
            quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::Bool)
        }
        MoveType::U8 => quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::U8),
        MoveType::U16 => quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::U16),
        MoveType::U32 => quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::U32),
        MoveType::U64 => quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::U64),
        MoveType::U128 => {
            quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::U128)
        }
        MoveType::U256 => {
            quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::U256)
        }
        MoveType::Address => {
            quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::Address)
        }
        MoveType::Signer => {
            quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::Signer)
        }
        MoveType::Vector { items } => {
            let items = move_type_to_type_expression(items);

            quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::Vector { items: #items })
        }
        MoveType::Struct(MoveStructTag {
            address,
            module,
            name,
            generic_type_params,
        }) => {
            let address = address.to_standard_string();
            let module = module.to_string();
            let name = name.to_string();
            let generic_type_params = generic_type_params.iter().map(move_type_to_type_expression);

            quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::Struct(::move_bindgen::aptos_rest_client::aptos_api_types::MoveStructTag {
                address: #address.parse().unwrap(),
                module: #module.parse().unwrap(),
                name: #name.parse().unwrap(),
                generic_type_params: vec![#(#generic_type_params)*],
            }))
        }
        MoveType::GenericTypeParam { index } => {
            quote!(::move_bindgen::aptos_rest_client::aptos_api_types::MoveType::GenericTypeParam { index: #index })
        }
        MoveType::Reference { mutable: _, to: _ } => todo!(),
        MoveType::Unparsable(_) => todo!(),
    }
}

fn is_signer(move_type: &MoveType) -> bool {
    match move_type {
        MoveType::Signer => true,
        MoveType::Reference { mutable: false, to } => is_signer(to),
        _ => false,
    }
}

fn is_string(mt: &MoveStructTag) -> bool {
    mt.address == "0x1".parse().unwrap()
        && mt.module == "string".parse().unwrap()
        && mt.name == "String".parse().unwrap()
        && mt.generic_type_params.is_empty()
}

fn is_option(mt: &MoveStructTag) -> bool {
    mt.address == "0x1".parse().unwrap()
        && mt.module == "option".parse().unwrap()
        && mt.name == "Option".parse().unwrap()
        && mt.generic_type_params.len() == 1
}

fn is_type_info(mt: &MoveStructTag) -> bool {
    mt.address == "0x1".parse().unwrap()
        && mt.module == "type_info".parse().unwrap()
        && mt.name == "TypeInfo".parse().unwrap()
        && mt.generic_type_params.is_empty()
}

fn is_fungible_asset_metadata(mt: &MoveStructTag) -> bool {
    mt.address == "0x1".parse().unwrap()
        && mt.module == "fungible_asset".parse().unwrap()
        && mt.name == "Metadata".parse().unwrap()
        && mt.generic_type_params.is_empty()
}

fn is_object_object(mt: &MoveStructTag) -> bool {
    mt.address == "0x1".parse().unwrap()
        && mt.module == "object".parse().unwrap()
        && mt.name == "Object".parse().unwrap()
        && mt.generic_type_params.len() == 1
}

fn is_simple_map(mt: &MoveStructTag) -> bool {
    mt.address == "0x1".parse().unwrap()
        && mt.module == "simple_map".parse().unwrap()
        && mt.name == "SimpleMap".parse().unwrap()
        && mt.generic_type_params.len() == 2
}
