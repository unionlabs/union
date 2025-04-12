use std::{collections::BTreeMap, env, fs, path::Path};

use heck::ToPascalCase;

fn main() {
    let path = "../../deployments/universal-chain-ids.json";

    println!("cargo:rerun-if-changed={path}");

    let universal_chain_ids =
        serde_json::from_str::<BTreeMap<String, Vec<String>>>(&fs::read_to_string(path).unwrap())
            .unwrap();

    let variants = universal_chain_ids
        .keys()
        .map(|family| {
            format!(
                r#"
                /// ```txt
                /// chain_family_name: {family}
                /// ```
                {},
                "#,
                family.to_pascal_case()
            )
        })
        .collect::<String>();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("out.rs");

    let enum_ = format!(
        r#"
        /// The `<chain_family_name>` portion of a universal chain id.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Family {{
            {variants}
        }}
        "#
    );

    let impls = format!(
        r#"
        impl core::fmt::Display for Family {{
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
                f.write_str(match self {{
                    {display}
                }})
            }}
        }}

        impl core::str::FromStr for Family {{
            type Err = UnknownFamily;

            fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {{
                match s {{
                    {from_str}
                    s => Err(UnknownFamily(s.into())),
                }}
            }}
        }}

        /// Error returned by [`Family::from_str`].
        #[derive(Debug, PartialEq, thiserror::Error)]
        #[error("unknown universal chain id family {{0:?}}")]
        pub struct UnknownFamily(pub alloc::string::String);
        "#,
        display = universal_chain_ids
            .keys()
            .map(|family| format!(r#"Self::{} => "{}","#, family.to_pascal_case(), family))
            .collect::<String>(),
        from_str = universal_chain_ids
            .keys()
            .map(|family| format!(
                r#""{}" => core::result::Result::Ok(Self::{}),"#,
                family,
                family.to_pascal_case()
            ))
            .collect::<String>()
    );

    let well_known = format!(
        r#"
        /// Well-known universal chain ids, as defined in [`universal-chain-ids.json`].
        ///
        /// [`universal-chain-ids.json`]: https://github.com/unionlabs/union/blob/main/deployments/universal-chain-ids.json
        pub mod well_known {{
            #![allow(clippy::enum_glob_use)]
            use super::{{UniversalChainId, Family::*, Id}};
            {}
        }}"#,
        universal_chain_ids
            .iter()
            .flat_map(
                |(family, chain_ids)| chain_ids.iter().map(move |chain_id| format!(
                    r#"
                    /// ```txt
                    /// chain_family_name: {family}
                    /// chain_id:          {chain_id}
                    /// ```
                    pub const {}_{}: UniversalChainId = UniversalChainId::new({}, Id::new("{}").unwrap());
                    "#,
                    family.to_uppercase(),
                    chain_id.to_uppercase().replace('-', "_"),
                    family.to_pascal_case(),
                    chain_id
                ))
            )
            .collect::<String>()
    );

    fs::write(
        dest_path,
        format!(
            "
            {enum_}
            {impls}
            {well_known}
            ",
        ),
    )
    .unwrap();
}
