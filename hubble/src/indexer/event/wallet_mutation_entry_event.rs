use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{ContractAddress, MutationAmount, MutationDirection, WalletAddress},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WalletMutationEntryEvent {
    #[serde(flatten)]
    pub header: Header,
    pub contract_address_canonical: ContractAddress,
    pub wallet_address_canonical: WalletAddress,
    pub amount: MutationAmount,
    pub direction: MutationDirection,
}
