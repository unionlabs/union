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
use syn::{parse_quote, GenericArgument, Ident, ItemFn, ItemStruct, PathArguments, Type};

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

    let mk_struct = |module: Address, mod_name: &Ident, t: &MoveStruct| {
        let ident = format_ident!("{}", t.name.to_string());
        println!("{ident}");

        let generics = t
            .generic_type_params
            .iter()
            .enumerate()
            .map(|(i, _)| format_ident!("T{i}"));

        t.fields
            .iter()
            .map(|f| {
                let ident = format_ident!("{}", f.name.to_string());
                // println!("  {ident}: {:?}", f.typ);

                move_type_to_rust_type(module, &f.typ).map(|(_param_ty, field_ty)| {
                    (
                        quote! {
                            pub #ident: #field_ty,
                        },
                        find_struct_types_in_type(&field_ty),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map::<(ItemStruct, Vec<_>), _>(|fs| {
                let (fs, rs): (Vec<_>, Vec<_>) = fs.into_iter().unzip();

                (
                    parse_quote! {
                        #[macros::model]
                        #[derive(::move_bindgen::TypeTagged)]
                        #[type_tag(module = #mod_name)]
                        pub struct #ident<#(#generics)*> {
                            #(#fs)*
                        }
                    },
                    rs.into_iter().flatten().collect(),
                )
            })
    };

    // resolve events
    for abi in &abis {
        assert_eq!(abi.address, module);

        let mod_name = format_ident!("{}", abi.name.to_string());

        let events = abi
            .structs
            .iter()
            .filter(|t| t.is_event)
            .map(|t| mk_struct(module, &mod_name, t));

        for event in events {
            match event {
                Ok((event, rs)) => {
                    referenced_structs.extend(rs);
                    already_found_structs.insert((mod_name.clone(), event.ident.clone()));

                    mod_map
                        .entry(mod_name.clone())
                        .or_default()
                        .insert(event.ident.clone(), event);
                }
                Err(err) => {
                    eprintln!("{err}");
                }
            }
        }
    }

    let mut output_ts = HashMap::<_, TokenStream>::new();

    // resolve fns
    for abi in &abis {
        assert_eq!(abi.address, module);

        let mod_name = format_ident!("{}", abi.name.to_string());

        let (fns, rs): (Vec<_>, Vec<_>) = abi
            .exposed_functions
            .iter()
            .filter(|f| {
                matches!(f.visibility, MoveFunctionVisibility::Public) && (f.is_entry || f.is_view)
            })
            .map(|f| {
                let ident = format_ident!("{}", f.name.to_string());
                println!("======= {ident}");

                let (ret, ret_rs): (Vec<_>, Vec<_>) = f
                    .return_
                    .iter()
                    .map(|r| move_type_to_rust_type(module, r).map(|(param_ty, field_ty)| (param_ty, find_struct_types_in_type(&field_ty))))
                    .collect::<Result<Vec<_>, _>>()?.into_iter().unzip();

                f.params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        move_type_to_rust_type(module, p).map(|(param, field)| {
                            let ident = format_ident!("_{i}");
                            let rs = find_struct_types_in_type(&field);

                            // dbg!(&rs);

                            (p, (ident, ((param, field), rs)))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(|ps| {
                        #[allow(clippy::type_complexity)] // all hail our lord and saviour `unzip`
                        let (mts, (idents, ((param_tys, field_tys), rs))): (Vec<_>, (Vec<_>, ((Vec<_>, Vec<_>), Vec<_>))) = ps.into_iter().unzip();

                        let mts_ts = mts.iter().map(|typ| move_type_to_type_literal(typ));

                        let raw_ret = quote!((#(#ret,)*));

                        let (ret, ret_expr) = if ret.len() == 1 {
                            let ret = &ret[0];
                            (quote!(#ret), quote!(ret.0))
                        } else {
                            (quote!((#(#ret,)*)), quote!(ret))
                        };

                        let params = if param_tys.is_empty() {
                            quote!()
                        } else {
                            quote!((#(#idents,)*): (#(#param_tys,)*),)
                        };

                        let ts = if f.is_view {
                            parse_quote! {
                                async fn #ident(
                                    &self,
                                    contract_address: ::aptos_types::account_address::AccountAddress,
                                    #params
                                    ledger_version: Option<u64>
                                ) -> ::core::result::Result<#ret, ::aptos_rest_client::error::RestError>
                                {
                                    let response = self
                                        .client()
                                        .view(
                                            &::aptos_rest_client::aptos_api_types::ViewRequest {
                                                function: ::aptos_rest_client::aptos_api_types::EntryFunctionId {
                                                    module:
                                                        ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                                            address: contract_address.into(),
                                                            name: stringify!(#mod_name).parse().unwrap(),
                                                        },
                                                    name: stringify!(#ident).parse().unwrap(),
                                                },
                                                type_arguments: vec![#(#mts_ts,)*],
                                                arguments: vec![#(serde_json::to_value(#field_tys::from(#idents)).unwrap(),)*],
                                            },
                                            ledger_version,
                                        )
                                        .await?
                                        .into_inner();

                                    let ret = ::serde_json::from_value::<#raw_ret>(::serde_json::Value::from(response))?;

                                    Ok(#ret_expr)
                                }
                            }
                        } else {
                            let generic_type_params = f.generic_type_params
                                .iter().enumerate()
                                .map(|(index, _)| format_ident!("T{index}"));

                            let params = if param_tys.is_empty() {
                                quote!()
                            } else {
                                quote!((#(#idents,)*): (#(impl ::move_bindgen::IntoTypeTagged<#param_tys>,)*),)
                            };

                            parse_quote! {
                                fn #ident<
                                    #(#generic_type_params: ::serde::Serialize + ::move_bindgen::TypeTagged,)*
                                >(&self, contract_address: ::aptos_types::account_address::AccountAddress, #params) -> ::aptos_types::transaction::EntryFunction
                                {
                                    // let (values, type_args): (Vec<_>, Vec<_>) = vec![#({
                                    //     let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(#idents);
                                    //     (
                                    //         bcs::to_bytes(&t).unwrap(),
                                    //         <#param_tys as ::move_bindgen::TypeTagged>::type_tag(ctx),
                                    //     )
                                    // },)*].into_iter().unzip();

                                    ::aptos_types::transaction::EntryFunction::new(
                                        ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                            address: contract_address.into(),
                                            name: stringify!(#mod_name).parse().unwrap(),
                                        }.into(),
                                        stringify!(#ident).parse().unwrap(),
                                        // TODO: We don't use this currently but this should be fixed somehow(?)
                                        vec![],
                                        vec![#(bcs::to_bytes(&::move_bindgen::IntoTypeTagged::into_type_tagged(#idents).0).unwrap(),)*],
                                    )
                                }
                            }
                        };

                        (ts, ret_rs.into_iter().chain(rs))
                    })
            })
            .collect::<Result<Vec<(ItemFn, _)>, _>>()?
            .into_iter()
            .unzip();

        referenced_structs.extend(rs.into_iter().flatten().flatten());

        // dbg!(&mod_map);

        if !fns.is_empty() {
            output_ts.entry(mod_name).or_default().extend(quote! {
                pub trait ClientExt {
                    fn client(&self) -> &::aptos_rest_client::Client;

                    #(#fns)*
                }
            });
        };
    }

    let mut output = String::new();

    output += "#![allow(async_fn_in_trait,
non_snake_case,
clippy::useless_conversion,
clippy::unused_unit,
clippy::too_many_arguments)]
";

    // resolve types referenced in events and fns
    while !referenced_structs.is_empty() {
        // dbg!(&referenced_structs, &already_found_structs);

        for abi in &abis {
            let mod_name = format_ident!("{}", abi.name.to_string());
            println!("{mod_name}");

            let structs = abi
                .structs
                .iter()
                .filter(|t| {
                    let key = (mod_name.clone(), format_ident!("{}", t.name.to_string()));
                    // not an event, referenced, but not already found
                    !t.is_event
                        && referenced_structs.contains(&key)
                        && !already_found_structs.contains(&key)
                })
                .map(|t| mk_struct(module, &mod_name, t))
                .collect::<Vec<_>>();

            for s in structs {
                match s {
                    Ok((s, rs)) => {
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
                    Err(err) => {
                        eprintln!("{err}");
                    }
                }
            }
        }
    }

    for (k, v) in mod_map {
        let structs = v.into_iter().map(|m| {
            // dbg!(m.keys());

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

fn find_struct_types_in_type(ty: &Type) -> Vec<(Ident, Ident)> {
    match ty {
        Type::Path(ty) => {
            if ty.path.segments.first().unwrap().ident == "super" {
                vec![(
                    ty.path.segments.get(1).unwrap().ident.clone(),
                    ty.path.segments.get(2).unwrap().ident.clone(),
                )]
            } else if let PathArguments::AngleBracketed(args) =
                &ty.path.segments.last().unwrap().arguments
            {
                args.args
                    .iter()
                    .flat_map(|t| {
                        if let GenericArgument::Type(ty) = t {
                            find_struct_types_in_type(ty)
                        } else {
                            vec![]
                        }
                    })
                    .collect()
            } else {
                vec![]
            }
            // TODO: Recurse into the generics here to find all types
        }
        _ => vec![],
    }
}

/// (param type, field type)
fn move_type_to_rust_type(this_module: Address, typ: &MoveType) -> Result<(Type, Type), Bde> {
    let is_string = |mt: &MoveStructTag| {
        mt.address == "0x1".parse().unwrap()
            && mt.module == "string".parse().unwrap()
            && mt.name == "String".parse().unwrap()
            && mt.generic_type_params.is_empty()
    };

    Ok(match typ {
        MoveType::Bool => (parse_quote!(bool), parse_quote!(bool)),
        MoveType::U8 => (parse_quote!(u8), parse_quote!(u8)),
        MoveType::U16 => (
            parse_quote!(u16),
            parse_quote!(::aptos_rest_client::aptos_api_types::U16),
        ),
        MoveType::U32 => (
            parse_quote!(u32),
            parse_quote!(::aptos_rest_client::aptos_api_types::U32),
        ),
        MoveType::U64 => (
            parse_quote!(u64),
            parse_quote!(::aptos_rest_client::aptos_api_types::U64),
        ),
        MoveType::U128 => (
            parse_quote!(u128),
            parse_quote!(::aptos_rest_client::aptos_api_types::U128),
        ),
        MoveType::U256 => (
            parse_quote!(U256),
            parse_quote!(::aptos_rest_client::aptos_api_types::U256),
        ),
        MoveType::Address => (
            parse_quote!(::aptos_rest_client::aptos_api_types::Address),
            parse_quote!(::aptos_rest_client::aptos_api_types::Address),
        ),
        MoveType::Signer => (parse_quote!(Signer), parse_quote!(Signer)),
        MoveType::Vector { items } => {
            let (param, field) = move_type_to_rust_type(this_module, items)?;
            (parse_quote!(Vec<#param>), parse_quote!(Vec<#field>))
        }
        MoveType::Struct(s) if is_string(s) => (parse_quote!(String), parse_quote!(String)),
        // MoveType::Struct(s) if is_smart_table(&s) => {
        //     let t0 = move_type_to_rust_type(this_module, &s.generic_type_params[0])?;
        //     let t1 = move_type_to_rust_type(this_module, &s.generic_type_params[1])?;

        //     parse_quote!(SmartTable<#t0, #t1>)
        // }
        // MoveType::Struct(s) if is_table(&s) => {
        //     let t0 = move_type_to_rust_type(this_module, &s.generic_type_params[0])?;
        //     let t1 = move_type_to_rust_type(this_module, &s.generic_type_params[1])?;

        //     parse_quote!(Table<#t0, #t1>)
        // }
        MoveType::Struct(MoveStructTag {
            address,
            module,
            name,
            generic_type_params,
        }) => {
            if address != &this_module {
                return Err(format!("no clue where this is coming from ({address}::{module}::{name}<{generic_type_params:?}>)").into());
            }

            let module = format_ident!("{}", module.to_string());
            let ident = format_ident!("{}", name.to_string());
            let (param, field): (Vec<_>, Vec<_>) = generic_type_params
                .iter()
                .map(|m| move_type_to_rust_type(this_module, m))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .unzip();

            (
                parse_quote!(super::#module::#ident<#(#param,)*>),
                parse_quote!(super::#module::#ident<#(#field,)*>),
            )
        }
        MoveType::GenericTypeParam { index } => {
            let ident = format_ident!("T{index}");

            (parse_quote!(#ident), parse_quote!(#ident))
        }
        MoveType::Reference { mutable: _, to } => move_type_to_rust_type(this_module, to)?,
        MoveType::Unparsable(_) => todo!(),
    })
}

fn move_type_to_type_literal(typ: &MoveType) -> proc_macro2::TokenStream {
    match typ {
        MoveType::Bool => quote!(::aptos_rest_client::aptos_api_types::MoveType::Bool),
        MoveType::U8 => quote!(::aptos_rest_client::aptos_api_types::MoveType::U8),
        MoveType::U16 => quote!(::aptos_rest_client::aptos_api_types::MoveType::U16),
        MoveType::U32 => quote!(::aptos_rest_client::aptos_api_types::MoveType::U32),
        MoveType::U64 => quote!(::aptos_rest_client::aptos_api_types::MoveType::U64),
        MoveType::U128 => quote!(::aptos_rest_client::aptos_api_types::MoveType::U128),
        MoveType::U256 => quote!(::aptos_rest_client::aptos_api_types::MoveType::U256),
        MoveType::Address => quote!(::aptos_rest_client::aptos_api_types::MoveType::Address),
        MoveType::Signer => quote!(::aptos_rest_client::aptos_api_types::MoveType::Signer),
        MoveType::Vector { items } => {
            let items = move_type_to_type_literal(items);

            quote!(::aptos_rest_client::aptos_api_types::MoveType::Vector { items: #items })
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
            let generic_type_params = generic_type_params.iter().map(move_type_to_type_literal);

            quote!(::aptos_rest_client::aptos_api_types::MoveType::Struct(::aptos_rest_client::aptos_api_types::MoveStructTag {
                address: #address.parse().unwrap(),
                module: #module.parse().unwrap(),
                name: #name.parse().unwrap(),
                generic_type_params: vec![#(#generic_type_params)*],
            }))
        }
        MoveType::GenericTypeParam { index } => {
            quote!(::aptos_rest_client::aptos_api_types::MoveType::GenericTypeParam { index: #index })
        }
        MoveType::Reference { mutable: _, to: _ } => todo!(),
        MoveType::Unparsable(_) => todo!(),
    }
}
