// @generated
/// ProofOfPossessionBTC is the proof of possession that a Babylon
/// address and a Bitcoin secp256k1 secret key are held by the same
/// person
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOfPossessionBtc {
    /// btc_sig_type indicates the type of btc_sig in the pop
    #[prost(enumeration = "BtcSigType", tag = "1")]
    pub btc_sig_type: i32,
    /// btc_sig is the signature generated via sign(sk_btc, babylon_staker_address)
    /// the signature follows encoding in either BIP-340 spec or BIP-322 spec
    #[prost(bytes = "vec", tag = "2")]
    pub btc_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ProofOfPossessionBtc {
    const NAME: &'static str = "ProofOfPossessionBTC";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BIP322Sig is a BIP-322 signature together with the address corresponding to
/// the signer
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bip322Sig {
    /// address is the signer's address
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// sig is the actual signature in BIP-322 format
    #[prost(bytes = "vec", tag = "2")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Bip322Sig {
    const NAME: &'static str = "BIP322Sig";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCSigType indicates the type of btc_sig in a pop
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BtcSigType {
    /// BIP340 means the btc_sig will follow the BIP-340 encoding
    Bip340 = 0,
    /// BIP322 means the btc_sig will follow the BIP-322 encoding
    Bip322 = 1,
    /// ECDSA means the btc_sig will follow the ECDSA encoding
    /// ref: <https://github.com/okx/js-wallet-sdk/blob/a57c2acbe6ce917c0aa4e951d96c4e562ad58444/packages/coin-bitcoin/src/BtcWallet.ts#L331>
    Ecdsa = 2,
}
impl BtcSigType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BtcSigType::Bip340 => "BIP340",
            BtcSigType::Bip322 => "BIP322",
            BtcSigType::Ecdsa => "ECDSA",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BIP340" => Some(Self::Bip340),
            "BIP322" => Some(Self::Bip322),
            "ECDSA" => Some(Self::Ecdsa),
            _ => None,
        }
    }
}
/// FinalityProvider defines a finality provider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProvider {
    /// addr is the bech32 address identifier of the finality provider.
    #[prost(string, tag = "1")]
    pub addr: ::prost::alloc::string::String,
    /// description defines the description terms for the finality provider.
    #[prost(message, optional, tag = "2")]
    pub description:
        ::core::option::Option<super::super::super::cosmos::staking::v1beta1::Description>,
    /// commission defines the commission rate of the finality provider.
    #[prost(string, tag = "3")]
    pub commission: ::prost::alloc::string::String,
    /// btc_pk is the Bitcoin secp256k1 PK of this finality provider
    /// the PK follows encoding in BIP-340 spec
    #[prost(bytes = "vec", tag = "4")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// pop is the proof of possession of the btc_pk, where the BTC
    /// private key signs the bech32 bbn addr of the finality provider.
    #[prost(message, optional, tag = "5")]
    pub pop: ::core::option::Option<ProofOfPossessionBtc>,
    /// slashed_babylon_height indicates the Babylon height when
    /// the finality provider is slashed.
    /// if it's 0 then the finality provider is not slashed
    #[prost(uint64, tag = "6")]
    pub slashed_babylon_height: u64,
    /// slashed_btc_height indicates the BTC height when
    /// the finality provider is slashed.
    /// if it's 0 then the finality provider is not slashed
    #[prost(uint32, tag = "7")]
    pub slashed_btc_height: u32,
    /// jailed defines whether the finality provider is jailed
    #[prost(bool, tag = "8")]
    pub jailed: bool,
    /// highest_voted_height is the highest height for which the
    /// finality provider has voted
    ///
    /// NOTE: consumer_id field is not yet backported to the release branch.
    /// To keep it consistent with the code on main branch, consumer_info has field number 11 instead of 10.
    #[prost(uint32, tag = "9")]
    pub highest_voted_height: u32,
    /// commission_info contains information details of the finality provider commission.
    #[prost(message, optional, tag = "11")]
    pub commission_info: ::core::option::Option<CommissionInfo>,
}
impl ::prost::Name for FinalityProvider {
    const NAME: &'static str = "FinalityProvider";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// CommissionInfo defines the information related to the commission of
/// a finality provider.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommissionInfo {
    /// max_rate defines the maximum commission rate which validator can ever charge, as a fraction.
    #[prost(string, tag = "1")]
    pub max_rate: ::prost::alloc::string::String,
    /// max_change_rate defines the maximum daily increase of the validator commission, as a fraction.
    #[prost(string, tag = "2")]
    pub max_change_rate: ::prost::alloc::string::String,
    /// update_time is the last time the commission rate was changed.
    #[prost(message, optional, tag = "3")]
    pub update_time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
}
impl ::prost::Name for CommissionInfo {
    const NAME: &'static str = "CommissionInfo";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// FinalityProviderWithMeta wraps the FinalityProvider with metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderWithMeta {
    /// btc_pk is the Bitcoin secp256k1 PK of thisfinality provider
    /// the PK follows encoding in BIP-340 spec
    #[prost(bytes = "vec", tag = "1")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// height is the queried Babylon height
    #[prost(uint64, tag = "2")]
    pub height: u64,
    /// voting_power is the voting power of this finality provider at the given
    /// height
    #[prost(uint64, tag = "3")]
    pub voting_power: u64,
    /// slashed_babylon_height indicates the Babylon height when
    /// the finality provider is slashed.
    /// if it's 0 then the finality provider is not slashed
    #[prost(uint64, tag = "4")]
    pub slashed_babylon_height: u64,
    /// slashed_btc_height indicates the BTC height when
    /// the finality provider is slashed.
    /// if it's 0 then the finality provider is not slashed
    #[prost(uint32, tag = "5")]
    pub slashed_btc_height: u32,
    /// jailed defines whether the finality provider is detected jailed
    #[prost(bool, tag = "6")]
    pub jailed: bool,
    /// highest_voted_height is the highest height for which the
    /// finality provider has voted
    #[prost(uint32, tag = "7")]
    pub highest_voted_height: u32,
}
impl ::prost::Name for FinalityProviderWithMeta {
    const NAME: &'static str = "FinalityProviderWithMeta";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCDelegation defines a BTC delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegation {
    /// staker_addr is the address to receive rewards from BTC delegation.
    #[prost(string, tag = "1")]
    pub staker_addr: ::prost::alloc::string::String,
    /// btc_pk is the Bitcoin secp256k1 PK of this BTC delegation
    /// the PK follows encoding in BIP-340 spec
    #[prost(bytes = "vec", tag = "2")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// pop is the proof of possession of babylon_pk and btc_pk
    #[prost(message, optional, tag = "3")]
    pub pop: ::core::option::Option<ProofOfPossessionBtc>,
    /// fp_btc_pk_list is the list of BIP-340 PKs of the finality providers that
    /// this BTC delegation delegates to
    /// If there is more than 1 PKs, then this means the delegation is restaked
    /// to multiple finality providers
    #[prost(bytes = "vec", repeated, tag = "4")]
    pub fp_btc_pk_list: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// staking_time is the number of blocks for which the delegation is locked on
    /// BTC chain
    #[prost(uint32, tag = "5")]
    pub staking_time: u32,
    /// start_height is the start BTC height of the BTC delegation
    /// it is the start BTC height of the timelock
    #[prost(uint32, tag = "6")]
    pub start_height: u32,
    /// end_height is the end height of the BTC delegation
    /// it is calculated by end_height = start_height + staking_time
    #[prost(uint32, tag = "7")]
    pub end_height: u32,
    /// total_sat is the total amount of BTC stakes in this delegation
    /// quantified in satoshi
    #[prost(uint64, tag = "8")]
    pub total_sat: u64,
    /// staking_tx is the staking tx
    #[prost(bytes = "vec", tag = "9")]
    pub staking_tx: ::prost::alloc::vec::Vec<u8>,
    /// staking_output_idx is the index of the staking output in the staking tx
    #[prost(uint32, tag = "10")]
    pub staking_output_idx: u32,
    /// slashing_tx is the slashing tx
    /// It is partially signed by SK corresponding to btc_pk, but not signed by
    /// finality provider or covenant yet.
    #[prost(bytes = "vec", tag = "11")]
    pub slashing_tx: ::prost::alloc::vec::Vec<u8>,
    /// delegator_sig is the signature on the slashing tx
    /// by the delegator (i.e., SK corresponding to btc_pk).
    /// It will be a part of the witness for the staking tx output.
    #[prost(bytes = "vec", tag = "12")]
    pub delegator_sig: ::prost::alloc::vec::Vec<u8>,
    /// covenant_sigs is a list of adaptor signatures on the slashing tx
    /// by each covenant member
    /// It will be a part of the witness for the staking tx output.
    #[prost(message, repeated, tag = "13")]
    pub covenant_sigs: ::prost::alloc::vec::Vec<CovenantAdaptorSignatures>,
    /// unbonding_time describes how long the funds will be locked either in
    /// unbonding output or slashing change output
    #[prost(uint32, tag = "14")]
    pub unbonding_time: u32,
    /// btc_undelegation is the information about the early unbonding path of the
    /// BTC delegation
    #[prost(message, optional, tag = "15")]
    pub btc_undelegation: ::core::option::Option<BtcUndelegation>,
    /// version of the params used to validate the delegation
    #[prost(uint32, tag = "16")]
    pub params_version: u32,
    /// btc_tip_height is the height of the BTC light client tip at the time of
    /// the delegation creation
    #[prost(uint32, tag = "17")]
    pub btc_tip_height: u32,
}
impl ::prost::Name for BtcDelegation {
    const NAME: &'static str = "BTCDelegation";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// DelegatorUnbondingInfo contains the information about transaction which spent
/// the staking output. It contains:
/// - spend_stake_tx: the transaction which spent the staking output
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorUnbondingInfo {
    /// spend_stake_tx is the transaction which spent the staking output. It is
    /// filled only if spend_stake_tx is different than unbonding_tx registered
    /// on the Babylon chain.
    #[prost(bytes = "vec", tag = "1")]
    pub spend_stake_tx: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for DelegatorUnbondingInfo {
    const NAME: &'static str = "DelegatorUnbondingInfo";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCUndelegation contains the information about the early unbonding path of
/// the BTC delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcUndelegation {
    /// unbonding_tx is the transaction which will transfer the funds from staking
    /// output to unbonding output. Unbonding output will usually have lower
    /// timelock than staking output.
    #[prost(bytes = "vec", tag = "1")]
    pub unbonding_tx: ::prost::alloc::vec::Vec<u8>,
    /// slashing_tx is the slashing tx for unbonding transactions
    /// It is partially signed by SK corresponding to btc_pk, but not signed by
    /// finality provider or covenant yet.
    #[prost(bytes = "vec", tag = "2")]
    pub slashing_tx: ::prost::alloc::vec::Vec<u8>,
    /// delegator_slashing_sig is the signature on the slashing tx
    /// by the delegator (i.e., SK corresponding to btc_pk).
    /// It will be a part of the witness for the unbonding tx output.
    #[prost(bytes = "vec", tag = "3")]
    pub delegator_slashing_sig: ::prost::alloc::vec::Vec<u8>,
    /// covenant_slashing_sigs is a list of adaptor signatures on the slashing tx
    /// by each covenant member
    /// It will be a part of the witness for the staking tx output.
    #[prost(message, repeated, tag = "4")]
    pub covenant_slashing_sigs: ::prost::alloc::vec::Vec<CovenantAdaptorSignatures>,
    /// covenant_unbonding_sig_list is the list of signatures on the unbonding tx
    /// by covenant members
    /// It must be provided after processing undelegate message by Babylon
    #[prost(message, repeated, tag = "5")]
    pub covenant_unbonding_sig_list: ::prost::alloc::vec::Vec<SignatureInfo>,
    /// delegator_unbonding_info is the information about transaction which spent
    /// the staking output
    #[prost(message, optional, tag = "6")]
    pub delegator_unbonding_info: ::core::option::Option<DelegatorUnbondingInfo>,
}
impl ::prost::Name for BtcUndelegation {
    const NAME: &'static str = "BTCUndelegation";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCDelegatorDelegations is a collection of BTC delegations from the same
/// delegator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegatorDelegations {
    #[prost(message, repeated, tag = "1")]
    pub dels: ::prost::alloc::vec::Vec<BtcDelegation>,
}
impl ::prost::Name for BtcDelegatorDelegations {
    const NAME: &'static str = "BTCDelegatorDelegations";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCDelegatorDelegationIndex is a list of staking tx hashes of BTC delegations
/// from the same delegator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegatorDelegationIndex {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub staking_tx_hash_list: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for BtcDelegatorDelegationIndex {
    const NAME: &'static str = "BTCDelegatorDelegationIndex";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// SignatureInfo is a BIP-340 signature together with its signer's BIP-340 PK
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignatureInfo {
    #[prost(bytes = "vec", tag = "1")]
    pub pk: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SignatureInfo {
    const NAME: &'static str = "SignatureInfo";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// CovenantAdaptorSignatures is a list adaptor signatures signed by the
/// covenant with different finality provider's public keys as encryption keys
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CovenantAdaptorSignatures {
    /// cov_pk is the public key of the covenant emulator, used as the public key
    /// of the adaptor signature
    #[prost(bytes = "vec", tag = "1")]
    pub cov_pk: ::prost::alloc::vec::Vec<u8>,
    /// adaptor_sigs is a list of adaptor signatures, each encrypted by a restaked
    /// BTC finality provider's public key
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub adaptor_sigs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for CovenantAdaptorSignatures {
    const NAME: &'static str = "CovenantAdaptorSignatures";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// SelectiveSlashingEvidence is the evidence that the finality provider
/// selectively slashed a BTC delegation
/// NOTE: it's possible that a slashed finality provider exploits the
/// SelectiveSlashingEvidence endpoint while it is actually slashed due to
/// equivocation. But such behaviour does not affect the system's security
/// or gives any benefit for the adversary
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SelectiveSlashingEvidence {
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "1")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// fp_btc_pk is the BTC PK of the finality provider who
    /// launches the selective slashing offence
    #[prost(bytes = "vec", tag = "2")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// recovered_fp_btc_sk is the finality provider's BTC SK recovered from
    /// the covenant adaptor/Schnorr signature pair. It is the consequence
    /// of selective slashing.
    #[prost(bytes = "vec", tag = "3")]
    pub recovered_fp_btc_sk: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SelectiveSlashingEvidence {
    const NAME: &'static str = "SelectiveSlashingEvidence";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// InclusionProof proves the existence of tx on BTC blockchain
/// including
/// - the position of the tx on BTC blockchain
/// - the Merkle proof that this tx is on the above position
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InclusionProof {
    /// key is the position (txIdx, blockHash) of this tx on BTC blockchain
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<super::super::btccheckpoint::v1::TransactionKey>,
    /// proof is the Merkle proof that this tx is included in the position in `key`
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for InclusionProof {
    const NAME: &'static str = "InclusionProof";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// LargestBtcReOrg stores the largest BTC reorg recorded
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LargestBtcReOrg {
    /// BlockDiff is the difference of the block height of the BTC header Tip - the btc height
    /// which it was rolled back
    #[prost(uint32, tag = "1")]
    pub block_diff: u32,
    /// RollbackFrom is the latest BTC block header prior to rollback
    #[prost(message, optional, tag = "2")]
    pub rollback_from: ::core::option::Option<super::super::btclightclient::v1::BtcHeaderInfo>,
    /// RollbackTo is the BTC block header which we rollback to
    #[prost(message, optional, tag = "3")]
    pub rollback_to: ::core::option::Option<super::super::btclightclient::v1::BtcHeaderInfo>,
}
impl ::prost::Name for LargestBtcReOrg {
    const NAME: &'static str = "LargestBtcReOrg";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCDelegationStatus is the status of a delegation.
/// There are two possible valid state transition paths for a BTC delegation:
/// - PENDING -> VERIFIED -> ACTIVE -> UNBONDED -> EXPIRED
/// - PENDING -> VERIFIED -> ACTIVE -> UNBONDED/EXPIRED
/// and one invalid state transition path:
/// - PENDING -> VERIFIED -> UNBONDED i.e the staker unbonded before
/// activating delegation on Babylon chain.
/// In valid transition paths, the delegation becomes UNBONDED when:
/// - either the staking transaction timelock expires
/// - or the staker requests early undelegation through MsgBTCUndelegate message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BtcDelegationStatus {
    /// PENDING defines a delegation that is waiting for covenant signatures.
    Pending = 0,
    /// VERIFIED defines a delegation that has covenant signatures but is not yet
    /// included in the BTC chain.
    Verified = 1,
    /// ACTIVE defines a delegation that has voting power
    Active = 2,
    /// UNBONDED defines a delegation no longer has voting power
    /// by receiving unbonding tx with signatures from staker and covenant
    /// committee
    Unbonded = 3,
    /// EXPIRED defines a delegation no longer has voting power
    /// for reaching the end of staking transaction timelock
    Expired = 4,
    /// ANY is any of the above status
    Any = 5,
}
impl BtcDelegationStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BtcDelegationStatus::Pending => "PENDING",
            BtcDelegationStatus::Verified => "VERIFIED",
            BtcDelegationStatus::Active => "ACTIVE",
            BtcDelegationStatus::Unbonded => "UNBONDED",
            BtcDelegationStatus::Expired => "EXPIRED",
            BtcDelegationStatus::Any => "ANY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PENDING" => Some(Self::Pending),
            "VERIFIED" => Some(Self::Verified),
            "ACTIVE" => Some(Self::Active),
            "UNBONDED" => Some(Self::Unbonded),
            "EXPIRED" => Some(Self::Expired),
            "ANY" => Some(Self::Any),
            _ => None,
        }
    }
}
/// EventFinalityProviderCreated is the event emitted when a finality provider is created
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventFinalityProviderCreated {
    /// btc_pk_hex is the hex string of Bitcoin secp256k1 PK of this finality provider
    #[prost(string, tag = "1")]
    pub btc_pk_hex: ::prost::alloc::string::String,
    /// addr is the babylon address to receive commission from delegations.
    #[prost(string, tag = "2")]
    pub addr: ::prost::alloc::string::String,
    /// commission defines the commission rate of the finality provider in decimals.
    #[prost(string, tag = "3")]
    pub commission: ::prost::alloc::string::String,
    /// moniker defines a human-readable name for the finality provider.
    #[prost(string, tag = "4")]
    pub moniker: ::prost::alloc::string::String,
    /// identity defines an optional identity signature (ex. UPort or Keybase).
    #[prost(string, tag = "5")]
    pub identity: ::prost::alloc::string::String,
    /// website defines an optional website link.
    #[prost(string, tag = "6")]
    pub website: ::prost::alloc::string::String,
    /// security_contact defines an optional email for security contact.
    #[prost(string, tag = "7")]
    pub security_contact: ::prost::alloc::string::String,
    /// details define other optional details.
    #[prost(string, tag = "8")]
    pub details: ::prost::alloc::string::String,
}
impl ::prost::Name for EventFinalityProviderCreated {
    const NAME: &'static str = "EventFinalityProviderCreated";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventFinalityProviderEdited is the event emitted when a finality provider is edited
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventFinalityProviderEdited {
    /// btc_pk_hex is the hex string of Bitcoin secp256k1 PK of this finality provider
    #[prost(string, tag = "1")]
    pub btc_pk_hex: ::prost::alloc::string::String,
    /// commission defines the commission rate of the finality provider in decimals.
    #[prost(string, tag = "2")]
    pub commission: ::prost::alloc::string::String,
    /// moniker defines a human-readable name for the finality provider.
    #[prost(string, tag = "3")]
    pub moniker: ::prost::alloc::string::String,
    /// identity defines an optional identity signature (ex. UPort or Keybase).
    #[prost(string, tag = "4")]
    pub identity: ::prost::alloc::string::String,
    /// website defines an optional website link.
    #[prost(string, tag = "5")]
    pub website: ::prost::alloc::string::String,
    /// security_contact defines an optional email for security contact.
    #[prost(string, tag = "6")]
    pub security_contact: ::prost::alloc::string::String,
    /// details define other optional details.
    #[prost(string, tag = "7")]
    pub details: ::prost::alloc::string::String,
}
impl ::prost::Name for EventFinalityProviderEdited {
    const NAME: &'static str = "EventFinalityProviderEdited";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventBTCDelegationStateUpdate is the event emitted when a BTC delegation's state is
/// updated. There are the following possible state transitions:
/// - non-existing -> pending, which happens upon `MsgCreateBTCDelegation`
/// - pending -> active, which happens upon `MsgAddCovenantSigs`
/// - active -> unbonded, which happens upon `MsgBTCUndelegate` or upon staking tx timelock expires
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBtcDelegationStateUpdate {
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "1")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// new_state is the new state of this BTC delegation
    #[prost(enumeration = "BtcDelegationStatus", tag = "2")]
    pub new_state: i32,
}
impl ::prost::Name for EventBtcDelegationStateUpdate {
    const NAME: &'static str = "EventBTCDelegationStateUpdate";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventSelectiveSlashing is the event emitted when an adversarial
/// finality provider selectively slashes a BTC delegation. This will
/// result in slashing of all BTC delegations under this finality provider.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventSelectiveSlashing {
    /// evidence is the evidence of selective slashing
    #[prost(message, optional, tag = "1")]
    pub evidence: ::core::option::Option<SelectiveSlashingEvidence>,
}
impl ::prost::Name for EventSelectiveSlashing {
    const NAME: &'static str = "EventSelectiveSlashing";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventPowerDistUpdate is an event that affects voting power distribution
/// of BTC staking protocol
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventPowerDistUpdate {
    /// ev is the event that affects voting power distribution
    #[prost(oneof = "event_power_dist_update::Ev", tags = "1, 2, 3, 4")]
    pub ev: ::core::option::Option<event_power_dist_update::Ev>,
}
/// Nested message and enum types in `EventPowerDistUpdate`.
pub mod event_power_dist_update {
    /// EventSlashedFinalityProvider defines an event that a finality provider
    /// is slashed
    /// TODO: unify with existing slashing events
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct EventSlashedFinalityProvider {
        #[prost(bytes = "vec", tag = "1")]
        pub pk: ::prost::alloc::vec::Vec<u8>,
    }
    impl ::prost::Name for EventSlashedFinalityProvider {
        const NAME: &'static str = "EventSlashedFinalityProvider";
        const PACKAGE: &'static str = "babylon.btcstaking.v1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("babylon.btcstaking.v1.EventPowerDistUpdate.{}", Self::NAME)
        }
    }
    /// EventJailedFinalityProvider defines an event that a finality provider
    /// is jailed after being detected sluggish
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct EventJailedFinalityProvider {
        #[prost(bytes = "vec", tag = "1")]
        pub pk: ::prost::alloc::vec::Vec<u8>,
    }
    impl ::prost::Name for EventJailedFinalityProvider {
        const NAME: &'static str = "EventJailedFinalityProvider";
        const PACKAGE: &'static str = "babylon.btcstaking.v1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("babylon.btcstaking.v1.EventPowerDistUpdate.{}", Self::NAME)
        }
    }
    /// EventUnjailedFinalityProvider defines an event that a jailed finality provider
    /// is unjailed after the jailing period is passed
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct EventUnjailedFinalityProvider {
        #[prost(bytes = "vec", tag = "1")]
        pub pk: ::prost::alloc::vec::Vec<u8>,
    }
    impl ::prost::Name for EventUnjailedFinalityProvider {
        const NAME: &'static str = "EventUnjailedFinalityProvider";
        const PACKAGE: &'static str = "babylon.btcstaking.v1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("babylon.btcstaking.v1.EventPowerDistUpdate.{}", Self::NAME)
        }
    }
    /// ev is the event that affects voting power distribution
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Ev {
        /// slashed_fp means a finality provider is slashed
        #[prost(message, tag = "1")]
        SlashedFp(EventSlashedFinalityProvider),
        /// jailed_fp means a finality provider is jailed
        #[prost(message, tag = "2")]
        JailedFp(EventJailedFinalityProvider),
        /// unjailed_fp means a jailed finality provider is unjailed
        #[prost(message, tag = "3")]
        UnjailedFp(EventUnjailedFinalityProvider),
        /// btc_del_state_update means a BTC delegation's state is updated
        #[prost(message, tag = "4")]
        BtcDelStateUpdate(super::EventBtcDelegationStateUpdate),
    }
}
impl ::prost::Name for EventPowerDistUpdate {
    const NAME: &'static str = "EventPowerDistUpdate";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// A finality provider starts with status INACTIVE once registered.
/// Possible status transitions are when:
/// 1. it has accumulated sufficient delegations and has
/// timestamped public randomness:
/// INACTIVE -> ACTIVE
/// 2. it is jailed due to downtime:
/// ACTIVE -> JAILED
/// 3. it is slashed due to double-sign:
/// ACTIVE -> SLASHED
/// 4. it is unjailed after a jailing period:
/// JAILED -> INACTIVE/ACTIVE (depending on (1))
/// 5. it does not have sufficient delegations or does not
/// have timestamped public randomness:
/// ACTIVE -> INACTIVE.
/// Note that it is impossible for a SLASHED finality provider to
/// transition to other status
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventFinalityProviderStatusChange {
    /// btc_pk is the BTC public key of the finality provider
    #[prost(string, tag = "1")]
    pub btc_pk: ::prost::alloc::string::String,
    /// new_state is the status that the finality provider
    /// is transitioned to, following FinalityProviderStatus
    #[prost(string, tag = "2")]
    pub new_state: ::prost::alloc::string::String,
}
impl ::prost::Name for EventFinalityProviderStatusChange {
    const NAME: &'static str = "EventFinalityProviderStatusChange";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventBTCDelegationCreated is the event emitted when a BTC delegation is created
/// on the Babylon chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBtcDelegationCreated {
    /// staking_tx_hex is the hex encoded staking tx
    #[prost(string, tag = "1")]
    pub staking_tx_hex: ::prost::alloc::string::String,
    /// staking_output_index is the index of the staking output in the staking tx
    #[prost(string, tag = "2")]
    pub staking_output_index: ::prost::alloc::string::String,
    /// version of the params used to validate the delegation
    #[prost(string, tag = "3")]
    pub params_version: ::prost::alloc::string::String,
    /// finality_provider_btc_pks_hex is the list of hex str of Bitcoin secp256k1 PK of
    /// the finality providers that this BTC delegation delegates to
    /// the PK follows encoding in BIP-340 spec
    #[prost(string, repeated, tag = "4")]
    pub finality_provider_btc_pks_hex: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// staker_btc_pk_hex is the hex str of Bitcoin secp256k1 PK of the staker that
    /// creates this BTC delegation the PK follows encoding in BIP-340 spec
    #[prost(string, tag = "5")]
    pub staker_btc_pk_hex: ::prost::alloc::string::String,
    /// staking_time is the timelock of the staking tx specified in the BTC script
    #[prost(string, tag = "6")]
    pub staking_time: ::prost::alloc::string::String,
    /// unbonding_time is the time is timelock on unbonding tx chosen by the staker
    #[prost(string, tag = "7")]
    pub unbonding_time: ::prost::alloc::string::String,
    /// unbonding_tx is hex encoded bytes of the unsigned unbonding tx
    #[prost(string, tag = "8")]
    pub unbonding_tx: ::prost::alloc::string::String,
    /// new_state of the BTC delegation
    #[prost(string, tag = "9")]
    pub new_state: ::prost::alloc::string::String,
    /// staker Babylon address
    #[prost(string, tag = "10")]
    pub staker_addr: ::prost::alloc::string::String,
}
impl ::prost::Name for EventBtcDelegationCreated {
    const NAME: &'static str = "EventBTCDelegationCreated";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventCovenantSignatureReceived is the event emitted when a covenant committee
/// sends valid covenant signatures for a BTC delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCovenantSignatureReceived {
    /// staking_tx_hash is the hash of the staking identifing the BTC delegation
    /// that this covenant signature is for
    #[prost(string, tag = "1")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// covenant_btc_pk_hex is the hex str of Bitcoin secp256k1 PK of the
    /// covnenat committee that send the signature
    #[prost(string, tag = "2")]
    pub covenant_btc_pk_hex: ::prost::alloc::string::String,
    /// covenant_unbonding_signature_hex is the hex str of the BIP340 Schnorr
    /// signature of the covenant committee on the unbonding tx
    #[prost(string, tag = "3")]
    pub covenant_unbonding_signature_hex: ::prost::alloc::string::String,
}
impl ::prost::Name for EventCovenantSignatureReceived {
    const NAME: &'static str = "EventCovenantSignatureReceived";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventCovenantQuorumReached is the event emitted quorum of covenant committee
/// is reached for a BTC delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCovenantQuorumReached {
    /// staking_tx_hash is the hash of the staking identifing the BTC delegation
    /// that this covenant signature is for
    #[prost(string, tag = "1")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// new_state of the BTC delegation
    #[prost(string, tag = "2")]
    pub new_state: ::prost::alloc::string::String,
}
impl ::prost::Name for EventCovenantQuorumReached {
    const NAME: &'static str = "EventCovenantQuorumReached";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventBTCDelegationInclusionProofReceived is the event emitted when a BTC delegation
/// inclusion proof is received
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBtcDelegationInclusionProofReceived {
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "1")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// start_height is the start BTC height of the BTC delegation
    /// it is the start BTC height of the timelock
    #[prost(string, tag = "2")]
    pub start_height: ::prost::alloc::string::String,
    /// end_height is the end height of the BTC delegation
    /// it is calculated by end_height = start_height + staking_time
    #[prost(string, tag = "3")]
    pub end_height: ::prost::alloc::string::String,
    /// new_state of the BTC delegation
    #[prost(string, tag = "4")]
    pub new_state: ::prost::alloc::string::String,
}
impl ::prost::Name for EventBtcDelegationInclusionProofReceived {
    const NAME: &'static str = "EventBTCDelegationInclusionProofReceived";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventBTCDelgationUnbondedEarly is the event emitted when a BTC delegation
/// is unbonded by staker sending unbonding tx to BTC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBtcDelgationUnbondedEarly {
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "1")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// start_height is the start BTC height of the early unbonding
    #[prost(string, tag = "2")]
    pub start_height: ::prost::alloc::string::String,
    /// new_state of the BTC delegation
    #[prost(string, tag = "3")]
    pub new_state: ::prost::alloc::string::String,
}
impl ::prost::Name for EventBtcDelgationUnbondedEarly {
    const NAME: &'static str = "EventBTCDelgationUnbondedEarly";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventBTCDelegationExpired is the event emitted when a BTC delegation
/// is unbonded by expiration of the staking tx timelock
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBtcDelegationExpired {
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "1")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// new_state of the BTC delegation
    #[prost(string, tag = "2")]
    pub new_state: ::prost::alloc::string::String,
}
impl ::prost::Name for EventBtcDelegationExpired {
    const NAME: &'static str = "EventBTCDelegationExpired";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventUnexpectedUnbondingTx is the event emitted when an unbonding tx is
/// is different that the one registered in the BTC delegation.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventUnexpectedUnbondingTx {
    /// staking_tx_hash uniquely identifies a BTC delegation being unbonded
    #[prost(string, tag = "1")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// spend_stake_tx_hash has of the transactin spending staking output
    #[prost(string, tag = "2")]
    pub spend_stake_tx_hash: ::prost::alloc::string::String,
    /// spend_stake_tx_header_hash is the hash of the header of the block that
    /// includes the spend_stake_tx
    #[prost(string, tag = "3")]
    pub spend_stake_tx_header_hash: ::prost::alloc::string::String,
    /// spend_stake_tx_block_index is the spend_stake_tx index in the block
    #[prost(uint32, tag = "4")]
    pub spend_stake_tx_block_index: u32,
}
impl ::prost::Name for EventUnexpectedUnbondingTx {
    const NAME: &'static str = "EventUnexpectedUnbondingTx";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// FinalityProviderStatus is the status of a finality provider.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FinalityProviderStatus {
    /// FINALITY_PROVIDER_STATUS_INACTIVE defines a finality provider that does not have sufficient
    /// delegations or does not have timestamped public randomness.
    Inactive = 0,
    /// FINALITY_PROVIDER_STATUS_ACTIVE defines a finality provider that have sufficient delegations
    /// and have timestamped public randomness.
    Active = 1,
    /// FINALITY_PROVIDER_STATUS_JAILED defines a finality provider that is jailed due to downtime
    Jailed = 2,
    /// FINALITY_PROVIDER_STATUS_SLASHED defines a finality provider that is slashed due to double-sign
    Slashed = 3,
}
impl FinalityProviderStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            FinalityProviderStatus::Inactive => "FINALITY_PROVIDER_STATUS_INACTIVE",
            FinalityProviderStatus::Active => "FINALITY_PROVIDER_STATUS_ACTIVE",
            FinalityProviderStatus::Jailed => "FINALITY_PROVIDER_STATUS_JAILED",
            FinalityProviderStatus::Slashed => "FINALITY_PROVIDER_STATUS_SLASHED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FINALITY_PROVIDER_STATUS_INACTIVE" => Some(Self::Inactive),
            "FINALITY_PROVIDER_STATUS_ACTIVE" => Some(Self::Active),
            "FINALITY_PROVIDER_STATUS_JAILED" => Some(Self::Jailed),
            "FINALITY_PROVIDER_STATUS_SLASHED" => Some(Self::Slashed),
            _ => None,
        }
    }
}
/// Params defines the parameters for the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// PARAMETERS COVERING STAKING
    /// covenant_pks is the list of public keys held by the covenant committee
    /// each PK follows encoding in BIP-340 spec on Bitcoin
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub covenant_pks: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// covenant_quorum is the minimum number of signatures needed for the covenant
    /// multisignature
    #[prost(uint32, tag = "2")]
    pub covenant_quorum: u32,
    /// min_staking_value_sat is the minimum of satoshis locked in staking output
    #[prost(int64, tag = "3")]
    pub min_staking_value_sat: i64,
    /// max_staking_value_sat is the maximum of satoshis locked in staking output
    #[prost(int64, tag = "4")]
    pub max_staking_value_sat: i64,
    /// min_staking_time is the minimum lock time specified in staking output
    /// script
    #[prost(uint32, tag = "5")]
    pub min_staking_time_blocks: u32,
    /// max_staking_time_blocks is the maximum lock time time specified in staking
    /// output script
    #[prost(uint32, tag = "6")]
    pub max_staking_time_blocks: u32,
    /// PARAMETERS COVERING SLASHING
    /// slashing_pk_script is the pk_script expected in slashing output ie. the
    /// first output of slashing transaction
    #[prost(bytes = "vec", tag = "7")]
    pub slashing_pk_script: ::prost::alloc::vec::Vec<u8>,
    /// min_slashing_tx_fee_sat is the minimum amount of tx fee (quantified
    /// in Satoshi) needed for the pre-signed slashing tx. It covers both:
    /// staking slashing transaction and unbonding slashing transaction
    #[prost(int64, tag = "8")]
    pub min_slashing_tx_fee_sat: i64,
    /// slashing_rate determines the portion of the staked amount to be slashed,
    /// expressed as a decimal (e.g., 0.5 for 50%). Maximal precion is 2 decimal
    /// places
    #[prost(string, tag = "9")]
    pub slashing_rate: ::prost::alloc::string::String,
    /// PARAMETERS COVERING UNBONDING
    /// unbonding_time is the exact unbonding time required from unbonding
    /// transaction it must be larger than `checkpoint_finalization_timeout` from
    /// `btccheckpoint` module
    #[prost(uint32, tag = "10")]
    pub unbonding_time_blocks: u32,
    /// unbonding_fee exact fee required for unbonding transaction
    #[prost(int64, tag = "11")]
    pub unbonding_fee_sat: i64,
    /// PARAMETERS COVERING FINALITY PROVIDERS
    /// min_commission_rate is the chain-wide minimum commission rate that a
    /// finality provider can charge their delegators expressed as a decimal (e.g.,
    /// 0.5 for 50%). Maximal precion is 2 decimal places
    #[prost(string, tag = "12")]
    pub min_commission_rate: ::prost::alloc::string::String,
    /// base gas fee for delegation creation
    #[prost(uint64, tag = "13")]
    pub delegation_creation_base_gas_fee: u64,
    /// allow_list_expiration_height is the height at which the allow list expires
    /// i.e all staking transactions are allowed to enter Babylon chain afterwards
    /// setting it to 0 means allow list is disabled
    #[prost(uint64, tag = "14")]
    pub allow_list_expiration_height: u64,
    /// btc_activation_height is the btc height from which parameters are activated
    /// (inclusive)
    #[prost(uint32, tag = "15")]
    pub btc_activation_height: u32,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// HeightVersionPair pairs a btc height with a version of the parameters
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeightVersionPair {
    /// start_height is the height from which the parameters are activated
    /// (inclusive)
    #[prost(uint64, tag = "1")]
    pub start_height: u64,
    /// version is the version of the parameters
    #[prost(uint32, tag = "2")]
    pub version: u32,
}
impl ::prost::Name for HeightVersionPair {
    const NAME: &'static str = "HeightVersionPair";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// HeightToVersionMap maps a btc height to a version of the parameters
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeightToVersionMap {
    /// Pairs must be sorted by `start_height` in ascending order, without
    /// duplicates
    #[prost(message, repeated, tag = "1")]
    pub pairs: ::prost::alloc::vec::Vec<HeightVersionPair>,
}
impl ::prost::Name for HeightToVersionMap {
    const NAME: &'static str = "HeightToVersionMap";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// StoredParams attach information about the version of stored parameters
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoredParams {
    /// version of the stored parameters. Each parameters update
    /// increments version number by 1
    #[prost(uint32, tag = "1")]
    pub version: u32,
    /// NOTE: Parameters must always be provided
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for StoredParams {
    const NAME: &'static str = "StoredParams";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the btcstaking module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// different versions of params used through the history of the chain
    #[prost(message, repeated, tag = "1")]
    pub params: ::prost::alloc::vec::Vec<Params>,
    /// finality_providers all the finality providers registered.
    #[prost(message, repeated, tag = "2")]
    pub finality_providers: ::prost::alloc::vec::Vec<FinalityProvider>,
    /// btc_delegations all the btc delegations in the state.
    #[prost(message, repeated, tag = "3")]
    pub btc_delegations: ::prost::alloc::vec::Vec<BtcDelegation>,
    /// block_height_chains the block height of babylon and bitcoin.
    #[prost(message, repeated, tag = "5")]
    pub block_height_chains: ::prost::alloc::vec::Vec<BlockHeightBbnToBtc>,
    /// btc_delegators contains all the btc delegators with the associated finality provider.
    #[prost(message, repeated, tag = "6")]
    pub btc_delegators: ::prost::alloc::vec::Vec<BtcDelegator>,
    /// all the events and its indexes.
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<EventIndex>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BlockHeightBbnToBtc stores the btc <-> bbn block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeightBbnToBtc {
    /// block_height_bbn is the height of the block in the babylon chain.
    #[prost(uint64, tag = "1")]
    pub block_height_bbn: u64,
    /// block_height_btc is the height of the block in the BTC.
    #[prost(uint32, tag = "2")]
    pub block_height_btc: u32,
}
impl ::prost::Name for BlockHeightBbnToBtc {
    const NAME: &'static str = "BlockHeightBbnToBtc";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCDelegator BTC delegator information with the associated finality provider.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegator {
    /// idx the btc delegator index.
    #[prost(message, optional, tag = "1")]
    pub idx: ::core::option::Option<BtcDelegatorDelegationIndex>,
    /// fp_btc_pk the finality provider btc public key.
    #[prost(bytes = "vec", tag = "2")]
    pub fp_btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// del_btc_pk the delegator btc public key.
    #[prost(bytes = "vec", tag = "3")]
    pub del_btc_pk: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BtcDelegator {
    const NAME: &'static str = "BTCDelegator";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// EventIndex contains the event and its index.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventIndex {
    /// idx is the index the event was stored.
    #[prost(uint64, tag = "1")]
    pub idx: u64,
    /// block_height_btc is the height of the block in the BTC chain.
    #[prost(uint32, tag = "2")]
    pub block_height_btc: u32,
    /// event the event stored.
    #[prost(message, optional, tag = "3")]
    pub event: ::core::option::Option<EventPowerDistUpdate>,
}
impl ::prost::Name for EventIndex {
    const NAME: &'static str = "EventIndex";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsByVersionRequest {
    #[prost(uint32, tag = "1")]
    pub version: u32,
}
impl ::prost::Name for QueryParamsByVersionRequest {
    const NAME: &'static str = "QueryParamsByVersionRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsByVersionResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsByVersionResponse {
    const NAME: &'static str = "QueryParamsByVersionResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryParamsByBTCHeightRequest is request type for the Query/ParamsByBTCHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsByBtcHeightRequest {
    #[prost(uint32, tag = "1")]
    pub btc_height: u32,
}
impl ::prost::Name for QueryParamsByBtcHeightRequest {
    const NAME: &'static str = "QueryParamsByBTCHeightRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryParamsByBTCHeightResponse is response type for the Query/QueryParamsByBTCHeightResponse RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsByBtcHeightResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// version is the version of the params for the given BTC height
    #[prost(uint32, tag = "2")]
    pub version: u32,
}
impl ::prost::Name for QueryParamsByBtcHeightResponse {
    const NAME: &'static str = "QueryParamsByBTCHeightResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProvidersRequest is the request type for the
/// Query/FinalityProviders RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProvidersRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryFinalityProvidersRequest {
    const NAME: &'static str = "QueryFinalityProvidersRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProvidersResponse is the response type for the
/// Query/FinalityProviders RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProvidersResponse {
    /// finality_providers contains all the finality providers
    #[prost(message, repeated, tag = "1")]
    pub finality_providers: ::prost::alloc::vec::Vec<FinalityProviderResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryFinalityProvidersResponse {
    const NAME: &'static str = "QueryFinalityProvidersResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProviderRequest requests information about a finality provider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProviderRequest {
    /// fp_btc_pk_hex is the hex str of Bitcoin secp256k1 PK of the finality provider
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryFinalityProviderRequest {
    const NAME: &'static str = "QueryFinalityProviderRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProviderResponse contains information about a finality provider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProviderResponse {
    /// finality_provider contains the FinalityProvider
    #[prost(message, optional, tag = "1")]
    pub finality_provider: ::core::option::Option<FinalityProviderResponse>,
}
impl ::prost::Name for QueryFinalityProviderResponse {
    const NAME: &'static str = "QueryFinalityProviderResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryBTCDelegationsRequest is the request type for the
/// Query/BTCDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcDelegationsRequest {
    /// status is the queried status for BTC delegations
    #[prost(enumeration = "BtcDelegationStatus", tag = "1")]
    pub status: i32,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryBtcDelegationsRequest {
    const NAME: &'static str = "QueryBTCDelegationsRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryBTCDelegationsResponse is the response type for the
/// Query/BTCDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcDelegationsResponse {
    /// btc_delegations contains all the queried BTC delegations under the given status
    #[prost(message, repeated, tag = "1")]
    pub btc_delegations: ::prost::alloc::vec::Vec<BtcDelegationResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryBtcDelegationsResponse {
    const NAME: &'static str = "QueryBTCDelegationsResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProviderDelegationsRequest is the request type for the
/// Query/FinalityProviderDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProviderDelegationsRequest {
    /// fp_btc_pk_hex is the hex str of Bitcoin secp256k1 PK of the finality providerthat
    /// this BTC delegation delegates to
    /// the PK follows encoding in BIP-340 spec
    #[prost(string, tag = "1")]
    pub fp_btc_pk_hex: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryFinalityProviderDelegationsRequest {
    const NAME: &'static str = "QueryFinalityProviderDelegationsRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryFinalityProviderDelegationsResponse is the response type for the
/// Query/FinalityProviderDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryFinalityProviderDelegationsResponse {
    /// btc_delegator_delegations contains all the queried BTC delegations.
    #[prost(message, repeated, tag = "1")]
    pub btc_delegator_delegations: ::prost::alloc::vec::Vec<BtcDelegatorDelegationsResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryFinalityProviderDelegationsResponse {
    const NAME: &'static str = "QueryFinalityProviderDelegationsResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryBTCDelegationRequest is the request type to retrieve a BTC delegation by
/// staking tx hash
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcDelegationRequest {
    /// Hash of staking transaction in btc format
    #[prost(string, tag = "1")]
    pub staking_tx_hash_hex: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryBtcDelegationRequest {
    const NAME: &'static str = "QueryBTCDelegationRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryBTCDelegationResponse is response type matching QueryBTCDelegationRequest
/// and containing BTC delegation information
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBtcDelegationResponse {
    /// BTCDelegation represents the client needed information of an BTCDelegation.
    #[prost(message, optional, tag = "1")]
    pub btc_delegation: ::core::option::Option<BtcDelegationResponse>,
}
impl ::prost::Name for QueryBtcDelegationResponse {
    const NAME: &'static str = "QueryBTCDelegationResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCDelegationResponse is the client needed information from a BTCDelegation with the current status based on parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegationResponse {
    /// staker_addr is the address to receive rewards from BTC delegation.
    #[prost(string, tag = "1")]
    pub staker_addr: ::prost::alloc::string::String,
    /// btc_pk is the Bitcoin secp256k1 PK of this BTC delegation
    /// the PK follows encoding in BIP-340 spec
    #[prost(bytes = "vec", tag = "2")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// fp_btc_pk_list is the list of BIP-340 PKs of the finality providers that
    /// this BTC delegation delegates to
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub fp_btc_pk_list: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// staking_time is the number of blocks for which the delegation is locked on BTC chain
    #[prost(uint32, tag = "4")]
    pub staking_time: u32,
    /// start_height is the start BTC height of the BTC delegation
    /// it is the start BTC height of the timelock
    #[prost(uint32, tag = "5")]
    pub start_height: u32,
    /// end_height is the end height of the BTC delegation
    /// it is the end BTC height of the timelock - w
    #[prost(uint32, tag = "6")]
    pub end_height: u32,
    /// total_sat is the total amount of BTC stakes in this delegation
    /// quantified in satoshi
    #[prost(uint64, tag = "7")]
    pub total_sat: u64,
    /// staking_tx_hex is the hex string of staking tx
    #[prost(string, tag = "8")]
    pub staking_tx_hex: ::prost::alloc::string::String,
    /// slashing_tx_hex is the hex string of slashing tx
    #[prost(string, tag = "9")]
    pub slashing_tx_hex: ::prost::alloc::string::String,
    /// delegator_slash_sig_hex is the signature on the slashing tx
    /// by the delegator (i.e., SK corresponding to btc_pk) as string hex.
    /// It will be a part of the witness for the staking tx output.
    #[prost(string, tag = "10")]
    pub delegator_slash_sig_hex: ::prost::alloc::string::String,
    /// covenant_sigs is a list of adaptor signatures on the slashing tx
    /// by each covenant member
    /// It will be a part of the witness for the staking tx output.
    #[prost(message, repeated, tag = "11")]
    pub covenant_sigs: ::prost::alloc::vec::Vec<CovenantAdaptorSignatures>,
    /// staking_output_idx is the index of the staking output in the staking tx
    #[prost(uint32, tag = "12")]
    pub staking_output_idx: u32,
    /// whether this delegation is active
    #[prost(bool, tag = "13")]
    pub active: bool,
    /// descriptive status of current delegation.
    #[prost(string, tag = "14")]
    pub status_desc: ::prost::alloc::string::String,
    /// unbonding_time used in unbonding output timelock path and in slashing transactions
    /// change outputs
    #[prost(uint32, tag = "15")]
    pub unbonding_time: u32,
    /// undelegation_response is the undelegation info of this delegation.
    #[prost(message, optional, tag = "16")]
    pub undelegation_response: ::core::option::Option<BtcUndelegationResponse>,
    /// params version used to validate delegation
    #[prost(uint32, tag = "17")]
    pub params_version: u32,
}
impl ::prost::Name for BtcDelegationResponse {
    const NAME: &'static str = "BTCDelegationResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// DelegatorUnbondingInfoResponse provides all necessary info about transaction
/// which spent the staking output
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorUnbondingInfoResponse {
    /// spend_stake_tx_hex is the transaction which spent the staking output. It is
    /// filled only if the spend_stake_tx_hex is different than the unbonding_tx_hex
    #[prost(string, tag = "1")]
    pub spend_stake_tx_hex: ::prost::alloc::string::String,
}
impl ::prost::Name for DelegatorUnbondingInfoResponse {
    const NAME: &'static str = "DelegatorUnbondingInfoResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCUndelegationResponse provides all necessary info about the undeleagation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcUndelegationResponse {
    /// unbonding_tx is the transaction which will transfer the funds from staking
    /// output to unbonding output. Unbonding output will usually have lower timelock
    /// than staking output. The unbonding tx as string hex.
    #[prost(string, tag = "1")]
    pub unbonding_tx_hex: ::prost::alloc::string::String,
    /// covenant_unbonding_sig_list is the list of signatures on the unbonding tx
    /// by covenant members
    #[prost(message, repeated, tag = "2")]
    pub covenant_unbonding_sig_list: ::prost::alloc::vec::Vec<SignatureInfo>,
    /// slashingTxHex is the hex string of slashing tx
    #[prost(string, tag = "3")]
    pub slashing_tx_hex: ::prost::alloc::string::String,
    /// delegator_slashing_sig is the signature on the slashing tx
    /// by the delegator (i.e., SK corresponding to btc_pk).
    /// It will be a part of the witness for the unbonding tx output.
    /// The delegator slashing sig as string hex.
    #[prost(string, tag = "4")]
    pub delegator_slashing_sig_hex: ::prost::alloc::string::String,
    /// covenant_slashing_sigs is a list of adaptor signatures on the
    /// unbonding slashing tx by each covenant member
    /// It will be a part of the witness for the staking tx output.
    #[prost(message, repeated, tag = "5")]
    pub covenant_slashing_sigs: ::prost::alloc::vec::Vec<CovenantAdaptorSignatures>,
    /// btc_undelegation_info contains all necessary info about the transaction
    /// which spent the staking output
    #[prost(message, optional, tag = "6")]
    pub delegator_unbonding_info_response: ::core::option::Option<DelegatorUnbondingInfoResponse>,
}
impl ::prost::Name for BtcUndelegationResponse {
    const NAME: &'static str = "BTCUndelegationResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// BTCDelegatorDelegationsResponse is a collection of BTC delegations responses from the same delegator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BtcDelegatorDelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub dels: ::prost::alloc::vec::Vec<BtcDelegationResponse>,
}
impl ::prost::Name for BtcDelegatorDelegationsResponse {
    const NAME: &'static str = "BTCDelegatorDelegationsResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// FinalityProviderResponse defines a finality provider with voting power information.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FinalityProviderResponse {
    /// description defines the description terms for the finality provider.
    #[prost(message, optional, tag = "1")]
    pub description:
        ::core::option::Option<super::super::super::cosmos::staking::v1beta1::Description>,
    /// commission defines the commission rate of the finality provider.
    #[prost(string, tag = "2")]
    pub commission: ::prost::alloc::string::String,
    /// addr is the address to receive commission from delegations.
    #[prost(string, tag = "3")]
    pub addr: ::prost::alloc::string::String,
    /// btc_pk is the Bitcoin secp256k1 PK of this finality provider
    /// the PK follows encoding in BIP-340 spec
    #[prost(bytes = "vec", tag = "4")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// pop is the proof of possession of the BTC_PK by the fp addr.
    /// Essentially is the signature where the BTC SK sigs the fp addr.
    #[prost(message, optional, tag = "5")]
    pub pop: ::core::option::Option<ProofOfPossessionBtc>,
    /// slashed_babylon_height indicates the Babylon height when
    /// the finality provider is slashed.
    /// if it's 0 then the finality provider is not slashed
    #[prost(uint64, tag = "6")]
    pub slashed_babylon_height: u64,
    /// slashed_btc_height indicates the BTC height when
    /// the finality provider is slashed.
    /// if it's 0 then the finality provider is not slashed
    #[prost(uint32, tag = "7")]
    pub slashed_btc_height: u32,
    /// height is the queried Babylon height
    #[prost(uint64, tag = "8")]
    pub height: u64,
    /// jailed defines whether the finality provider is jailed
    #[prost(bool, tag = "9")]
    pub jailed: bool,
    /// highest_voted_height is the highest height for which the
    /// finality provider has voted
    #[prost(uint32, tag = "10")]
    pub highest_voted_height: u32,
    /// commission_info contains information details of the finality provider commission.
    #[prost(message, optional, tag = "11")]
    pub commission_info: ::core::option::Option<CommissionInfo>,
}
impl ::prost::Name for FinalityProviderResponse {
    const NAME: &'static str = "FinalityProviderResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryLargestBtcReOrgRequest query request of the largest BTC reorg request
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLargestBtcReOrgRequest {}
impl ::prost::Name for QueryLargestBtcReOrgRequest {
    const NAME: &'static str = "QueryLargestBtcReOrgRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryLargestBtcReOrgResponse stores the largest BTC reorg recorded
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLargestBtcReOrgResponse {
    /// BlockDiff is the difference of the block height of the BTC header Tip - the btc height
    /// which it was rolled back
    #[prost(uint32, tag = "1")]
    pub block_diff: u32,
    /// RollbackFrom is the latest BTC block header prior to rollback
    #[prost(message, optional, tag = "2")]
    pub rollback_from:
        ::core::option::Option<super::super::btclightclient::v1::BtcHeaderInfoResponse>,
    /// RollbackTo is the BTC block header which we rollback to
    #[prost(message, optional, tag = "3")]
    pub rollback_to:
        ::core::option::Option<super::super::btclightclient::v1::BtcHeaderInfoResponse>,
}
impl ::prost::Name for QueryLargestBtcReOrgResponse {
    const NAME: &'static str = "QueryLargestBtcReOrgResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryParamsVersionsRequest is the request type for the
/// Query/ParamsVersions RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsVersionsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryParamsVersionsRequest {
    const NAME: &'static str = "QueryParamsVersionsRequest";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// QueryParamsVersionsResponse stores all the params with versions.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsVersionsResponse {
    /// params holds all the params with version from this module.
    #[prost(message, repeated, tag = "1")]
    pub params: ::prost::alloc::vec::Vec<StoredParams>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryParamsVersionsResponse {
    const NAME: &'static str = "QueryParamsVersionsResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgCreateFinalityProvider is the message for creating a finality provider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateFinalityProvider {
    /// addr defines the address of the finality provider that will receive
    /// the commissions to all the delegations.
    #[prost(string, tag = "1")]
    pub addr: ::prost::alloc::string::String,
    /// description defines the description terms for the finality provider
    #[prost(message, optional, tag = "2")]
    pub description:
        ::core::option::Option<super::super::super::cosmos::staking::v1beta1::Description>,
    /// btc_pk is the Bitcoin secp256k1 PK of this finality provider
    /// the PK follows encoding in BIP-340 spec
    #[prost(bytes = "vec", tag = "4")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// pop is the proof of possession of btc_pk over the FP signer address.
    ///
    /// NOTE: consumer_id field is not yet backported to the release branch.
    /// To keep it consistent with the code on main branch, commission has field number 7 instead of 6.
    #[prost(message, optional, tag = "5")]
    pub pop: ::core::option::Option<ProofOfPossessionBtc>,
    /// commission is the finality provider commission information
    #[prost(message, optional, tag = "7")]
    pub commission: ::core::option::Option<CommissionRates>,
}
impl ::prost::Name for MsgCreateFinalityProvider {
    const NAME: &'static str = "MsgCreateFinalityProvider";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgCreateFinalityProviderResponse is the response for
/// MsgCreateFinalityProvider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateFinalityProviderResponse {}
impl ::prost::Name for MsgCreateFinalityProviderResponse {
    const NAME: &'static str = "MsgCreateFinalityProviderResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// CommissionRates defines the initial commission rates to be used for creating
/// a finality provider.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommissionRates {
    /// rate is the commission rate charged to delegators, as a fraction.
    #[prost(string, tag = "1")]
    pub rate: ::prost::alloc::string::String,
    /// max_rate defines the maximum commission rate which finality provider can
    /// ever charge, as a fraction.
    #[prost(string, tag = "2")]
    pub max_rate: ::prost::alloc::string::String,
    /// max_change_rate defines the maximum daily increase of the finality provider
    /// commission, as a fraction.
    #[prost(string, tag = "3")]
    pub max_change_rate: ::prost::alloc::string::String,
}
impl ::prost::Name for CommissionRates {
    const NAME: &'static str = "CommissionRates";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgEditFinalityProvider is the message for editing an existing finality
/// provider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgEditFinalityProvider {
    /// addr the address of the finality provider that whishes to edit his
    /// information.
    #[prost(string, tag = "1")]
    pub addr: ::prost::alloc::string::String,
    /// btc_pk is the Bitcoin secp256k1 PK of the finality provider to be edited
    #[prost(bytes = "vec", tag = "2")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// description defines the updated description terms for the finality provider
    #[prost(message, optional, tag = "3")]
    pub description:
        ::core::option::Option<super::super::super::cosmos::staking::v1beta1::Description>,
    /// commission defines the updated commission rate of the finality provider
    #[prost(string, tag = "4")]
    pub commission: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgEditFinalityProvider {
    const NAME: &'static str = "MsgEditFinalityProvider";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgEditFinalityProviderResponse is the response for MsgEditFinalityProvider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgEditFinalityProviderResponse {}
impl ::prost::Name for MsgEditFinalityProviderResponse {
    const NAME: &'static str = "MsgEditFinalityProviderResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgCreateBTCDelegation is the message for creating a BTC delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateBtcDelegation {
    /// staker_addr is the address to receive rewards from BTC delegation.
    #[prost(string, tag = "1")]
    pub staker_addr: ::prost::alloc::string::String,
    /// pop is the proof of possession of btc_pk by the staker_addr.
    #[prost(message, optional, tag = "2")]
    pub pop: ::core::option::Option<ProofOfPossessionBtc>,
    /// btc_pk is the Bitcoin secp256k1 PK of the BTC delegator
    #[prost(bytes = "vec", tag = "3")]
    pub btc_pk: ::prost::alloc::vec::Vec<u8>,
    /// fp_btc_pk_list is the list of Bitcoin secp256k1 PKs of the finality
    /// providers, if there is more than one finality provider pk it means that
    /// delegation is re-staked
    #[prost(bytes = "vec", repeated, tag = "4")]
    pub fp_btc_pk_list: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// staking_time is the time lock used in staking transaction
    #[prost(uint32, tag = "5")]
    pub staking_time: u32,
    /// staking_value  is the amount of satoshis locked in staking output
    #[prost(int64, tag = "6")]
    pub staking_value: i64,
    /// staking_tx is a bitcoin staking transaction i.e transaction that locks
    /// funds
    #[prost(bytes = "vec", tag = "7")]
    pub staking_tx: ::prost::alloc::vec::Vec<u8>,
    /// staking_tx_inclusion_proof is the inclusion proof of the staking tx in BTC
    /// chain
    #[prost(message, optional, tag = "8")]
    pub staking_tx_inclusion_proof: ::core::option::Option<InclusionProof>,
    /// slashing_tx is the slashing tx
    /// Note that the tx itself does not contain signatures, which are off-chain.
    #[prost(bytes = "vec", tag = "9")]
    pub slashing_tx: ::prost::alloc::vec::Vec<u8>,
    /// delegator_slashing_sig is the signature on the slashing tx by the delegator
    /// (i.e., SK corresponding to btc_pk). It will be a part of the witness for
    /// the staking tx output. The staking tx output further needs signatures from
    /// covenant and finality provider in order to be spendable.
    #[prost(bytes = "vec", tag = "10")]
    pub delegator_slashing_sig: ::prost::alloc::vec::Vec<u8>,
    /// unbonding_time is the time lock used when funds are being unbonded. It is
    /// be used in:
    /// - unbonding transaction, time lock spending path
    /// - staking slashing transaction, change output
    /// - unbonding slashing transaction, change output
    /// It must be smaller than math.MaxUInt16 and larger that
    /// max(MinUnbondingTime, CheckpointFinalizationTimeout)
    #[prost(uint32, tag = "11")]
    pub unbonding_time: u32,
    /// fields related to unbonding transaction
    /// unbonding_tx is a bitcoin unbonding transaction i.e transaction that spends
    /// staking output and sends it to the unbonding output
    #[prost(bytes = "vec", tag = "12")]
    pub unbonding_tx: ::prost::alloc::vec::Vec<u8>,
    /// unbonding_value is amount of satoshis locked in unbonding output.
    /// NOTE: staking_value and unbonding_value could be different because of the
    /// difference between the fee for staking tx and that for unbonding
    #[prost(int64, tag = "13")]
    pub unbonding_value: i64,
    /// unbonding_slashing_tx is the slashing tx which slash unbonding contract
    /// Note that the tx itself does not contain signatures, which are off-chain.
    #[prost(bytes = "vec", tag = "14")]
    pub unbonding_slashing_tx: ::prost::alloc::vec::Vec<u8>,
    /// delegator_unbonding_slashing_sig is the signature on the slashing tx by the
    /// delegator (i.e., SK corresponding to btc_pk).
    #[prost(bytes = "vec", tag = "15")]
    pub delegator_unbonding_slashing_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgCreateBtcDelegation {
    const NAME: &'static str = "MsgCreateBTCDelegation";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgCreateBTCDelegationResponse is the response for MsgCreateBTCDelegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateBtcDelegationResponse {}
impl ::prost::Name for MsgCreateBtcDelegationResponse {
    const NAME: &'static str = "MsgCreateBTCDelegationResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgAddBTCDelegationInclusionProof is the message for adding proof of
/// inclusion of BTC delegation on BTC chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddBtcDelegationInclusionProof {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "2")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// staking_tx_inclusion_proof is the inclusion proof of the staking tx in BTC
    /// chain
    #[prost(message, optional, tag = "3")]
    pub staking_tx_inclusion_proof: ::core::option::Option<InclusionProof>,
}
impl ::prost::Name for MsgAddBtcDelegationInclusionProof {
    const NAME: &'static str = "MsgAddBTCDelegationInclusionProof";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgAddBTCDelegationInclusionProofResponse is the response for
/// MsgAddBTCDelegationInclusionProof
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddBtcDelegationInclusionProofResponse {}
impl ::prost::Name for MsgAddBtcDelegationInclusionProofResponse {
    const NAME: &'static str = "MsgAddBTCDelegationInclusionProofResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgAddCovenantSigs is the message for handling signatures from a covenant
/// member
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddCovenantSigs {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// pk is the BTC public key of the covenant member
    #[prost(bytes = "vec", tag = "2")]
    pub pk: ::prost::alloc::vec::Vec<u8>,
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "3")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// sigs is a list of adaptor signatures of the covenant
    /// the order of sigs should respect the order of finality providers
    /// of the corresponding delegation
    #[prost(bytes = "vec", repeated, tag = "4")]
    pub slashing_tx_sigs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// unbonding_tx_sig is the signature of the covenant on the unbonding tx
    /// submitted to babylon the signature follows encoding in BIP-340 spec
    #[prost(bytes = "vec", tag = "5")]
    pub unbonding_tx_sig: ::prost::alloc::vec::Vec<u8>,
    /// slashing_unbonding_tx_sigs is a list of adaptor signatures of the covenant
    /// on slashing tx corresponding to unbonding tx submitted to babylon
    /// the order of sigs should respect the order of finality providers
    /// of the corresponding delegation
    #[prost(bytes = "vec", repeated, tag = "6")]
    pub slashing_unbonding_tx_sigs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for MsgAddCovenantSigs {
    const NAME: &'static str = "MsgAddCovenantSigs";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgAddCovenantSigsResponse is the response for MsgAddCovenantSigs
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddCovenantSigsResponse {}
impl ::prost::Name for MsgAddCovenantSigsResponse {
    const NAME: &'static str = "MsgAddCovenantSigsResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgBTCUndelegate is the message for handling signature on unbonding tx
/// from its delegator. This signature effectively proves that the delegator
/// wants to unbond this BTC delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBtcUndelegate {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "2")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// stake_spending_tx is a bitcoin transaction that spends the staking
    /// transaction i.e it has staking output as an input
    #[prost(bytes = "vec", tag = "3")]
    pub stake_spending_tx: ::prost::alloc::vec::Vec<u8>,
    /// spend_spending_tx_inclusion_proof is the proof of inclusion of the
    /// stake_spending_tx in the BTC chain
    #[prost(message, optional, tag = "4")]
    pub stake_spending_tx_inclusion_proof: ::core::option::Option<InclusionProof>,
    /// funding_transactions is a list of bitcoin transactions that funds the stake_spending_tx
    /// i.e. they are inputs of the stake_spending_tx
    #[prost(bytes = "vec", repeated, tag = "5")]
    pub funding_transactions: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for MsgBtcUndelegate {
    const NAME: &'static str = "MsgBTCUndelegate";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgBTCUndelegateResponse is the response for MsgBTCUndelegate
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBtcUndelegateResponse {}
impl ::prost::Name for MsgBtcUndelegateResponse {
    const NAME: &'static str = "MsgBTCUndelegateResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgSelectiveSlashingEvidence is the message for handling evidence of
/// selective slashing launched by a finality provider
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSelectiveSlashingEvidence {
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// staking_tx_hash is the hash of the staking tx.
    /// It uniquely identifies a BTC delegation
    #[prost(string, tag = "2")]
    pub staking_tx_hash: ::prost::alloc::string::String,
    /// recovered_fp_btc_sk is the BTC SK of the finality provider who
    /// launches the selective slashing offence. The SK is recovered by
    /// using a covenant adaptor signature and the corresponding Schnorr
    /// signature
    #[prost(bytes = "vec", tag = "3")]
    pub recovered_fp_btc_sk: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MsgSelectiveSlashingEvidence {
    const NAME: &'static str = "MsgSelectiveSlashingEvidence";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgSelectiveSlashingEvidenceResponse is the response for
/// MsgSelectiveSlashingEvidence
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSelectiveSlashingEvidenceResponse {}
impl ::prost::Name for MsgSelectiveSlashingEvidenceResponse {
    const NAME: &'static str = "MsgSelectiveSlashingEvidenceResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams defines a message for updating btcstaking module parameters.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address of the governance account.
    /// just FYI: cosmos.AddressString marks that this field should use type alias
    /// for AddressString instead of string, but the functionality is not yet
    /// implemented in cosmos-proto
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the finality parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse is the response to the MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "babylon.btcstaking.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.btcstaking.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
