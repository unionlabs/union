// @generated
pub mod tendermint {
    #[cfg(feature = "tendermint-abci")]
    // @@protoc_insertion_point(attribute:tendermint.abci)
    pub mod abci {
        include!("tendermint.abci.rs");
        // @@protoc_insertion_point(tendermint.abci)
    }
    #[cfg(feature = "tendermint-blocksync")]
    // @@protoc_insertion_point(attribute:tendermint.blocksync)
    pub mod blocksync {
        include!("tendermint.blocksync.rs");
        // @@protoc_insertion_point(tendermint.blocksync)
    }
    #[cfg(feature = "tendermint-consensus")]
    // @@protoc_insertion_point(attribute:tendermint.consensus)
    pub mod consensus {
        include!("tendermint.consensus.rs");
        // @@protoc_insertion_point(tendermint.consensus)
    }
    #[cfg(feature = "tendermint-crypto")]
    // @@protoc_insertion_point(attribute:tendermint.crypto)
    pub mod crypto {
        include!("tendermint.crypto.rs");
        // @@protoc_insertion_point(tendermint.crypto)
    }
    pub mod libs {
        #[cfg(feature = "tendermint-libs-bits")]
        // @@protoc_insertion_point(attribute:tendermint.libs.bits)
        pub mod bits {
            include!("tendermint.libs.bits.rs");
            // @@protoc_insertion_point(tendermint.libs.bits)
        }
    }
    #[cfg(feature = "tendermint-mempool")]
    // @@protoc_insertion_point(attribute:tendermint.mempool)
    pub mod mempool {
        include!("tendermint.mempool.rs");
        // @@protoc_insertion_point(tendermint.mempool)
    }
    #[cfg(feature = "tendermint-p2p")]
    // @@protoc_insertion_point(attribute:tendermint.p2p)
    pub mod p2p {
        include!("tendermint.p2p.rs");
        // @@protoc_insertion_point(tendermint.p2p)
    }
    #[cfg(feature = "tendermint-privval")]
    // @@protoc_insertion_point(attribute:tendermint.privval)
    pub mod privval {
        include!("tendermint.privval.rs");
        // @@protoc_insertion_point(tendermint.privval)
    }
    pub mod rpc {
        #[cfg(feature = "tendermint-rpc-grpc")]
        // @@protoc_insertion_point(attribute:tendermint.rpc.grpc)
        pub mod grpc {
            include!("tendermint.rpc.grpc.rs");
            // @@protoc_insertion_point(tendermint.rpc.grpc)
        }
    }
    #[cfg(feature = "tendermint-state")]
    // @@protoc_insertion_point(attribute:tendermint.state)
    pub mod state {
        include!("tendermint.state.rs");
        // @@protoc_insertion_point(tendermint.state)
    }
    #[cfg(feature = "tendermint-statesync")]
    // @@protoc_insertion_point(attribute:tendermint.statesync)
    pub mod statesync {
        include!("tendermint.statesync.rs");
        // @@protoc_insertion_point(tendermint.statesync)
    }
    #[cfg(feature = "tendermint-store")]
    // @@protoc_insertion_point(attribute:tendermint.store)
    pub mod store {
        include!("tendermint.store.rs");
        // @@protoc_insertion_point(tendermint.store)
    }
    #[cfg(feature = "tendermint-types")]
    // @@protoc_insertion_point(attribute:tendermint.types)
    pub mod types {
        include!("tendermint.types.rs");
        // @@protoc_insertion_point(tendermint.types)
    }
    #[cfg(feature = "tendermint-version")]
    // @@protoc_insertion_point(attribute:tendermint.version)
    pub mod version {
        include!("tendermint.version.rs");
        // @@protoc_insertion_point(tendermint.version)
    }
}
