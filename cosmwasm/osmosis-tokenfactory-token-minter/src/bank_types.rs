use cosmwasm_schema::cw_serde;
use protos::cosmos::bank::v1beta1 as bank_proto;
use ucs03_zkgm_token_minter_api::Metadata;

/// Normally this type exists under `cosmwasm_std::DenomMetadata`, but the `aliases` field
/// of the `DenomUnit` cannot be parsed because the data is serialized to `Null` in the go side
/// when `aliases` is empty. [The fix](https://github.com/CosmWasm/cosmwasm/pull/2417) is already made
/// but we are going to use these types as a temporary fix until we make sure the chains that we are going
/// to deploy this have the fix already.
#[cw_serde]
pub struct DenomMetadata {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub uri_hash: String,
}

#[cw_serde]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Option<Vec<String>>,
}

#[cw_serde]
pub struct DenomMetadataResponse {
    pub metadata: DenomMetadata,
}

/// Create a metadata for `MsgSetDenomMetadata`.
///
/// # Example
/// Given:
///     - Denom: factory/osmo14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sq2r9g9/3j6g2NP5ZnAhm7qgk2gfM
///     - Metadata:
///         - Name: canimanam
///         - Symbol: CNM
///         - Decimals: 12
/// We get a token:
/// - Base: factory/osmo14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sq2r9g9/3j6g2NP5ZnAhm7qgk2gfM
/// - Name: canimanam
/// - Symbol: CNM
/// - Display: CNM
/// - Denom Units:
///     - 0:
///         - Denom: factory/osmo14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sq2r9g9/3j6g2NP5ZnAhm7qgk2gfM
///         - Exponent: 0
///         - Aliases:
///             - CNM
///     - 1:
///         - Denom: CNM
///         - Expoenent: 12
///         - Aliases: Null
///
/// Note that there are several rules that we had to follow to create a correct token:
/// 1. The first entry of `denom_units` need to have a `denom` such that it's equal to the `base` field.
/// 2. The `denom_units` need to have an entry with `0` exponent.
pub fn new_proto_metadata(denom: String, metadata: Metadata) -> bank_proto::Metadata {
    bank_proto::Metadata {
        description: "".into(),
        denom_units: vec![
            bank_proto::DenomUnit {
                denom: denom.clone(),
                exponent: 0,
                aliases: vec![metadata.symbol.clone()],
            },
            bank_proto::DenomUnit {
                denom: metadata.symbol.clone(),
                exponent: metadata.decimals.into(),
                aliases: vec![],
            },
        ],
        base: denom,
        display: metadata.symbol.clone(),
        name: metadata.name,
        symbol: metadata.symbol,
        uri: "".into(),
        uri_hash: "".into(),
    }
}
