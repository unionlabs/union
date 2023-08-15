pub mod block;
pub mod block_id;
pub mod block_id_flag;
pub mod commit;
pub mod commit_sig;
pub mod header;
pub mod part_set_header;
pub mod signed_header;
pub mod simple_validator;

pub mod canonical_vote {
    use serde::{Deserialize, Serialize};

    use crate::tendermint::types::canonical_block_id::CanonicalBlockId;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct CanonicalVote {
        /// type alias for byte
        pub ty: i32,
        /// canonicalization requires fixed size encoding here
        pub height: u64,
        /// canonicalization requires fixed size encoding here
        pub round: i64,
        pub block_id: CanonicalBlockId,
        pub chain_id: String,
    }
}

pub mod canonical_block_id {
    use serde::{Deserialize, Serialize};

    use crate::{
        ethereum::H256, tendermint::types::canonical_block_header::CanonicalPartSetHeader,
    };

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct CanonicalBlockId {
        pub hash: H256,
        pub part_set_header: CanonicalPartSetHeader,
    }
}

pub mod canonical_block_header {
    use serde::{Deserialize, Serialize};

    use crate::ethereum::H256;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct CanonicalPartSetHeader {
        pub total: u32,
        pub hash: H256,
    }
}

pub mod signed_msg_type {
    use crate::macros::wrapper_enum;

    wrapper_enum! {
        #[proto(protos::tendermint::types::SignedMsgType)]
        pub enum SignedMsgType {
            Unknown = 0,
            Prevote = 1,
            Precommit = 2,
            Proposal = 32,
        }
    }
}
