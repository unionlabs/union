use core::cmp::min;

pub mod block;
pub mod block_id;
pub mod block_meta;
pub mod commit;
pub mod data;
pub mod event;
pub mod event_attribute;
pub mod header;
pub mod info_response;
pub mod node_info;
pub mod node_info_other;
pub mod part_set_header;
pub mod proof;
pub mod proof_op;
pub mod public_key;
pub mod query_response;
pub mod response_base;
pub mod signed_header;
pub mod signed_msg_type;
pub mod validator;
pub mod validator_set;
pub mod version_info;
pub mod vote;

pub use block::Block;
pub use block_id::BlockId;
pub use block_meta::BlockMeta;
pub use commit::Commit;
pub use data::Data;
pub use event::Event;
pub use event_attribute::EventAttribute;
pub use header::Header;
pub use info_response::InfoResponse;
pub use node_info::NodeInfo;
pub use node_info_other::NodeInfoOther;
pub use part_set_header::PartSetHeader;
pub use proof::Proof;
pub use proof_op::ProofOp;
pub use public_key::PublicKey;
pub use query_response::QueryResponse;
pub use response_base::ResponseBase;
pub use signed_header::SignedHeader;
pub use signed_msg_type::SignedMsgType;
use unionlabs::primitives::{Bytes, FixedBytes, encoding::HexUnprefixed};
pub use validator::Validator;
pub use validator_set::ValidatorSet;
pub use version_info::VersionInfo;
pub use vote::Vote;

pub trait Amino {
    fn marshal_sized(&self) -> Bytes;
}

/// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/fingerprint.go#L6>
fn fingerprint(slice: impl AsRef<[u8]>) -> FixedBytes<6, HexUnprefixed> {
    let mut fingerprint = FixedBytes::default();
    let end = min(6, slice.as_ref().len());
    fingerprint[0..end].copy_from_slice(&slice.as_ref()[0..end]);
    fingerprint
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn fingerprint_works() {
        for (i, o) in [
            (hex!("").as_slice(), hex!("000000000000").as_slice()),
            (hex!("0102").as_slice(), hex!("010200000000").as_slice()),
            (
                hex!("010203010203").as_slice(),
                hex!("010203010203").as_slice(),
            ),
            (
                hex!("010203010203AA").as_slice(),
                hex!("010203010203").as_slice(),
            ),
            (
                hex!("010203010203AAAAAA").as_slice(),
                hex!("010203010203").as_slice(),
            ),
        ] {
            assert_eq!(fingerprint(i).get(), o);
        }
    }
}
