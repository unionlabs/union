use macros::model;
use rlp::Encodable;
use sha2::Digest;
use sha3::Keccak256;

use crate::{
    errors::InvalidLength,
    hash::{H160, H2048, H256, H64},
    uint::U256,
};

#[model(proto(
    raw(protos::union::ibc::lightclients::arbitrum::v1::L2Header),
    into,
    from
))]
#[derive(rlp::RlpEncodable)]
pub struct L2Header {
    pub parent_hash: H256,
    pub sha3_uncles: H256,
    pub miner: H160,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    // Box since 256 bytes is quite large
    pub logs_bloom: Box<H2048>,
    pub difficulty: U256,
    pub number: U256,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub extra_data: H256,
    pub mix_hash: H256,
    pub nonce: H64,
    pub base_fee_per_gas: U256,
}

impl L2Header {
    #[must_use]
    pub fn hash(&self) -> H256 {
        H256::from(Keccak256::new().chain_update(self.rlp_bytes()).finalize())
    }
}

impl TryFrom<protos::union::ibc::lightclients::arbitrum::v1::L2Header> for L2Header {
    type Error = TryFromL2HeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::arbitrum::v1::L2Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            parent_hash: value
                .parent_hash
                .try_into()
                .map_err(TryFromL2HeaderError::ParentHash)?,
            sha3_uncles: value
                .sha3_uncles
                .try_into()
                .map_err(TryFromL2HeaderError::Sha3Uncles)?,
            miner: value
                .miner
                .try_into()
                .map_err(TryFromL2HeaderError::Miner)?,
            state_root: value
                .state_root
                .try_into()
                .map_err(TryFromL2HeaderError::StateRoot)?,
            transactions_root: value
                .transactions_root
                .try_into()
                .map_err(TryFromL2HeaderError::TransactionsRoot)?,
            receipts_root: value
                .receipts_root
                .try_into()
                .map_err(TryFromL2HeaderError::ReceiptRoot)?,
            logs_bloom: value
                .logs_bloom
                .try_into()
                .map(Box::new)
                .map_err(TryFromL2HeaderError::LogsBloom)?,
            difficulty: U256::try_from_be_bytes(&value.difficulty)
                .map_err(TryFromL2HeaderError::Difficulty)?,
            number: U256::try_from_be_bytes(&value.number).map_err(TryFromL2HeaderError::Number)?,
            gas_limit: value.gas_limit,
            gas_used: value.gas_used,
            timestamp: value.timestamp,
            extra_data: value
                .extra_data
                .try_into()
                .map_err(TryFromL2HeaderError::ExtraData)?,
            mix_hash: value
                .mix_hash
                .try_into()
                .map_err(TryFromL2HeaderError::MixHash)?,
            nonce: value
                .nonce
                .try_into()
                .map_err(TryFromL2HeaderError::Nonce)?,
            base_fee_per_gas: U256::try_from_be_bytes(&value.base_fee_per_gas)
                .map_err(TryFromL2HeaderError::BaseFeePerGas)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromL2HeaderError {
    #[error("invalid parent hash")]
    ParentHash(#[source] InvalidLength),
    #[error("invalid sha3 uncles")]
    Sha3Uncles(#[source] InvalidLength),
    #[error("invalid miner")]
    Miner(#[source] InvalidLength),
    #[error("invalid state root")]
    StateRoot(#[source] InvalidLength),
    #[error("invalid transactions root")]
    TransactionsRoot(#[source] InvalidLength),
    #[error("invalid receipt root")]
    ReceiptRoot(#[source] InvalidLength),
    #[error("invalid logs bloom")]
    LogsBloom(#[source] InvalidLength),
    #[error("invalid difficulty")]
    Difficulty(#[source] InvalidLength),
    #[error("invalid number")]
    Number(#[source] InvalidLength),
    #[error("invalid extra data")]
    ExtraData(#[source] InvalidLength),
    #[error("invalid mix hash")]
    MixHash(#[source] InvalidLength),
    #[error("invalid nonce")]
    Nonce(#[source] InvalidLength),
    #[error("invalid base fee per gas")]
    BaseFeePerGas(#[source] InvalidLength),
}

impl From<L2Header> for protos::union::ibc::lightclients::arbitrum::v1::L2Header {
    fn from(value: L2Header) -> Self {
        Self {
            parent_hash: value.parent_hash.into(),
            sha3_uncles: value.sha3_uncles.into(),
            miner: value.miner.into(),
            state_root: value.state_root.into(),
            transactions_root: value.transactions_root.into(),
            receipts_root: value.receipts_root.into(),
            logs_bloom: (*value.logs_bloom).into(),
            difficulty: value.difficulty.to_be_bytes().to_vec(),
            number: value.number.to_be_bytes().to_vec(),
            gas_limit: value.gas_limit,
            gas_used: value.gas_used,
            timestamp: value.timestamp,
            extra_data: value.extra_data.into(),
            mix_hash: value.mix_hash.into(),
            nonce: value.nonce.into(),
            base_fee_per_gas: value.base_fee_per_gas.to_be_bytes().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ethers::utils::keccak256;
    use hex_literal::hex;
    use rlp::Encodable;

    use super::*;

    #[test]
    fn rlp() {
        // "hash": "0xa548151261174cf854534934ca88e68220e328be563c01915fc11c740a543489",

        let header = L2Header {
            difficulty: U256::try_from_be_bytes(&hex!("01")).unwrap(),
            extra_data: H256(hex!(
                "327fc6b6bcdc7febddc41453d9f5c3703942ec221da53078a91e0b2dbfc02756"
            )),
            gas_limit: 0x0004_0000_0000_0000,
            gas_used: 0x703bc,
            logs_bloom: Box::new(
                hex!(
                    "0400000080000002000002000020000000000000002001000400001000000000 \
                      0000002002000000000000001000000000080001000000080000000000100000 \
                      0000000000000000000000080000000000000000000000000000000001000040 \
                      0000000000000040000000000000000000000000800100040000085000004000 \
                      0800000000000000000000000000000000014000000000000000000000000000 \
                      0000010000001000100002000000000000000020000000000000000004000000 \
                      0002200200002000000000800000000000000000000000000000000000000008 \
                      0000000000000010000000000800200000000001000000000000000010010000"
                )
                .into(),
            ),
            miner: hex!("a4b000000000000000000073657175656e636572").into(),
            mix_hash: H256(hex!(
                "000000000001cbb600000000012de36600000000000000140000000000000000"
            )),
            nonce: hex!("000000000016eb6d").into(),
            number: U256::try_from_be_bytes(&hex!("0c590339")).unwrap(),
            parent_hash: H256(hex!(
                "9ef9a044f15f12bcefd25572fd7600ae4dcc9a90fab9ad98f78abfb221d5731b"
            )),
            receipts_root: H256(hex!(
                "e3fcff2e9ddc6b6a38889ad0997b566a6ba2574ae85aebba4205da14659c175d"
            )),
            sha3_uncles: H256(hex!(
                "1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
            )),
            state_root: H256(hex!(
                "82467a71088bdab7e89d8fe077710172df602d417a77fc813235bb0ca2d3a6c5"
            )),
            timestamp: 0x6633_eab2,
            transactions_root: H256(hex!(
                "9361c0130edfe07e3943d06310c69d5d680d77d571724cd1de0d52f399966107"
            )),
            base_fee_per_gas: U256::try_from_be_bytes(&hex!("989680")).unwrap(),
        };

        assert_eq!(
            H256(keccak256(header.rlp_bytes())),
            H256(hex!(
                "a548151261174cf854534934ca88e68220e328be563c01915fc11c740a543489"
            ))
        );
    }
}
