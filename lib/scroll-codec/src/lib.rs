#![feature(iter_array_chunks)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! Scroll types, as specified in <https://github.com/scroll-tech/scroll/tree/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/libraries/codec>, with the commit from [this announcement](https://scroll.io/blog/blobs-are-here-scrolls-bernoulli-upgrade).

use serde::{Deserialize, Serialize};
use unionlabs::hash::H256;

use crate::batch_header::{BatchHeaderV3, BatchHeaderV3DecodeError};

pub mod batch_header;

alloy::sol! {
    // #[allow(clippy::pub_underscore_fields)]

    /// See <https://github.com/scroll-tech/scroll-contracts/blob/7bb751f9cf1b5fdde95297049e3407ce23d56ac6/src/mocks/ScrollChainMockFinalize.sol#L59>
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    function finalizeBundle(
        bytes calldata _batchHeader,
        bytes32 _postStateRoot,
        bytes32 _withdrawRoot
    );

    /// See <https://github.com/scroll-tech/scroll-contracts/blob/7bb751f9cf1b5fdde95297049e3407ce23d56ac6/src/L1/rollup/ScrollChain.sol#L537>
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    function finalizeBundleWithProof(
        bytes calldata _batchHeader,
        bytes32 _postStateRoot,
        bytes32 _withdrawRoot,
        bytes calldata _aggrProof
    );
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum HashBatchError {
    #[error("error decoding parent batch header")]
    BatchHeaderDecode(#[from] BatchHeaderV3DecodeError),
}

/// Reconstruct a batch hash given a batch header.
/// Partial function only valid for batch header V3+.
///
/// # Errors
///
/// Fails if the batch header can't be decoded.
pub fn hash_batch(batch_header: Vec<u8>) -> Result<H256, HashBatchError> {
    let batch_header = BatchHeaderV3::decode(batch_header)?;
    Ok(batch_header.compute_batch_hash())
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::hash::H256;

    use crate::hash_batch;

    #[test]
    fn test_hash_batch() {
        let headers = [
            (
                hex!("03000000000001245100000000000000010000000000101a4fa380f90c47b1ba7fc126c791ed7b1d7de26fb793b40eb513aca94bdf7f1195960103f88ca94493d70d700352092ab0a60708bbe17245c5848873550550e78529ff6170751da66131c1181c8f6b5469b04fcdd1c83d61b698db91aa94c9b8c2530000000066c360443460f04f6511752e6f2635b61af4f360021133c2c1b53e2967cf2f689258751d687a320d464eac03e4e3e4d874f1aaf4c005bc77ccbd2906cc77e51c7a197001"),
                hex!("E8300E026CADB35C0CEAF029226CDDD1F1B52D11A9D8861B899F890B47FCE02E")
            ),
            (
                hex!("03000000000001244f00000000000000000000000000101a4efe5be02800ce0b4f88dd9c2c7fa445c5a5dd45ded8a1fe59ecb2a0de47bd0f6601b6927fcaeaff9065fbe2b030f63176f30fd5ca5aced3d3a54837bc3f788474f9cfeb1078ec8ed1d1051aa5b595dc1248c43fe0bf67fcfc1c4400fdaf0541f20000000066c33a4939995dc16d981ea89c9706e70b034383627a58b3f9ff702d5bd3d7dcb3db269d5e7a0d19299ccba865d649fdfe161df417ef79266723540d751b0798bc055d2b"),
                hex!("9665FA18D352744EAB828AC026B3B0528EB1B6DD6C6A264797D93FE8AB565ECA")
            ),
            (
                hex!("03000000000001244e00000000000000030000000000101a4ed47a65d6c3a55a82be60c1ffd343c5079d70911621414dfbcbf2cf664d0669d80133054baa7af3f37c5787801d0b0200551b5ab5636ace22baeb8407d038fdd70c2788800c1a74dfa4f13b1cb438d95a9d267059dfaea86a62ec1195a5d323360000000066c327074fb657217fc9838d72697a108409adef6cc4c0bd2371b080e5e4826b260dca712c851545531b62438f62dbe391ed5bcf50cf6632f731a1845128eb44692ca94f"),
                hex!("F9CFEB1078EC8ED1D1051AA5B595DC1248C43FE0BF67FCFC1C4400FDAF0541F2")
            ),
            (
                hex!("03000000000001244d00000000000000020000000000101a4b5f0f16489871e4aadb23363c11f891319670743f4bf8e369698d406eb9f2abed01c49e662aaafb6bd22b507b04280bcb9c68616ecdf4fd43e61e31d1f1ef7b2b7c2be00605b3991d7128eb7c75bf0987eb1bcd8eadd23234e18695c9177b3c670000000066c3141569855d117753668dc0801ff670e8876e04eea0f777eba44028f2ccc80c2eb1430d5df02aab9db3e7aa62028f4edc5db2d6b5215316492f9b6a4d4bb21b554737"),
                hex!("0C2788800C1A74DFA4F13B1CB438D95A9D267059DFAEA86A62EC1195A5D32336")
            ),
            (
                hex!("040000000000012bd200000000000000020000000000101c9373d135e03db7cdf1836f5c1dca8ee1904851300464d8be7adea8694643e7776d0111b6c7fcf7b6c59e2cf2cb7dd8b637d847d53c0dbcf207761f7d214503c96e592dfea5df424583bf3d39c9641bf3dbb4c30a425f22aa5e843fc9f1234e25fc0000000066d5719e53e21bd0459892f5687970ba5baa4dcdd9a70e876d8e7fb33a86db1a925d8d0244c936e811cba2e5726aad59c7f2e94b11db9f619ea7ba672b15d25fe15157a8"),
                hex!("41261fb29d31ed23060103ee8a84129956adec97c33685414e6afb1ad9802cb4")
            )
        ];
        for (header, expected_hash) in headers {
            assert_eq!(
                hash_batch(header.to_vec()).unwrap(),
                <H256>::new(expected_hash)
            );
        }
    }
}
