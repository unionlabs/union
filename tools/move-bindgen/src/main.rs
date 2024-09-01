use std::{collections::HashMap, error::Error, fmt::Write};

use aptos_rest_client::{
    aptos_api_types::{Address, MoveFunctionVisibility, MoveStructTag, MoveType},
    Client,
};
use quote::{format_ident, quote};
use syn::{parse_quote, ItemFn, ItemStruct};

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

    // resolve types
    for abi in &abis {
        assert_eq!(abi.address, module);

        let mod_name = format_ident!("{}", abi.name.to_string());

        let structs = abi
            .structs
            .iter()
            // .filter(|t| futures::future::ready(t.is_event && !t.is_native))
            .map(|t| {
                let ident = format_ident!("{}", t.name.to_string());
                // println!("{ident}");

                t.fields
                    .iter()
                    .map(|f| {
                        let ident = format_ident!("{}", f.name.to_string());
                        // println!("  {ident}: {:?}", f.typ);

                        move_type_to_rust_type(module, &f.typ).map(|(_param_ty, field_ty)| {
                            quote! {
                                #ident: #field_ty,
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(|fs| {
                        parse_quote! {
                            #[macros::model]
                            #[derive(::move_bindgen::TypeTagged)]
                            #[type_tag(module = #mod_name)]
                            pub struct #ident {
                                #(#fs)*
                            }
                        }
                    })
            })
            .collect::<Result<Vec<ItemStruct>, _>>();

        match structs {
            Ok(ok) => mod_map
                .entry(mod_name.clone())
                .or_default()
                .extend(ok.into_iter().map(|s| (s.ident.clone(), s))),
            Err(err) => {
                eprintln!("{err}")
            }
        }
    }

    let mut output = String::new();

    output += "#![allow(async_fn_in_trait,
non_snake_case,
clippy::useless_conversion,
clippy::unused_unit,
clippy::too_many_arguments)]
";

    for abi in abis {
        assert_eq!(abi.address, module);

        let mod_name = format_ident!("{}", abi.name.to_string());

        let fns = abi
            .exposed_functions
            .iter()
            .filter(|f| {
                matches!(f.visibility, MoveFunctionVisibility::Public) && (f.is_entry || f.is_view)
            })
            .map(|f| {
                let ident = format_ident!("{}", f.name.to_string());
                // println!("{ident}");

                let ret = f
                    .return_
                    .iter()
                    .map(|r| move_type_to_rust_type(module, r).map(|(param_ty, _field_ty)|param_ty))
                    .collect::<Result<Vec<_>, _>>()?;

                f.params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        move_type_to_rust_type(module, p).map(|ty| {
                            let ident = format_ident!("_{i}");
                            (p, (ident, ty))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(|ps| {
                        let (mts, (idents, (param_tys, field_tys))): (Vec<_>, (Vec<_>, (Vec<_>, Vec<_>))) = ps.into_iter().unzip();

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

                        if f.is_view {
                            parse_quote! {
                                async fn #ident(
                                    &self,
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
                                                            address: self.module_address().into(),
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
                                >(&self, #params) -> ::aptos_types::transaction::EntryFunction
                                {
                                    let (values, type_args): (Vec<_>, Vec<_>) = vec![#({
                                        let (t, ctx) = ::move_bindgen::IntoTypeTagged::into_type_tagged(#idents);
                                        (
                                            bcs::to_bytes(&t).unwrap(),
                                            <#param_tys as ::move_bindgen::TypeTagged>::type_tag(ctx),
                                        )
                                    },)*].into_iter().unzip();

                                    ::aptos_types::transaction::EntryFunction::new(
                                        ::aptos_rest_client::aptos_api_types::MoveModuleId {
                                            address: self.module_address().into(),
                                            name: stringify!(#mod_name).parse().unwrap(),
                                        }.into(),
                                        stringify!(#ident).parse().unwrap(),
                                        type_args,
                                        values,
                                    )
                                }
                            }
                        }
                    })
            })
            .collect::<Result<Vec<ItemFn>, _>>()?;

        let structs = mod_map.get(&mod_name).into_iter().flat_map(|m| m.values());

        let client_ext_trait = if fns.is_empty() {
            quote!()
        } else {
            quote! {
                pub trait ClientExt {
                    fn client(&self) -> &::aptos_rest_client::Client;
                    fn module_address(&self) -> ::aptos_types::account_address::AccountAddress;

                    #(#fns)*
                }
            }
        };

        writeln!(&mut output).unwrap();

        output += &prettyplease::unparse(&syn::parse_quote! {
            pub mod #mod_name {
                #client_ext_trait

                #(#structs)*
            }
        });
    }

    std::fs::write(outfile, &output)?;

    Ok(())
}

/// (param type, field type)
fn move_type_to_rust_type(
    this_module: Address,
    typ: &MoveType,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), Bde> {
    let is_string = |mt: &MoveStructTag| {
        mt.address == "0x1".parse().unwrap()
            && mt.module == "string".parse().unwrap()
            && mt.name == "String".parse().unwrap()
            && mt.generic_type_params.is_empty()
    };

    Ok(match typ {
        MoveType::Bool => (quote!(bool), quote!(bool)),
        MoveType::U8 => (quote!(u8), quote!(u8)),
        MoveType::U16 => (
            quote!(u16),
            quote!(::aptos_rest_client::aptos_api_types::U16),
        ),
        MoveType::U32 => (
            quote!(u32),
            quote!(::aptos_rest_client::aptos_api_types::U32),
        ),
        MoveType::U64 => (
            quote!(u64),
            quote!(::aptos_rest_client::aptos_api_types::U64),
        ),
        MoveType::U128 => (
            quote!(u128),
            quote!(::aptos_rest_client::aptos_api_types::U128),
        ),
        MoveType::U256 => (
            quote!(U256),
            quote!(::aptos_rest_client::aptos_api_types::U256),
        ),
        MoveType::Address => (
            quote!(::aptos_rest_client::aptos_api_types::Address),
            quote!(::aptos_rest_client::aptos_api_types::Address),
        ),
        MoveType::Signer => (quote!(Signer), quote!(Signer)),
        MoveType::Vector { items } => {
            let (param, field) = move_type_to_rust_type(this_module, items)?;
            (quote!(Vec<#param>), quote!(Vec<#field>))
        }
        MoveType::Struct(s) if is_string(s) => (quote!(String), quote!(String)),
        // MoveType::Struct(s) if is_smart_table(&s) => {
        //     let t0 = move_type_to_rust_type(this_module, &s.generic_type_params[0])?;
        //     let t1 = move_type_to_rust_type(this_module, &s.generic_type_params[1])?;

        //     quote!(SmartTable<#t0, #t1>)
        // }
        // MoveType::Struct(s) if is_table(&s) => {
        //     let t0 = move_type_to_rust_type(this_module, &s.generic_type_params[0])?;
        //     let t1 = move_type_to_rust_type(this_module, &s.generic_type_params[1])?;

        //     quote!(Table<#t0, #t1>)
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
                quote!(super::#module::#ident<#(#param,)*>),
                quote!(super::#module::#ident<#(#field,)*>),
            )
        }
        MoveType::GenericTypeParam { index } => {
            let ident = format_ident!("T{index}");

            (quote!(#ident), quote!(#ident))
        }
        MoveType::Reference { mutable, to } => move_type_to_rust_type(this_module, to)?,
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
        MoveType::Reference { mutable, to } => todo!(),
        MoveType::Unparsable(_) => todo!(),
    }
}
