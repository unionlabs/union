use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, CustomMsg, CustomQuery, Uint128};

/// Special messages to be supported by any chain that supports token_factory
#[cw_serde]
pub enum TokenFactoryMsg {
    /// CreateDenom creates a new factory denom, of denomination:
    /// factory/{creating contract bech32 address}/{Subdenom}
    /// Subdenom can be of length at most 44 characters, in [0-9a-zA-Z./]
    /// Empty subdenoms are valid.
    /// The (creating contract address, subdenom) pair must be unique.
    /// The created denom's admin is the creating contract address,
    /// but this admin can be changed using the UpdateAdmin binding.
    ///
    /// If you set an initial metadata here, this is equivalent
    /// to calling SetMetadata directly on the returned denom.
    CreateDenom {
        subdenom: String,
        // TODO: upgrade tokenfactory to handle this
        // metadata: Option<Metadata>,
    },
    /// ChangeAdmin changes the admin for a factory denom.
    /// Can only be called by the current contract admin.
    /// If the NewAdminAddress is empty, the denom will have no admin.
    ChangeAdmin {
        denom: String,
        new_admin_address: String,
    },
    /// Contracts can mint native tokens for an existing factory denom
    /// that they are the admin of.
    MintTokens {
        denom: String,
        amount: Uint128,
        mint_to_address: String,
    },
    /// Contracts can burn native tokens for an existing factory denom
    /// that they are the admin of.
    /// Currently, the burn from address must be the admin contract.
    BurnTokens {
        denom: String,
        amount: Uint128,
        burn_from_address: String,
    },
    /// Contracts can set metadata for an existing factory denom that they are
    /// admin of.
    SetDenomMetadata { denom: String, metadata: Metadata },
}

/// This maps to cosmos.bank.v1beta1.Metadata protobuf struct
#[cw_serde]
pub struct Metadata {
    pub description: Option<String>,
    /// denom_units represents the list of DenomUnit's for a given coin
    pub denom_units: Vec<DenomUnit>,
    /// base represents the base denom (should be the DenomUnit with exponent = 0).
    pub base: Option<String>,
    /// display indicates the suggested denom that should be displayed in clients.
    pub display: Option<String>,
    /// name defines the name of the token (eg: Cosmos Atom)
    pub name: Option<String>,
    /// symbol is the token symbol usually shown on exchanges (eg: ATOM). This can
    /// be the same as the display.
    pub symbol: Option<String>,
    /// URI to a document (on or off-chain) that contains additional information. Optional.
    pub uri: Option<String>,
    /// URIHash is a sha256 hash of a document pointed by URI. It's used to verify that the document didn't change. Optional.
    pub uri_hash: Option<String>,
}

/// This maps to cosmos.bank.v1beta1.DenomUnit protobuf struct
#[cw_serde]
pub struct DenomUnit {
    /// denom represents the string name of the given denom unit (e.g uatom).
    pub denom: String,
    /// exponent represents power of 10 exponent that one must
    /// raise the base_denom to in order to equal the given DenomUnit's denom
    /// 1 denom = 1^exponent base_denom
    /// (e.g. with a base_denom of uatom, one can create a DenomUnit of 'atom' with
    /// exponent = 6, thus: 1 atom = 10^6 uatom).
    exponent: u32,
    /// aliases is a list of string aliases for the given denom
    aliases: Vec<String>,
}

/// This maps to tokenfactory.v1beta1.Params protobuf struct
#[cw_serde]
pub struct Params {
    pub denom_creation_fee: Vec<Coin>,
}

impl CustomMsg for TokenFactoryMsg {}

/// This is in the data field in the reply from a TokenFactoryMsg::CreateDenom SubMsg
/// Custom code to parse from protobuf with minimal wasm bytecode bloat
pub struct CreateDenomResponse {
    pub new_token_denom: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum TokenFactoryQuery {
    /// Given a subdenom created by the address `creator_addr` via `TokenFactoryMsg::CreateDenom`,
    /// returns the full denom as used by `BankMsg::Send`.
    /// You may call `FullDenom { creator_addr: env.contract.address, subdenom }` to find the denom issued
    /// by the current contract.
    #[returns(FullDenomResponse)]
    FullDenom {
        creator_addr: String,
        subdenom: String,
    },
    /// Returns the metadata set for this denom, if present. May return None.
    /// This will also return metadata for native tokens created outside
    /// of the token factory (like staking tokens)
    #[returns(MetadataResponse)]
    Metadata { denom: String },
    /// Returns info on admin of the denom, only if created/managed via token factory.
    /// Errors if denom doesn't exist or was created by another module.
    #[returns(AdminResponse)]
    Admin { denom: String },
    /// List all denoms that were created by the given creator.
    /// This does not imply all tokens currently managed by the creator.
    /// (Admin may have changed)
    #[returns(DenomsByCreatorResponse)]
    DenomsByCreator { creator: String },
    /// Returns configuration params for TokenFactory modules
    #[returns(ParamsResponse)]
    Params {},
}

impl CustomQuery for TokenFactoryQuery {}

#[cw_serde]
pub struct FullDenomResponse {
    pub denom: String,
}

#[cw_serde]
pub struct MetadataResponse {
    /// Empty if this was never set for the given denom
    pub metadata: Option<Metadata>,
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: String,
}

#[cw_serde]
pub struct DenomsByCreatorResponse {
    pub denoms: Vec<String>,
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: Params,
}
