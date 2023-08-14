use std::num::ParseIntError;

#[cfg(feature = "ethabi")]
use crate::EthAbi;
use crate::{Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq)]
pub struct FungibleTokenPacketData {
    /// the token denomination to be transferred
    pub denom: String,
    /// the token amount to be transferred
    pub amount: u64,
    /// the sender address
    pub sender: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// optional memo
    pub memo: Option<String>,
}

impl Proto for FungibleTokenPacketData {
    type Proto = protos::ibc::applications::transfer::v2::FungibleTokenPacketData;
}

impl TypeUrl for protos::ibc::applications::transfer::v2::FungibleTokenPacketData {
    const TYPE_URL: &'static str = "/ibc.applications.transfer.v2.FungibleTokenPacketData";
}

#[derive(Debug)]
pub enum TryFromFungibleTokenPacketDataError {
    Amount(ParseIntError),
}

impl TryFrom<protos::ibc::applications::transfer::v2::FungibleTokenPacketData>
    for FungibleTokenPacketData
{
    type Error = TryFromFungibleTokenPacketDataError;

    fn try_from(
        value: protos::ibc::applications::transfer::v2::FungibleTokenPacketData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: value.denom,
            amount: value
                .amount
                .parse()
                .map_err(TryFromFungibleTokenPacketDataError::Amount)?,
            sender: value.sender,
            receiver: value.receiver,
            memo: (!value.memo.is_empty()).then_some(value.memo),
        })
    }
}

impl From<FungibleTokenPacketData>
    for protos::ibc::applications::transfer::v2::FungibleTokenPacketData
{
    fn from(value: FungibleTokenPacketData) -> Self {
        Self {
            denom: value.denom,
            amount: value.amount.to_string(),
            sender: value.sender,
            receiver: value.receiver,
            memo: value.memo.unwrap_or_default(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl EthAbi for FungibleTokenPacketData {
    type EthAbi = contracts::glue::TransferPacket;
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromTransferPacketError {
    Amount(&'static str),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TransferPacket> for FungibleTokenPacketData {
    type Error = TryFromTransferPacketError;

    fn try_from(value: contracts::glue::TransferPacket) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: value.denom,
            amount: value
                .amount
                .try_into()
                .map_err(TryFromTransferPacketError::Amount)?,
            sender: value.sender,
            receiver: value.receiver,
            memo: None,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<FungibleTokenPacketData> for contracts::glue::TransferPacket {
    fn from(value: FungibleTokenPacketData) -> Self {
        if value.memo.is_some() {
            tracing::warn!(
                data = ?value,
                "memo on FungibleTokenPacketData silently ignored as TransferPacket does not support memo"
            );
        }

        Self {
            amount: value.amount.into(),
            denom: value.denom,
            receiver: value.receiver,
            sender: value.sender,
        }
    }
}
