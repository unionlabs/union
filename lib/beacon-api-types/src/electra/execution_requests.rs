#[cfg(feature = "ssz")]
use {
    crate::chain_spec::ChainSpec,
    ssz::{types::List, Ssz},
};

use crate::electra::{ConsolidationRequest, DepositRequest, WithdrawalRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct ExecutionRequests {
    pub deposits: Vec<DepositRequest>,
    pub withdrawals: Vec<WithdrawalRequest>,
    pub consolidations: Vec<ConsolidationRequest>,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct ExecutionRequestsSsz<C: ChainSpec> {
    pub deposits: List<DepositRequest, C::MAX_DEPOSIT_REQUESTS_PER_PAYLOAD>,
    pub withdrawals: List<WithdrawalRequest, C::MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD>,
    pub consolidations: List<ConsolidationRequest, C::MAX_CONSOLIDATION_REQUESTS_PER_PAYLOAD>,
}
