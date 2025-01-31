// @generated
/// WARNING: This message is deprecated in favor of `MsgCreateConsumer`.
/// ConsumerAdditionProposal is a governance proposal on the provider chain to
/// spawn a new consumer chain. If it passes, then all validators on the provider
/// chain are expected to validate the consumer chain at spawn time or get
/// slashed. It is recommended that spawn time occurs after the proposal end
/// time.
/// Use MsgConsumerAddition to submit this proposal type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerAdditionProposal {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the proposed chain-id of the new consumer chain, must be different from all
    /// other consumer chain ids of the executing provider chain.
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    /// the proposed initial height of new consumer chain.
    /// For a completely new chain, this will be {0,1}. However, it may be
    /// different if this is a chain that is converting to a consumer chain.
    #[prost(message, optional, tag = "4")]
    pub initial_height:
        ::core::option::Option<super::super::super::super::ibc::core::client::v1::Height>,
    /// The hash of the consumer chain genesis state without the consumer CCV
    /// module genesis params. It is used for off-chain confirmation of
    /// genesis.json validity by validators and other parties.
    #[prost(bytes = "vec", tag = "5")]
    pub genesis_hash: ::prost::alloc::vec::Vec<u8>,
    /// The hash of the consumer chain binary that should be run by validators on
    /// chain initialization. It is used for off-chain confirmation of binary
    /// validity by validators and other parties.
    #[prost(bytes = "vec", tag = "6")]
    pub binary_hash: ::prost::alloc::vec::Vec<u8>,
    /// spawn time is the time on the provider chain at which the consumer chain
    /// genesis is finalized and all validators will be responsible for starting
    /// their consumer chain validator node.
    #[prost(message, optional, tag = "7")]
    pub spawn_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// Unbonding period for the consumer,
    /// which should be smaller than that of the provider in general.
    #[prost(message, optional, tag = "8")]
    pub unbonding_period: ::core::option::Option<::pbjson_types::Duration>,
    /// Sent CCV related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "9")]
    pub ccv_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// Sent transfer related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "10")]
    pub transfer_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// The fraction of tokens allocated to the consumer redistribution address
    /// during distribution events. The fraction is a string representing a
    /// decimal number. For example "0.75" would represent 75%.
    #[prost(string, tag = "11")]
    pub consumer_redistribution_fraction: ::prost::alloc::string::String,
    /// BlocksPerDistributionTransmission is the number of blocks between
    /// ibc-token-transfers from the consumer chain to the provider chain. On
    /// sending transmission event, `consumer_redistribution_fraction` of the
    /// accumulated tokens are sent to the consumer redistribution address.
    #[prost(int64, tag = "12")]
    pub blocks_per_distribution_transmission: i64,
    /// The number of historical info entries to persist in store.
    /// This param is a part of the cosmos sdk staking module. In the case of
    /// a ccv enabled consumer chain, the ccv module acts as the staking module.
    #[prost(int64, tag = "13")]
    pub historical_entries: i64,
    /// The ID of a token transfer channel used for the Reward Distribution
    /// sub-protocol. If DistributionTransmissionChannel == "", a new transfer
    /// channel is created on top of the same connection as the CCV channel.
    /// Note that transfer_channel_id is the ID of the channel end on the consumer
    /// chain. It is most relevant for chains performing a standalone to consumer
    /// changeover in order to maintain the existing ibc transfer channel
    #[prost(string, tag = "14")]
    pub distribution_transmission_channel: ::prost::alloc::string::String,
    /// Corresponds to the percentage of validators that have to validate the chain under the Top N case.
    /// For example, 53 corresponds to a Top 53% chain, meaning that the top 53% provider validators by voting power
    /// have to validate the proposed consumer chain. top_N can either be 0 or any value in \[50, 100\].
    /// A chain can join with top_N == 0 as an Opt In chain, or with top_N ∈ \[50, 100\] as a Top N chain.
    #[prost(uint32, tag = "15")]
    pub top_n: u32,
    /// Corresponds to the maximum power (percentage-wise) a validator can have on the consumer chain. For instance, if
    /// `validators_power_cap` is set to 32, it means that no validator can have more than 32% of the voting power on the
    /// consumer chain. Note that this might not be feasible. For example, think of a consumer chain with only
    /// 5 validators and with `validators_power_cap` set to 10%. In such a scenario, at least one validator would need
    /// to have more than 20% of the total voting power. Therefore, `validators_power_cap` operates on a best-effort basis.
    #[prost(uint32, tag = "16")]
    pub validators_power_cap: u32,
    /// Corresponds to the maximum number of validators that can validate a consumer chain.
    /// Only applicable to Opt In chains. Setting `validator_set_cap` on a Top N chain is a no-op.
    #[prost(uint32, tag = "17")]
    pub validator_set_cap: u32,
    /// Corresponds to a list of provider consensus addresses of validators that are the ONLY ones that can validate
    /// the consumer chain.
    #[prost(string, repeated, tag = "18")]
    pub allowlist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Corresponds to a list of provider consensus addresses of validators that CANNOT validate the consumer chain.
    #[prost(string, repeated, tag = "19")]
    pub denylist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Corresponds to the minimal amount of (provider chain) stake required to validate on the consumer chain.
    #[prost(uint64, tag = "20")]
    pub min_stake: u64,
    /// Corresponds to whether inactive validators are allowed to validate the consumer chain.
    #[prost(bool, tag = "21")]
    pub allow_inactive_vals: bool,
}
impl ::prost::Name for ConsumerAdditionProposal {
    const NAME: &'static str = "ConsumerAdditionProposal";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// WARNING: This message is deprecated in favor of `MsgRemoveConsumer`.
/// ConsumerRemovalProposal is a governance proposal on the provider chain to
/// remove (and stop) a consumer chain. If it passes, all the consumer chain's
/// state is removed from the provider chain. The outstanding unbonding operation
/// funds are released.
/// Use MsgConsumerRemoval to submit this proposal type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerRemovalProposal {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the chain-id of the consumer chain to be stopped
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    /// the time on the provider chain at which all validators are responsible to
    /// stop their consumer chain validator node
    #[prost(message, optional, tag = "4")]
    pub stop_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for ConsumerRemovalProposal {
    const NAME: &'static str = "ConsumerRemovalProposal";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// WARNING: This message is deprecated in favor of `MsgUpdateConsumer`.
/// ConsumerModificationProposal is a governance proposal on the provider chain to modify parameters of a running
/// consumer chain. If it passes, the consumer chain's state is updated to take into account the newest params.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerModificationProposal {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the chain-id of the consumer chain to be modified
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    /// Corresponds to the percentage of validators that have to validate the chain under the Top N case.
    /// For example, 53 corresponds to a Top 53% chain, meaning that the top 53% provider validators by voting power
    /// have to validate the proposed consumer chain. top_N can either be 0 or any value in \[50, 100\].
    /// A chain can join with top_N == 0 as an Opt In chain, or with top_N ∈ \[50, 100\] as a Top N chain.
    #[prost(uint32, tag = "4")]
    pub top_n: u32,
    /// Corresponds to the maximum power (percentage-wise) a validator can have on the consumer chain. For instance, if
    /// `validators_power_cap` is set to 32, it means that no validator can have more than 32% of the voting power on the
    /// consumer chain. Note that this might not be feasible. For example, think of a consumer chain with only
    /// 5 validators and with `validators_power_cap` set to 10%. In such a scenario, at least one validator would need
    /// to have more than 20% of the total voting power. Therefore, `validators_power_cap` operates on a best-effort basis.
    #[prost(uint32, tag = "5")]
    pub validators_power_cap: u32,
    /// Corresponds to the maximum number of validators that can validate a consumer chain.
    /// Only applicable to Opt In chains. Setting `validator_set_cap` on a Top N chain is a no-op.
    #[prost(uint32, tag = "6")]
    pub validator_set_cap: u32,
    /// Corresponds to a list of provider consensus addresses of validators that are the ONLY ones that can validate
    /// the consumer chain.
    #[prost(string, repeated, tag = "7")]
    pub allowlist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Corresponds to a list of provider consensus addresses of validators that CANNOT validate the consumer chain.
    #[prost(string, repeated, tag = "8")]
    pub denylist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Corresponds to the minimal amount of (provider chain) stake required to validate on the consumer chain.
    #[prost(uint64, tag = "9")]
    pub min_stake: u64,
    /// Corresponds to whether inactive validators are allowed to validate the consumer chain.
    #[prost(bool, tag = "10")]
    pub allow_inactive_vals: bool,
}
impl ::prost::Name for ConsumerModificationProposal {
    const NAME: &'static str = "ConsumerModificationProposal";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// EquivocationProposal is a governance proposal on the provider chain to
/// punish a validator for equivocation on a consumer chain.
///
/// This type is only used internally to the consumer CCV module.
/// WARNING: This message is deprecated now that equivocations can be submitted
/// and verified automatically on the provider. (see SubmitConsumerDoubleVoting in proto/interchain-security/ccv/provider/v1/tx.proto).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EquivocationProposal {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the list of equivocations that will be processed
    #[prost(message, repeated, tag = "3")]
    pub equivocations: ::prost::alloc::vec::Vec<
        super::super::super::super::cosmos::evidence::v1beta1::Equivocation,
    >,
}
impl ::prost::Name for EquivocationProposal {
    const NAME: &'static str = "EquivocationProposal";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ChangeRewardDenomsProposal is a governance proposal on the provider chain to
/// mutate the set of denoms accepted by the provider as rewards.
/// Use MsgChangeRewardDenoms to submit this proposal type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangeRewardDenomsProposal {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the list of consumer reward denoms to add
    #[prost(string, repeated, tag = "3")]
    pub denoms_to_add: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// the list of consumer reward denoms to remove
    #[prost(string, repeated, tag = "4")]
    pub denoms_to_remove: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ChangeRewardDenomsProposal {
    const NAME: &'static str = "ChangeRewardDenomsProposal";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// A persisted queue entry indicating that a slash packet data instance needs to
/// be handled. This type belongs in the "global" queue, to coordinate slash
/// packet handling times between consumers.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GlobalSlashEntry {
    /// Block time that slash packet was received by provider chain.
    /// This field is used for store key iteration ordering.
    #[prost(message, optional, tag = "1")]
    pub recv_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// The consumer that sent a slash packet.
    #[prost(string, tag = "2")]
    pub consumer_chain_id: ::prost::alloc::string::String,
    /// The IBC sequence number of the recv packet.
    /// This field is used in the store key to ensure uniqueness.
    #[prost(uint64, tag = "3")]
    pub ibc_seq_num: u64,
    /// The provider's consensus address of the validator being slashed.
    /// This field is used to obtain validator power in HandleThrottleQueues.
    ///
    /// This field is not used in the store key, but is persisted in value bytes,
    /// see QueueGlobalSlashEntry.
    #[prost(bytes = "vec", tag = "4")]
    pub provider_val_cons_addr: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for GlobalSlashEntry {
    const NAME: &'static str = "GlobalSlashEntry";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// Params defines the parameters for CCV Provider module
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(message, optional, tag = "1")]
    pub template_client: ::core::option::Option<
        super::super::super::super::ibc::lightclients::tendermint::v1::ClientState,
    >,
    /// TrustingPeriodFraction is used to compute the consumer and provider IBC
    /// client's TrustingPeriod from the chain defined UnbondingPeriod
    #[prost(string, tag = "2")]
    pub trusting_period_fraction: ::prost::alloc::string::String,
    /// Sent IBC packets will timeout after this duration
    #[prost(message, optional, tag = "3")]
    pub ccv_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// The period for which the slash meter is replenished
    #[prost(message, optional, tag = "6")]
    pub slash_meter_replenish_period: ::core::option::Option<::pbjson_types::Duration>,
    /// The fraction of total voting power that is replenished to the slash meter
    /// every replenish period. This param also serves as a maximum fraction of
    /// total voting power that the slash meter can hold.
    #[prost(string, tag = "7")]
    pub slash_meter_replenish_fraction: ::prost::alloc::string::String,
    /// The fee required to be paid to add a reward denom
    #[prost(message, optional, tag = "9")]
    pub consumer_reward_denom_registration_fee:
        ::core::option::Option<super::super::super::super::cosmos::base::v1beta1::Coin>,
    /// The number of blocks that comprise an epoch.
    #[prost(int64, tag = "10")]
    pub blocks_per_epoch: i64,
    /// The number of epochs a validator has to validate a consumer chain in order to start receiving rewards from that chain.
    #[prost(int64, tag = "11")]
    pub number_of_epochs_to_start_receiving_rewards: i64,
    /// The maximal number of validators that will be passed
    /// to the consensus engine on the provider.
    #[prost(int64, tag = "12")]
    pub max_provider_consensus_validators: i64,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// SlashAcks contains cons addresses of consumer chain validators
/// successfully slashed on the provider chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SlashAcks {
    #[prost(string, repeated, tag = "1")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for SlashAcks {
    const NAME: &'static str = "SlashAcks";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ConsumerAdditionProposals holds pending governance proposals on the provider
/// chain to spawn a new chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerAdditionProposals {
    /// proposals waiting for spawn_time to pass
    #[prost(message, repeated, tag = "1")]
    pub pending: ::prost::alloc::vec::Vec<ConsumerAdditionProposal>,
}
impl ::prost::Name for ConsumerAdditionProposals {
    const NAME: &'static str = "ConsumerAdditionProposals";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ConsumerRemovalProposals holds pending governance proposals on the provider
/// chain to remove (and stop) a consumer chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerRemovalProposals {
    /// proposals waiting for stop_time to pass
    #[prost(message, repeated, tag = "1")]
    pub pending: ::prost::alloc::vec::Vec<ConsumerRemovalProposal>,
}
impl ::prost::Name for ConsumerRemovalProposals {
    const NAME: &'static str = "ConsumerRemovalProposals";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// AddressList contains a list of consensus addresses
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressList {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for AddressList {
    const NAME: &'static str = "AddressList";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// WARNING: This message is deprecated and is not used.
/// ChannelToChain is used to map a CCV channel ID to the consumer chainID
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelToChain {
    #[prost(string, tag = "1")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for ChannelToChain {
    const NAME: &'static str = "ChannelToChain";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ValidatorSetChangePackets is a pb list of ccv.ValidatorSetChangePacketData.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSetChangePackets {
    #[prost(message, repeated, tag = "1")]
    pub list: ::prost::alloc::vec::Vec<super::super::v1::ValidatorSetChangePacketData>,
}
impl ::prost::Name for ValidatorSetChangePackets {
    const NAME: &'static str = "ValidatorSetChangePackets";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyAssignmentReplacement {
    #[prost(bytes = "vec", tag = "1")]
    pub provider_addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub prev_c_key:
        ::core::option::Option<super::super::super::super::tendermint::crypto::PublicKey>,
    #[prost(int64, tag = "3")]
    pub power: i64,
}
impl ::prost::Name for KeyAssignmentReplacement {
    const NAME: &'static str = "KeyAssignmentReplacement";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// Used to serialize the ValidatorConsumerPubKey index from key assignment
/// ValidatorConsumerPubKey: (chainID, providerAddr consAddr) -> consumerKey
/// tmprotocrypto.PublicKey
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorConsumerPubKey {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub provider_addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub consumer_key:
        ::core::option::Option<super::super::super::super::tendermint::crypto::PublicKey>,
}
impl ::prost::Name for ValidatorConsumerPubKey {
    const NAME: &'static str = "ValidatorConsumerPubKey";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// Used to serialize the ValidatorConsumerAddr index from key assignment
/// ValidatorByConsumerAddr: (chainID, consumerAddr consAddr) -> providerAddr
/// consAddr
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorByConsumerAddr {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub consumer_addr: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub provider_addr: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ValidatorByConsumerAddr {
    const NAME: &'static str = "ValidatorByConsumerAddr";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// Used to serialize the ConsumerAddrsToPruneV2 index from key assignment
/// ConsumerAddrsToPruneV2: (chainID, pruneTs time.Time) -> consumerAddrs AddressList
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerAddrsToPruneV2 {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub prune_ts: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(message, optional, tag = "3")]
    pub consumer_addrs: ::core::option::Option<AddressList>,
}
impl ::prost::Name for ConsumerAddrsToPruneV2 {
    const NAME: &'static str = "ConsumerAddrsToPruneV2";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ConsensusValidator is used to express a validator that
/// should be validating on a chain.
/// It contains relevant info for
/// a validator that is expected to validate on
/// either the provider or a consumer chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusValidator {
    /// validator's consensus address on the provider chain
    #[prost(bytes = "vec", tag = "1")]
    pub provider_cons_addr: ::prost::alloc::vec::Vec<u8>,
    /// voting power the validator has during this epoch
    #[prost(int64, tag = "2")]
    pub power: i64,
    /// public key the validator uses on the consumer chain during this epoch
    #[prost(message, optional, tag = "3")]
    pub public_key:
        ::core::option::Option<super::super::super::super::tendermint::crypto::PublicKey>,
    /// height the validator had when it FIRST became a consumer validator
    /// If a validator becomes a consumer validator at height `H` and is continuously a consumer validator for all the upcoming
    /// epochs, then the height of the validator SHOULD remain `H`. This height only resets to a different height if a validator
    /// stops being a consumer validator during an epoch and later becomes again a consumer validator.
    #[prost(int64, tag = "4")]
    pub join_height: i64,
}
impl ::prost::Name for ConsensusValidator {
    const NAME: &'static str = "ConsensusValidator";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ConsumerRewardsAllocation stores the rewards allocated by a consumer chain
/// to the consumer rewards pool. It is used to allocate the tokens to the consumer
/// opted-in validators and the community pool during BeginBlock.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerRewardsAllocation {
    #[prost(message, repeated, tag = "1")]
    pub rewards:
        ::prost::alloc::vec::Vec<super::super::super::super::cosmos::base::v1beta1::DecCoin>,
}
impl ::prost::Name for ConsumerRewardsAllocation {
    const NAME: &'static str = "ConsumerRewardsAllocation";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ConsumerMetadata contains general information about the registered chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerMetadata {
    /// the name of the chain
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// the description of the chain
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the metadata (e.g., GitHub repository URL) of the chain
    #[prost(string, tag = "3")]
    pub metadata: ::prost::alloc::string::String,
}
impl ::prost::Name for ConsumerMetadata {
    const NAME: &'static str = "ConsumerMetadata";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ConsumerInitializationParameters are the parameters needed to launch a chain
///
/// ---------- ---------- ----------
/// Following fields are used when the consumer chain launches and are not needed by the provider afterwards.
/// ---------- ---------- ----------
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerInitializationParameters {
    /// the proposed initial height of new consumer chain.
    /// For a completely new chain, this will be {0,1}. However, it may be
    /// different if this is a chain that is converting to a consumer chain.
    #[prost(message, optional, tag = "1")]
    pub initial_height:
        ::core::option::Option<super::super::super::super::ibc::core::client::v1::Height>,
    /// The hash of the consumer chain genesis state without the consumer CCV
    /// module genesis params. It is used for off-chain confirmation of
    /// genesis.json validity by validators and other parties.
    #[prost(bytes = "vec", tag = "2")]
    pub genesis_hash: ::prost::alloc::vec::Vec<u8>,
    /// The hash of the consumer chain binary that should be run by validators on
    /// chain initialization. It is used for off-chain confirmation of binary
    /// validity by validators and other parties.
    #[prost(bytes = "vec", tag = "3")]
    pub binary_hash: ::prost::alloc::vec::Vec<u8>,
    /// spawn time is the time on the provider chain at which the consumer chain
    /// genesis is finalized and all validators will be responsible for starting
    /// their consumer chain validator node.
    #[prost(message, optional, tag = "4")]
    pub spawn_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// Unbonding period for the consumer,
    /// which should be smaller than that of the provider in general.
    #[prost(message, optional, tag = "5")]
    pub unbonding_period: ::core::option::Option<::pbjson_types::Duration>,
    /// Sent CCV related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "6")]
    pub ccv_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// Sent transfer related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "7")]
    pub transfer_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// The fraction of tokens allocated to the consumer redistribution address
    /// during distribution events. The fraction is a string representing a
    /// decimal number. For example "0.75" would represent 75%.
    #[prost(string, tag = "8")]
    pub consumer_redistribution_fraction: ::prost::alloc::string::String,
    /// BlocksPerDistributionTransmission is the number of blocks between
    /// ibc-token-transfers from the consumer chain to the provider chain. On
    /// sending transmission event, `consumer_redistribution_fraction` of the
    /// accumulated tokens are sent to the consumer redistribution address.
    #[prost(int64, tag = "9")]
    pub blocks_per_distribution_transmission: i64,
    /// The number of historical info entries to persist in store.
    /// This param is a part of the cosmos sdk staking module. In the case of
    /// a ccv enabled consumer chain, the ccv module acts as the staking module.
    #[prost(int64, tag = "10")]
    pub historical_entries: i64,
    /// The ID of a token transfer channel used for the Reward Distribution
    /// sub-protocol. If DistributionTransmissionChannel == "", a new transfer
    /// channel is created on top of the same connection as the CCV channel.
    /// Note that transfer_channel_id is the ID of the channel end on the consumer
    /// chain. It is most relevant for chains performing a standalone to consumer
    /// changeover in order to maintain the existing ibc transfer channel
    #[prost(string, tag = "11")]
    pub distribution_transmission_channel: ::prost::alloc::string::String,
    /// The ID of the connection end on the provider chain on top of which the CCV
    /// channel will be established. If connection_id == "", a new client of the
    /// consumer chain and a new connection on top of this client are created.
    /// Note that a standalone chain can transition to a consumer chain while
    /// maintaining existing IBC channels to other chains by providing a valid connection_id.
    #[prost(string, tag = "12")]
    pub connection_id: ::prost::alloc::string::String,
}
impl ::prost::Name for ConsumerInitializationParameters {
    const NAME: &'static str = "ConsumerInitializationParameters";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// PowerShapingParameters contains parameters that shape the validator set that we send to the consumer chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PowerShapingParameters {
    /// Corresponds to the percentage of validators that have to validate the chain under the Top N case.
    /// For example, 53 corresponds to a Top 53% chain, meaning that the top 53% provider validators by voting power
    /// have to validate the proposed consumer chain. top_N can either be 0 or any value in \[50, 100\].
    /// A chain can join with top_N == 0 as an Opt In chain, or with top_N ∈ \[50, 100\] as a Top N chain.
    #[prost(uint32, tag = "1")]
    pub top_n: u32,
    /// `validators_power_cap` corresponds to the maximum power (percentage-wise) a validator can have on the consumer chain.
    /// For instance, if `validators_power_cap` is set to 32, no validator can have more than 32% of the total voting power of the
    /// consumer chain. The power cap is intended as a safeguard against a validator having too much power on the consumer
    /// chain and hence "taking over" the consumer chain.
    ///
    /// To respect this power cap, the voting powers of the validators that run the consumer chain are decremented or
    /// incremented accordingly. It is important to note that the voting powers of validators on the provider do **not** change.
    /// For example, assume that the provider chain has among others, validators `A`, `B`, `C`, and `D` with voting powers
    /// 100, 1, 1, 1 respectively. Assume that only those 4 validators opt in on a consumer chain. Without a power cap set,
    /// validator `A` would have 100 / (100 + 1 + 1 + 1) = ~97% of the total voting power on the consumer chain, while
    /// validators `B`, `C`, and `D` would have 1 /(100 + 1 + 1 + 1) = ~1% of the total voting power on the consumer chain.
    /// If `validators_power_cap` is set to 30%, then the voting power of `A` would be reduced from 100 to 30 on the consumer
    /// chain, the voting power of `B` would be increased from 1 to 25, and the power of `C` and `D` would be increased from
    /// 1 to 24. After those modifications, `A` would have 30 / (30 + 25 + 24 + 24) = ~29% of the total voting power of the
    /// consumer chain, `B` would have 25 / (30 + 25 + 24 + 24) = ~25%, and `C` and `D` would both have 24 / (30 + 25 + 24 + 24) = ~23%.
    /// Naturally, there are many ways to change the voting powers of validators to respect the power cap, and ICS chooses
    /// one of them (see the `NoMoreThanPercentOfTheSum` function).
    ///
    /// Note that respecting `validators_power_cap` might NOT always be possible. For example, if we have a consumer
    /// chain with only 5 validators and `validators_power_cap` is set to 10%, then it is not possible to respect the
    /// `validators_power_cap`. If the voting power of each validator is capped to a maximum of 10% of the total consumer
    /// chain's voting power, then the total voting power of the consumer chain would add up to 50% which obviously does not
    /// make sense (percentages should add up to 100%). In cases where it is not feasible to respect the power cap, all
    /// validators on the consumer chain will have equal voting power in order to minimize the power of a single validator.
    /// Thus, in the example of 5 validators and a `validators_power_cap` set to 10%, all validators would end up having 20%
    /// of the total voting power on the consumer chain. Therefore, `validators_power_cap` operates on a best-effort basis.
    /// For more information on the power cap and other power-shaping parameters, please refer to the ICS docs and
    /// specifically `interchain-security/docs/docs/features/power-shaping.md`.
    #[prost(uint32, tag = "2")]
    pub validators_power_cap: u32,
    /// Corresponds to the maximum number of validators that can validate a consumer chain.
    /// Only applicable to Opt In chains. Setting `validator_set_cap` on a Top N chain is a no-op.
    #[prost(uint32, tag = "3")]
    pub validator_set_cap: u32,
    /// corresponds to a list of provider consensus addresses of validators that are the ONLY ones that can validate the consumer chain
    #[prost(string, repeated, tag = "4")]
    pub allowlist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// corresponds to a list of provider consensus addresses of validators that CANNOT validate the consumer chain
    #[prost(string, repeated, tag = "5")]
    pub denylist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Corresponds to the minimal amount of (provider chain) stake required to validate on the consumer chain.
    #[prost(uint64, tag = "6")]
    pub min_stake: u64,
    /// Corresponds to whether inactive validators are allowed to validate the consumer chain.
    #[prost(bool, tag = "7")]
    pub allow_inactive_vals: bool,
    /// Corresponds to a list of provider consensus addresses of validators that should have PRIORITY to validate on the consumer chain,
    /// meaning as long as they are eligible/opted in to validate on the consumer chain, the validator set will be
    /// filled with these validators first, and other validators will be added to the validator set only if there are
    /// not enough eligible priority validators.
    #[prost(string, repeated, tag = "8")]
    pub prioritylist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for PowerShapingParameters {
    const NAME: &'static str = "PowerShapingParameters";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ConsumerIds contains consumer ids of chains
/// Used so we can easily (de)serialize slices of strings
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerIds {
    #[prost(string, repeated, tag = "1")]
    pub ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ConsumerIds {
    const NAME: &'static str = "ConsumerIds";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// AllowlistedRewardDenoms corresponds to the denoms allowlisted by a specific consumer id
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllowlistedRewardDenoms {
    #[prost(string, repeated, tag = "1")]
    pub denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for AllowlistedRewardDenoms {
    const NAME: &'static str = "AllowlistedRewardDenoms";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
///
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InfractionParameters {
    #[prost(message, optional, tag = "1")]
    pub double_sign: ::core::option::Option<SlashJailParameters>,
    #[prost(message, optional, tag = "2")]
    pub downtime: ::core::option::Option<SlashJailParameters>,
}
impl ::prost::Name for InfractionParameters {
    const NAME: &'static str = "InfractionParameters";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
///
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SlashJailParameters {
    #[prost(bytes = "vec", tag = "1")]
    pub slash_fraction: ::prost::alloc::vec::Vec<u8>,
    /// for permanent jailing use 9223372036854775807 which is the largest value a time.Duration can hold (approximately 292 years)
    #[prost(message, optional, tag = "2")]
    pub jail_duration: ::core::option::Option<::pbjson_types::Duration>,
    /// Indicates whether the validator should be tombstoned when slashed
    #[prost(bool, tag = "3")]
    pub tombstone: bool,
}
impl ::prost::Name for SlashJailParameters {
    const NAME: &'static str = "SlashJailParameters";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ConsumerPhase indicates the phases of a consumer chain according to ADR 019
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ConsumerPhase {
    /// UNSPECIFIED defines an empty phase.
    Unspecified = 0,
    /// REGISTERED defines the phase in which a consumer chain has been assigned a unique consumer id.
    /// A chain in this phase cannot yet launch.
    Registered = 1,
    /// INITIALIZED defines the phase in which a consumer chain has set all the needed parameters to launch but
    /// has not yet launched (e.g., because the `spawnTime` of the consumer chain has not yet been reached).
    Initialized = 2,
    /// LAUNCHED defines the phase in which a consumer chain is running and consuming a subset of the validator
    /// set of the provider.
    Launched = 3,
    /// STOPPED defines the phase in which a previously-launched chain has stopped.
    Stopped = 4,
    /// DELETED defines the phase in which the state of a stopped chain has been deleted.
    Deleted = 5,
}
impl ConsumerPhase {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ConsumerPhase::Unspecified => "CONSUMER_PHASE_UNSPECIFIED",
            ConsumerPhase::Registered => "CONSUMER_PHASE_REGISTERED",
            ConsumerPhase::Initialized => "CONSUMER_PHASE_INITIALIZED",
            ConsumerPhase::Launched => "CONSUMER_PHASE_LAUNCHED",
            ConsumerPhase::Stopped => "CONSUMER_PHASE_STOPPED",
            ConsumerPhase::Deleted => "CONSUMER_PHASE_DELETED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CONSUMER_PHASE_UNSPECIFIED" => Some(Self::Unspecified),
            "CONSUMER_PHASE_REGISTERED" => Some(Self::Registered),
            "CONSUMER_PHASE_INITIALIZED" => Some(Self::Initialized),
            "CONSUMER_PHASE_LAUNCHED" => Some(Self::Launched),
            "CONSUMER_PHASE_STOPPED" => Some(Self::Stopped),
            "CONSUMER_PHASE_DELETED" => Some(Self::Deleted),
            _ => None,
        }
    }
}
/// GenesisState defines the CCV provider chain genesis state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// strictly positive and set to 1 (DefaultValsetUpdateID) for a new chain
    #[prost(uint64, tag = "1")]
    pub valset_update_id: u64,
    /// empty for a new chain
    #[prost(message, repeated, tag = "2")]
    pub consumer_states: ::prost::alloc::vec::Vec<ConsumerState>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "5")]
    pub valset_update_id_to_height: ::prost::alloc::vec::Vec<ValsetUpdateIdToHeight>,
    #[prost(message, optional, tag = "8")]
    pub params: ::core::option::Option<Params>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "9")]
    pub validator_consumer_pubkeys: ::prost::alloc::vec::Vec<ValidatorConsumerPubKey>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "10")]
    pub validators_by_consumer_addr: ::prost::alloc::vec::Vec<ValidatorByConsumerAddr>,
    /// empty for a new chain
    #[prost(message, repeated, tag = "14")]
    pub consumer_addrs_to_prune_v2: ::prost::alloc::vec::Vec<ConsumerAddrsToPruneV2>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// The provider CCV module's knowledge of consumer state.
///
/// Note this type is only used internally to the provider CCV module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsumerState {
    /// ChainID defines the chain ID for the consumer chain
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// ChannelID defines the IBC channel ID for the consumer chain
    #[prost(string, tag = "2")]
    pub channel_id: ::prost::alloc::string::String,
    /// ClientID defines the IBC client ID for the consumer chain
    #[prost(string, tag = "3")]
    pub client_id: ::prost::alloc::string::String,
    /// InitalHeight defines the initial block height for the consumer chain
    #[prost(uint64, tag = "4")]
    pub initial_height: u64,
    /// ConsumerGenesis defines the initial consumer chain genesis states
    #[prost(message, optional, tag = "5")]
    pub consumer_genesis: ::core::option::Option<super::super::v1::ConsumerGenesisState>,
    /// PendingValsetChanges defines the pending validator set changes for the
    /// consumer chain
    #[prost(message, repeated, tag = "6")]
    pub pending_valset_changes:
        ::prost::alloc::vec::Vec<super::super::v1::ValidatorSetChangePacketData>,
    #[prost(string, repeated, tag = "7")]
    pub slash_downtime_ack: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// the phase of the consumer chain
    #[prost(enumeration = "ConsumerPhase", tag = "9")]
    pub phase: i32,
}
impl ::prost::Name for ConsumerState {
    const NAME: &'static str = "ConsumerState";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ValsetUpdateIdToHeight defines the genesis information for the mapping
/// of each valset update id to a block height
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValsetUpdateIdToHeight {
    #[prost(uint64, tag = "1")]
    pub valset_update_id: u64,
    #[prost(uint64, tag = "2")]
    pub height: u64,
}
impl ::prost::Name for ValsetUpdateIdToHeight {
    const NAME: &'static str = "ValsetUpdateIdToHeight";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerGenesisRequest {
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerGenesisRequest {
    const NAME: &'static str = "QueryConsumerGenesisRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerGenesisResponse {
    #[prost(message, optional, tag = "1")]
    pub genesis_state: ::core::option::Option<super::super::v1::ConsumerGenesisState>,
}
impl ::prost::Name for QueryConsumerGenesisResponse {
    const NAME: &'static str = "QueryConsumerGenesisResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainsRequest {
    /// The phase of the consumer chains returned (optional)
    /// Registered=1|Initialized=2|Launched=3|Stopped=4|Deleted=5
    #[prost(enumeration = "ConsumerPhase", tag = "1")]
    pub phase: i32,
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
impl ::prost::Name for QueryConsumerChainsRequest {
    const NAME: &'static str = "QueryConsumerChainsRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainsResponse {
    #[prost(message, repeated, tag = "1")]
    pub chains: ::prost::alloc::vec::Vec<Chain>,
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryConsumerChainsResponse {
    const NAME: &'static str = "QueryConsumerChainsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Chain {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(uint32, tag = "3")]
    pub top_n: u32,
    /// If the chain is a Top-N chain, this is the minimum power required to be in the top N.
    /// Otherwise, this is -1.
    #[prost(int64, tag = "4")]
    pub min_power_in_top_n: i64,
    /// Corresponds to the maximum power (percentage-wise) a validator can have on the consumer chain.
    #[prost(uint32, tag = "5")]
    pub validators_power_cap: u32,
    /// Corresponds to the maximum number of validators that can validate a consumer chain.
    /// Only applicable to Opt In chains. Setting `validator_set_cap` on a Top N chain is a no-op.
    #[prost(uint32, tag = "6")]
    pub validator_set_cap: u32,
    /// Corresponds to a list of provider consensus addresses of validators that are the ONLY ones that can validate
    /// the consumer chain.
    #[prost(string, repeated, tag = "7")]
    pub allowlist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Corresponds to a list of provider consensus addresses of validators that CANNOT validate the consumer chain.
    #[prost(string, repeated, tag = "8")]
    pub denylist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The phase the consumer chain
    #[prost(string, tag = "9")]
    pub phase: ::prost::alloc::string::String,
    /// The metadata of the consumer chain
    #[prost(message, optional, tag = "10")]
    pub metadata: ::core::option::Option<ConsumerMetadata>,
    /// Corresponds to the minimal amount of (provider chain) stake required to validate on the consumer chain.
    #[prost(uint64, tag = "11")]
    pub min_stake: u64,
    /// Corresponds to whether inactive validators are allowed to validate the consumer chain.
    #[prost(bool, tag = "12")]
    pub allow_inactive_vals: bool,
    #[prost(string, tag = "13")]
    pub consumer_id: ::prost::alloc::string::String,
    /// the reward denoms allowlisted by this consumer chain
    #[prost(message, optional, tag = "14")]
    pub allowlisted_reward_denoms: ::core::option::Option<AllowlistedRewardDenoms>,
    /// Corresponds to a list of provider consensus addresses of validators that should have PRIORITY to validate on the consumer chain,
    /// meaning as long as they are eligible/opted in to validate on the consumer chain, the validator set will be
    /// filled with these validators first, and other validators will be added to the validator set only if there are
    /// not enough eligible priority validators.
    #[prost(string, repeated, tag = "15")]
    pub prioritylist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Infraction parameters for slashing and jailing
    #[prost(message, optional, tag = "16")]
    pub infraction_parameters: ::core::option::Option<InfractionParameters>,
}
impl ::prost::Name for Chain {
    const NAME: &'static str = "Chain";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorConsumerAddrRequest {
    /// The consensus address of the validator on the provider chain
    #[prost(string, tag = "1")]
    pub provider_address: ::prost::alloc::string::String,
    /// The id of the consumer chain
    #[prost(string, tag = "2")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorConsumerAddrRequest {
    const NAME: &'static str = "QueryValidatorConsumerAddrRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorConsumerAddrResponse {
    /// The address of the validator on the consumer chain
    #[prost(string, tag = "1")]
    pub consumer_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorConsumerAddrResponse {
    const NAME: &'static str = "QueryValidatorConsumerAddrResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorProviderAddrRequest {
    /// The consensus address of the validator on the consumer chain
    #[prost(string, tag = "1")]
    pub consumer_address: ::prost::alloc::string::String,
    /// The id of the consumer chain
    #[prost(string, tag = "2")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorProviderAddrRequest {
    const NAME: &'static str = "QueryValidatorProviderAddrRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorProviderAddrResponse {
    /// The address of the validator on the provider chain
    #[prost(string, tag = "1")]
    pub provider_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorProviderAddrResponse {
    const NAME: &'static str = "QueryValidatorProviderAddrResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryThrottleStateRequest {}
impl ::prost::Name for QueryThrottleStateRequest {
    const NAME: &'static str = "QueryThrottleStateRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryThrottleStateResponse {
    /// current slash_meter state
    #[prost(int64, tag = "1")]
    pub slash_meter: i64,
    /// allowance of voting power units (int) that the slash meter is given per
    /// replenish period this also serves as the max value for the meter.
    #[prost(int64, tag = "2")]
    pub slash_meter_allowance: i64,
    /// next time the slash meter could potentially be replenished, iff it's not
    /// full
    #[prost(message, optional, tag = "3")]
    pub next_replenish_candidate: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for QueryThrottleStateResponse {
    const NAME: &'static str = "QueryThrottleStateResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRegisteredConsumerRewardDenomsRequest {}
impl ::prost::Name for QueryRegisteredConsumerRewardDenomsRequest {
    const NAME: &'static str = "QueryRegisteredConsumerRewardDenomsRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRegisteredConsumerRewardDenomsResponse {
    #[prost(string, repeated, tag = "1")]
    pub denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for QueryRegisteredConsumerRewardDenomsResponse {
    const NAME: &'static str = "QueryRegisteredConsumerRewardDenomsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllPairsValConsAddrByConsumerRequest {
    /// The id of the consumer chain
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryAllPairsValConsAddrByConsumerRequest {
    const NAME: &'static str = "QueryAllPairsValConsAddrByConsumerRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllPairsValConsAddrByConsumerResponse {
    #[prost(message, repeated, tag = "1")]
    pub pair_val_con_addr: ::prost::alloc::vec::Vec<PairValConAddrProviderAndConsumer>,
}
impl ::prost::Name for QueryAllPairsValConsAddrByConsumerResponse {
    const NAME: &'static str = "QueryAllPairsValConsAddrByConsumerResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PairValConAddrProviderAndConsumer {
    /// The consensus address of the validator on the provider chain
    #[prost(string, tag = "1")]
    pub provider_address: ::prost::alloc::string::String,
    /// The consensus address of the validator on the consumer chain
    #[prost(string, tag = "2")]
    pub consumer_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub consumer_key:
        ::core::option::Option<super::super::super::super::tendermint::crypto::PublicKey>,
}
impl ::prost::Name for PairValConAddrProviderAndConsumer {
    const NAME: &'static str = "PairValConAddrProviderAndConsumer";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainOptedInValidatorsRequest {
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerChainOptedInValidatorsRequest {
    const NAME: &'static str = "QueryConsumerChainOptedInValidatorsRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainOptedInValidatorsResponse {
    /// The consensus addresses of the validators on the provider chain
    #[prost(string, repeated, tag = "1")]
    pub validators_provider_addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for QueryConsumerChainOptedInValidatorsResponse {
    const NAME: &'static str = "QueryConsumerChainOptedInValidatorsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerValidatorsRequest {
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerValidatorsRequest {
    const NAME: &'static str = "QueryConsumerValidatorsRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerValidatorsValidator {
    /// The consensus address of the validator on the provider chain
    #[prost(string, tag = "1")]
    pub provider_address: ::prost::alloc::string::String,
    /// The consumer public key of the validator used on the consumer chain
    #[prost(message, optional, tag = "2")]
    pub consumer_key:
        ::core::option::Option<super::super::super::super::tendermint::crypto::PublicKey>,
    /// \[DEPRECATED\] use `consumer_power` instead
    #[deprecated]
    #[prost(int64, tag = "3")]
    pub power: i64,
    /// \[DEPRECATED\] use `consumer_commission_rate` instead
    #[deprecated]
    #[prost(string, tag = "4")]
    pub rate: ::prost::alloc::string::String,
    /// The power of the validator used on the consumer chain
    #[prost(int64, tag = "5")]
    pub consumer_power: i64,
    /// The rate to charge delegators on the consumer chain, as a fraction
    #[prost(string, tag = "6")]
    pub consumer_commission_rate: ::prost::alloc::string::String,
    /// The rate to charge delegators on the provider chain, as a fraction
    #[prost(string, tag = "7")]
    pub provider_commission_rate: ::prost::alloc::string::String,
    /// description defines the description terms for the validator
    #[prost(message, optional, tag = "8")]
    pub description:
        ::core::option::Option<super::super::super::super::cosmos::staking::v1beta1::Description>,
    /// provider_operator_address defines the address of the validator's operator
    #[prost(string, tag = "9")]
    pub provider_operator_address: ::prost::alloc::string::String,
    /// jailed defined whether the validator has been jailed from bonded status or not.
    #[prost(bool, tag = "10")]
    pub jailed: bool,
    /// status is the validator status (bonded/unbonding/unbonded).
    #[prost(
        enumeration = "super::super::super::super::cosmos::staking::v1beta1::BondStatus",
        tag = "11"
    )]
    pub status: i32,
    /// provider_tokens defines the delegated tokens (incl. self-delegation).
    #[prost(string, tag = "12")]
    pub provider_tokens: ::prost::alloc::string::String,
    /// The power of the validator used on the provider chain
    #[prost(int64, tag = "13")]
    pub provider_power: i64,
    /// validates_current_epoch defines whether the validator has to validate for the current epoch or not
    #[prost(bool, tag = "14")]
    pub validates_current_epoch: bool,
}
impl ::prost::Name for QueryConsumerValidatorsValidator {
    const NAME: &'static str = "QueryConsumerValidatorsValidator";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerValidatorsResponse {
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<QueryConsumerValidatorsValidator>,
}
impl ::prost::Name for QueryConsumerValidatorsResponse {
    const NAME: &'static str = "QueryConsumerValidatorsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainsValidatorHasToValidateRequest {
    /// The consensus address of the validator on the provider chain
    #[prost(string, tag = "1")]
    pub provider_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerChainsValidatorHasToValidateRequest {
    const NAME: &'static str = "QueryConsumerChainsValidatorHasToValidateRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainsValidatorHasToValidateResponse {
    #[prost(string, repeated, tag = "1")]
    pub consumer_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for QueryConsumerChainsValidatorHasToValidateResponse {
    const NAME: &'static str = "QueryConsumerChainsValidatorHasToValidateResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorConsumerCommissionRateRequest {
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
    /// The consensus address of the validator on the provider chain
    #[prost(string, tag = "2")]
    pub provider_address: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorConsumerCommissionRateRequest {
    const NAME: &'static str = "QueryValidatorConsumerCommissionRateRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorConsumerCommissionRateResponse {
    /// The rate to charge delegators on the consumer chain, as a fraction
    #[prost(string, tag = "1")]
    pub rate: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryValidatorConsumerCommissionRateResponse {
    const NAME: &'static str = "QueryValidatorConsumerCommissionRateResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBlocksUntilNextEpochRequest {}
impl ::prost::Name for QueryBlocksUntilNextEpochRequest {
    const NAME: &'static str = "QueryBlocksUntilNextEpochRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBlocksUntilNextEpochResponse {
    /// The number of blocks until the next epoch starts
    #[prost(uint64, tag = "1")]
    pub blocks_until_next_epoch: u64,
}
impl ::prost::Name for QueryBlocksUntilNextEpochResponse {
    const NAME: &'static str = "QueryBlocksUntilNextEpochResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerIdFromClientIdRequest {
    /// the client id (on the provider) that is tracking the consumer chain
    /// the client id can be found from the consumer chain by querying (i.e., `query ccvconsumer provider-info`)
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerIdFromClientIdRequest {
    const NAME: &'static str = "QueryConsumerIdFromClientIdRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerIdFromClientIdResponse {
    /// the consumer id of the chain associated with this client id
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerIdFromClientIdResponse {
    const NAME: &'static str = "QueryConsumerIdFromClientIdResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainRequest {
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerChainRequest {
    const NAME: &'static str = "QueryConsumerChainRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerChainResponse {
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub owner_address: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub phase: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "5")]
    pub metadata: ::core::option::Option<ConsumerMetadata>,
    #[prost(message, optional, tag = "6")]
    pub init_params: ::core::option::Option<ConsumerInitializationParameters>,
    #[prost(message, optional, tag = "7")]
    pub power_shaping_params: ::core::option::Option<PowerShapingParameters>,
    #[prost(message, optional, tag = "8")]
    pub infraction_parameters: ::core::option::Option<InfractionParameters>,
    /// corresponds to the id of the client that is created during launch
    #[prost(string, tag = "9")]
    pub client_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerChainResponse {
    const NAME: &'static str = "QueryConsumerChainResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerGenesisTimeRequest {
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConsumerGenesisTimeRequest {
    const NAME: &'static str = "QueryConsumerGenesisTimeRequest";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsumerGenesisTimeResponse {
    #[prost(message, optional, tag = "1")]
    pub genesis_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for QueryConsumerGenesisTimeResponse {
    const NAME: &'static str = "QueryConsumerGenesisTimeResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAssignConsumerKey {
    /// \[DEPRECATED\] use `consumer_id` instead
    #[deprecated]
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The validator address on the provider
    #[prost(string, tag = "2")]
    pub provider_addr: ::prost::alloc::string::String,
    /// The consensus public key to use on the consumer.
    /// in json string format corresponding to proto-any, ex:
    /// `{"@type":"/cosmos.crypto.ed25519.PubKey","key":"Ui5Gf1+mtWUdH8u3xlmzdKID+F3PK0sfXZ73GZ6q6is="}`
    #[prost(string, tag = "3")]
    pub consumer_key: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub signer: ::prost::alloc::string::String,
    /// the consumer id of the consumer chain to assign a consensus public key to
    #[prost(string, tag = "5")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgAssignConsumerKey {
    const NAME: &'static str = "MsgAssignConsumerKey";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAssignConsumerKeyResponse {}
impl ::prost::Name for MsgAssignConsumerKeyResponse {
    const NAME: &'static str = "MsgAssignConsumerKeyResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgSubmitConsumerMisbehaviour defines a message that reports a light client attack,
/// also known as a misbehaviour, observed on a consumer chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitConsumerMisbehaviour {
    #[prost(string, tag = "1")]
    pub submitter: ::prost::alloc::string::String,
    /// The Misbehaviour of the consumer chain wrapping
    /// two conflicting IBC headers
    #[prost(message, optional, tag = "2")]
    pub misbehaviour: ::core::option::Option<
        super::super::super::super::ibc::lightclients::tendermint::v1::Misbehaviour,
    >,
    /// the consumer id of the consumer chain where the misbehaviour occurred
    #[prost(string, tag = "3")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSubmitConsumerMisbehaviour {
    const NAME: &'static str = "MsgSubmitConsumerMisbehaviour";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitConsumerMisbehaviourResponse {}
impl ::prost::Name for MsgSubmitConsumerMisbehaviourResponse {
    const NAME: &'static str = "MsgSubmitConsumerMisbehaviourResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgSubmitConsumerDoubleVoting defines a message that reports
/// a double signing infraction observed on a consumer chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitConsumerDoubleVoting {
    #[prost(string, tag = "1")]
    pub submitter: ::prost::alloc::string::String,
    /// The equivocation of the consumer chain wrapping
    /// an evidence of a validator that signed two conflicting votes
    #[prost(message, optional, tag = "2")]
    pub duplicate_vote_evidence: ::core::option::Option<
        super::super::super::super::tendermint::types::DuplicateVoteEvidence,
    >,
    /// The light client header of the infraction block
    #[prost(message, optional, tag = "3")]
    pub infraction_block_header: ::core::option::Option<
        super::super::super::super::ibc::lightclients::tendermint::v1::Header,
    >,
    /// the consumer id of the consumer chain where the double-voting took place
    #[prost(string, tag = "4")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSubmitConsumerDoubleVoting {
    const NAME: &'static str = "MsgSubmitConsumerDoubleVoting";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitConsumerDoubleVotingResponse {}
impl ::prost::Name for MsgSubmitConsumerDoubleVotingResponse {
    const NAME: &'static str = "MsgSubmitConsumerDoubleVotingResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the Msg/UpdateParams request type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address of the governance account.
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/provider parameters to update.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// \[DEPRECATED\] Use `MsgCreateConsumer` instead
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConsumerAddition {
    /// the proposed chain-id of the new consumer chain, must be different from all
    /// other consumer chain ids of the executing provider chain.
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// the proposed initial height of new consumer chain.
    /// For a completely new chain, this will be {0,1}. However, it may be
    /// different if this is a chain that is converting to a consumer chain.
    #[prost(message, optional, tag = "2")]
    pub initial_height:
        ::core::option::Option<super::super::super::super::ibc::core::client::v1::Height>,
    /// The hash of the consumer chain genesis state without the consumer CCV
    /// module genesis params. It is used for off-chain confirmation of
    /// genesis.json validity by validators and other parties.
    #[prost(bytes = "vec", tag = "3")]
    pub genesis_hash: ::prost::alloc::vec::Vec<u8>,
    /// The hash of the consumer chain binary that should be run by validators on
    /// chain initialization. It is used for off-chain confirmation of binary
    /// validity by validators and other parties.
    #[prost(bytes = "vec", tag = "4")]
    pub binary_hash: ::prost::alloc::vec::Vec<u8>,
    /// spawn time is the time on the provider chain at which the consumer chain
    /// genesis is finalized and all validators will be responsible for starting
    /// their consumer chain validator node.
    #[prost(message, optional, tag = "5")]
    pub spawn_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// Unbonding period for the consumer,
    /// which should be smaller than that of the provider in general.
    #[prost(message, optional, tag = "6")]
    pub unbonding_period: ::core::option::Option<::pbjson_types::Duration>,
    /// Sent CCV related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "7")]
    pub ccv_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// Sent transfer related IBC packets will timeout after this duration
    #[prost(message, optional, tag = "8")]
    pub transfer_timeout_period: ::core::option::Option<::pbjson_types::Duration>,
    /// The fraction of tokens allocated to the consumer redistribution address
    /// during distribution events. The fraction is a string representing a
    /// decimal number. For example "0.75" would represent 75%.
    #[prost(string, tag = "9")]
    pub consumer_redistribution_fraction: ::prost::alloc::string::String,
    /// BlocksPerDistributionTransmission is the number of blocks between
    /// ibc-token-transfers from the consumer chain to the provider chain. On
    /// sending transmission event, `consumer_redistribution_fraction` of the
    /// accumulated tokens are sent to the consumer redistribution address.
    #[prost(int64, tag = "10")]
    pub blocks_per_distribution_transmission: i64,
    /// The number of historical info entries to persist in store.
    /// This param is a part of the cosmos sdk staking module. In the case of
    /// a ccv enabled consumer chain, the ccv module acts as the staking module.
    #[prost(int64, tag = "11")]
    pub historical_entries: i64,
    /// The ID of a token transfer channel used for the Reward Distribution
    /// sub-protocol. If DistributionTransmissionChannel == "", a new transfer
    /// channel is created on top of the same connection as the CCV channel.
    /// Note that transfer_channel_id is the ID of the channel end on the consumer
    /// chain. it is most relevant for chains performing a sovereign to consumer
    /// changeover in order to maintain the existing ibc transfer channel
    #[prost(string, tag = "12")]
    pub distribution_transmission_channel: ::prost::alloc::string::String,
    /// Corresponds to the percentage of validators that have to validate the chain under the Top N case.
    /// For example, 53 corresponds to a Top 53% chain, meaning that the top 53% provider validators by voting power
    /// have to validate the proposed consumer chain. top_N can either be 0 or any value in \[50, 100\].
    /// A chain can join with top_N == 0 as an Opt In chain, or with top_N ∈ \[50, 100\] as a Top N chain.
    #[prost(uint32, tag = "13")]
    pub top_n: u32,
    /// Corresponds to the maximum power (percentage-wise) a validator can have on the consumer chain. For instance, if
    /// `validators_power_cap` is set to 32, it means that no validator can have more than 32% of the voting power on the
    /// consumer chain. Note that this might not be feasible. For example, think of a consumer chain with only
    /// 5 validators and with `validators_power_cap` set to 10%. In such a scenario, at least one validator would need
    /// to have more than 20% of the total voting power. Therefore, `validators_power_cap` operates on a best-effort basis.
    #[prost(uint32, tag = "14")]
    pub validators_power_cap: u32,
    /// Corresponds to the maximum number of validators that can validate a consumer chain.
    /// Only applicable to Opt In chains. Setting `validator_set_cap` on a Top N chain is a no-op.
    #[prost(uint32, tag = "15")]
    pub validator_set_cap: u32,
    /// Corresponds to a list of provider consensus addresses of validators that are the ONLY ones that can validate
    /// the consumer chain.
    #[prost(string, repeated, tag = "16")]
    pub allowlist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Corresponds to a list of provider consensus addresses of validators that CANNOT validate the consumer chain.
    #[prost(string, repeated, tag = "17")]
    pub denylist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// signer address
    #[prost(string, tag = "18")]
    pub authority: ::prost::alloc::string::String,
    /// Corresponds to the minimal amount of (provider chain) stake required to validate on the consumer chain.
    #[prost(uint64, tag = "19")]
    pub min_stake: u64,
    /// Corresponds to whether inactive validators are allowed to validate the consumer chain.
    #[prost(bool, tag = "20")]
    pub allow_inactive_vals: bool,
}
impl ::prost::Name for MsgConsumerAddition {
    const NAME: &'static str = "MsgConsumerAddition";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// \[DEPRECATED\] Use `MsgRemoveConsumer` instead
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConsumerRemoval {
    /// the chain-id of the consumer chain to be stopped
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// the time on the provider chain at which all validators are responsible to
    /// stop their consumer chain validator node
    #[prost(message, optional, tag = "2")]
    pub stop_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// signer address
    #[prost(string, tag = "3")]
    pub authority: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgConsumerRemoval {
    const NAME: &'static str = "MsgConsumerRemoval";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgRemoveConsumer defines the message used to remove (and stop) a consumer chain.
/// If it passes, all the consumer chain's state is eventually removed from the provider chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveConsumer {
    /// the consumer id of the consumer chain to be stopped
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
    /// the address of the owner of the consumer chain to be stopped
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgRemoveConsumer {
    const NAME: &'static str = "MsgRemoveConsumer";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgRemoveConsumerResponse defines response type for MsgRemoveConsumer messages
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveConsumerResponse {}
impl ::prost::Name for MsgRemoveConsumerResponse {
    const NAME: &'static str = "MsgRemoveConsumerResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// ChangeRewardDenomsProposal is a governance proposal on the provider chain to
/// mutate the set of denoms accepted by the provider as rewards.
///
/// Note: this replaces ChangeRewardDenomsProposal which is deprecated and will be removed soon
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChangeRewardDenoms {
    /// the list of consumer reward denoms to add
    #[prost(string, repeated, tag = "1")]
    pub denoms_to_add: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// the list of consumer reward denoms to remove
    #[prost(string, repeated, tag = "2")]
    pub denoms_to_remove: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// authority is the address of the governance account
    #[prost(string, tag = "3")]
    pub authority: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgChangeRewardDenoms {
    const NAME: &'static str = "MsgChangeRewardDenoms";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgChangeRewardDenomsResponse defines response type for MsgChangeRewardDenoms messages
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChangeRewardDenomsResponse {}
impl ::prost::Name for MsgChangeRewardDenomsResponse {
    const NAME: &'static str = "MsgChangeRewardDenomsResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgOptIn {
    /// \[DEPRECATED\] use `consumer_id` instead
    #[deprecated]
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// the validator address on the provider
    #[prost(string, tag = "2")]
    pub provider_addr: ::prost::alloc::string::String,
    /// (optional) The consensus public key to use on the consumer in json string format corresponding to proto-any,
    /// for example `{"@type":"/cosmos.crypto.ed25519.PubKey","key":"Ui5Gf1+mtWUdH8u3xlmzdKID+F3PK0sfXZ73GZ6q6is="}`.
    /// This field is optional and can remain empty (i.e., `consumer_key = ""`). A validator can always change the
    /// consumer public key at a later stage by issuing a `MsgAssignConsumerKey` message.
    #[prost(string, tag = "3")]
    pub consumer_key: ::prost::alloc::string::String,
    /// submitter address
    #[prost(string, tag = "4")]
    pub signer: ::prost::alloc::string::String,
    /// the consumer id of the consumer chain to opt in to
    #[prost(string, tag = "5")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgOptIn {
    const NAME: &'static str = "MsgOptIn";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgOptInResponse {}
impl ::prost::Name for MsgOptInResponse {
    const NAME: &'static str = "MsgOptInResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgOptOut {
    /// \[DEPRECATED\] use `consumer_id` instead
    #[deprecated]
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// the validator address on the provider
    #[prost(string, tag = "2")]
    pub provider_addr: ::prost::alloc::string::String,
    /// submitter address
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
    /// the consumer id of the consumer chain to opt out from
    #[prost(string, tag = "4")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgOptOut {
    const NAME: &'static str = "MsgOptOut";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgOptOutResponse {}
impl ::prost::Name for MsgOptOutResponse {
    const NAME: &'static str = "MsgOptOutResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgSetConsumerCommissionRate allows validators to set
/// a per-consumer chain commission rate
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetConsumerCommissionRate {
    /// The validator address on the provider
    #[prost(string, tag = "1")]
    pub provider_addr: ::prost::alloc::string::String,
    /// \[DEPRECATED\] use `consumer_id` instead
    #[deprecated]
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    /// The rate to charge delegators on the consumer chain, as a fraction
    /// TODO: migrate rate from sdk.Dec to math.LegacyDec
    #[prost(string, tag = "3")]
    pub rate: ::prost::alloc::string::String,
    /// submitter address
    #[prost(string, tag = "4")]
    pub signer: ::prost::alloc::string::String,
    /// the consumer id of the consumer chain to set the commission rate
    #[prost(string, tag = "5")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSetConsumerCommissionRate {
    const NAME: &'static str = "MsgSetConsumerCommissionRate";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetConsumerCommissionRateResponse {}
impl ::prost::Name for MsgSetConsumerCommissionRateResponse {
    const NAME: &'static str = "MsgSetConsumerCommissionRateResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// \[DEPRECATED\] Use `MsgUpdateConsumer` instead
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConsumerModification {
    /// the title of the proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the chain-id of the consumer chain to be modified
    #[prost(string, tag = "3")]
    pub chain_id: ::prost::alloc::string::String,
    /// Corresponds to the percentage of validators that have to validate the chain under the Top N case.
    /// For example, 53 corresponds to a Top 53% chain, meaning that the top 53% provider validators by voting power
    /// have to validate the proposed consumer chain. top_N can either be 0 or any value in \[50, 100\].
    /// A chain can join with top_N == 0 as an Opt In chain, or with top_N ∈ \[50, 100\] as a Top N chain.
    #[prost(uint32, tag = "4")]
    pub top_n: u32,
    /// Corresponds to the maximum power (percentage-wise) a validator can have on the consumer chain. For instance, if
    /// `validators_power_cap` is set to 32, it means that no validator can have more than 32% of the voting power on the
    /// consumer chain. Note that this might not be feasible. For example, think of a consumer chain with only
    /// 5 validators and with `validators_power_cap` set to 10%. In such a scenario, at least one validator would need
    /// to have more than 20% of the total voting power. Therefore, `validators_power_cap` operates on a best-effort basis.
    #[prost(uint32, tag = "5")]
    pub validators_power_cap: u32,
    /// Corresponds to the maximum number of validators that can validate a consumer chain.
    /// Only applicable to Opt In chains. Setting `validator_set_cap` on a Top N chain is a no-op.
    #[prost(uint32, tag = "6")]
    pub validator_set_cap: u32,
    /// Corresponds to a list of provider consensus addresses of validators that are the ONLY ones that can validate
    /// the consumer chain.
    #[prost(string, repeated, tag = "7")]
    pub allowlist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Corresponds to a list of provider consensus addresses of validators that CANNOT validate the consumer chain.
    #[prost(string, repeated, tag = "8")]
    pub denylist: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// signer address
    #[prost(string, tag = "9")]
    pub authority: ::prost::alloc::string::String,
    /// Corresponds to the minimal amount of (provider chain) stake required to validate on the consumer chain.
    #[prost(uint64, tag = "10")]
    pub min_stake: u64,
    /// Corresponds to whether inactive validators are allowed to validate the consumer chain.
    #[prost(bool, tag = "11")]
    pub allow_inactive_vals: bool,
}
impl ::prost::Name for MsgConsumerModification {
    const NAME: &'static str = "MsgConsumerModification";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConsumerModificationResponse {}
impl ::prost::Name for MsgConsumerModificationResponse {
    const NAME: &'static str = "MsgConsumerModificationResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgCreateConsumer defines the message that creates a consumer chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateConsumer {
    /// Submitter address. If the message is successfully handled, the ownership of
    /// the consumer chain will given to this address.
    #[prost(string, tag = "1")]
    pub submitter: ::prost::alloc::string::String,
    /// the chain id of the new consumer chain
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub metadata: ::core::option::Option<ConsumerMetadata>,
    #[prost(message, optional, tag = "4")]
    pub initialization_parameters: ::core::option::Option<ConsumerInitializationParameters>,
    #[prost(message, optional, tag = "5")]
    pub power_shaping_parameters: ::core::option::Option<PowerShapingParameters>,
    /// allowlisted reward denoms of the consumer
    #[prost(message, optional, tag = "6")]
    pub allowlisted_reward_denoms: ::core::option::Option<AllowlistedRewardDenoms>,
    /// infraction parameters for slashing and jailing
    #[prost(message, optional, tag = "7")]
    pub infraction_parameters: ::core::option::Option<InfractionParameters>,
}
impl ::prost::Name for MsgCreateConsumer {
    const NAME: &'static str = "MsgCreateConsumer";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgCreateConsumerResponse defines response type for MsgCreateConsumer
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateConsumerResponse {
    #[prost(string, tag = "1")]
    pub consumer_id: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCreateConsumerResponse {
    const NAME: &'static str = "MsgCreateConsumerResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgUpdateConsumer defines the message used to modify a consumer chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateConsumer {
    /// the address of the owner of the consumer chain to be updated
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    /// the consumer id of the consumer chain to be updated
    #[prost(string, tag = "2")]
    pub consumer_id: ::prost::alloc::string::String,
    /// the new owner of the consumer when updated
    #[prost(string, tag = "3")]
    pub new_owner_address: ::prost::alloc::string::String,
    /// the metadata of the consumer when updated
    #[prost(message, optional, tag = "4")]
    pub metadata: ::core::option::Option<ConsumerMetadata>,
    /// initialization parameters can only be updated before a chain has launched
    #[prost(message, optional, tag = "5")]
    pub initialization_parameters: ::core::option::Option<ConsumerInitializationParameters>,
    /// the power-shaping parameters of the consumer when updated
    #[prost(message, optional, tag = "6")]
    pub power_shaping_parameters: ::core::option::Option<PowerShapingParameters>,
    /// allowlisted reward denoms of the consumer (if provided they overwrite previously set reward denoms)
    #[prost(message, optional, tag = "7")]
    pub allowlisted_reward_denoms: ::core::option::Option<AllowlistedRewardDenoms>,
    /// (optional) If the consumer chain has NOT yet launched, the chain id can be updated. After a chain has launched
    /// the chain id CANNOT be updated.
    /// This field is optional and can remain empty (i.e., `new_chain_id = ""`) or correspond to the chain id the chain already has.
    #[prost(string, tag = "8")]
    pub new_chain_id: ::prost::alloc::string::String,
    /// infraction parameters for slashing and jailing
    #[prost(message, optional, tag = "9")]
    pub infraction_parameters: ::core::option::Option<InfractionParameters>,
}
impl ::prost::Name for MsgUpdateConsumer {
    const NAME: &'static str = "MsgUpdateConsumer";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
/// MsgUpdateConsumerResponse defines response type for MsgUpdateConsumer messages
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateConsumerResponse {}
impl ::prost::Name for MsgUpdateConsumerResponse {
    const NAME: &'static str = "MsgUpdateConsumerResponse";
    const PACKAGE: &'static str = "interchain_security.ccv.provider.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("interchain_security.ccv.provider.v1.{}", Self::NAME)
    }
}
include!("interchain_security.ccv.provider.v1.tonic.rs");
// @@protoc_insertion_point(module)
