// @generated
pub mod capability {
    #[cfg(feature = "capability-v1")]
    // @@protoc_insertion_point(attribute:capability.v1)
    pub mod v1 {
        include!("capability.v1.rs");
        // @@protoc_insertion_point(capability.v1)
    }
}
pub mod ibc {
    pub mod applications {
        pub mod fee {
            #[cfg(feature = "ibc-applications-fee-v1")]
            // @@protoc_insertion_point(attribute:ibc.applications.fee.v1)
            pub mod v1 {
                include!("ibc.applications.fee.v1.rs");
                // @@protoc_insertion_point(ibc.applications.fee.v1)
            }
        }
        pub mod interchain_accounts {
            pub mod controller {
                #[cfg(feature = "ibc-applications-interchain_accounts-controller-v1")]
                // @@protoc_insertion_point(attribute:ibc.applications.interchain_accounts.controller.v1)
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.controller.v1.rs");
                    // @@protoc_insertion_point(ibc.applications.interchain_accounts.controller.v1)
                }
            }
            pub mod genesis {
                #[cfg(feature = "ibc-applications-interchain_accounts-genesis-v1")]
                // @@protoc_insertion_point(attribute:ibc.applications.interchain_accounts.genesis.v1)
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.genesis.v1.rs");
                    // @@protoc_insertion_point(ibc.applications.interchain_accounts.genesis.v1)
                }
            }
            pub mod host {
                #[cfg(feature = "ibc-applications-interchain_accounts-host-v1")]
                // @@protoc_insertion_point(attribute:ibc.applications.interchain_accounts.host.v1)
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.host.v1.rs");
                    // @@protoc_insertion_point(ibc.applications.interchain_accounts.host.v1)
                }
            }
            #[cfg(feature = "ibc-applications-interchain_accounts-v1")]
            // @@protoc_insertion_point(attribute:ibc.applications.interchain_accounts.v1)
            pub mod v1 {
                include!("ibc.applications.interchain_accounts.v1.rs");
                // @@protoc_insertion_point(ibc.applications.interchain_accounts.v1)
            }
        }
        pub mod transfer {
            #[cfg(feature = "ibc-applications-transfer-v1")]
            // @@protoc_insertion_point(attribute:ibc.applications.transfer.v1)
            pub mod v1 {
                include!("ibc.applications.transfer.v1.rs");
                // @@protoc_insertion_point(ibc.applications.transfer.v1)
            }
            #[cfg(feature = "ibc-applications-transfer-v2")]
            // @@protoc_insertion_point(attribute:ibc.applications.transfer.v2)
            pub mod v2 {
                include!("ibc.applications.transfer.v2.rs");
                // @@protoc_insertion_point(ibc.applications.transfer.v2)
            }
        }
    }
    pub mod core {
        pub mod channel {
            #[cfg(feature = "ibc-core-channel-v1")]
            // @@protoc_insertion_point(attribute:ibc.core.channel.v1)
            pub mod v1 {
                include!("ibc.core.channel.v1.rs");
                // @@protoc_insertion_point(ibc.core.channel.v1)
            }
        }
        pub mod client {
            #[cfg(feature = "ibc-core-client-v1")]
            // @@protoc_insertion_point(attribute:ibc.core.client.v1)
            pub mod v1 {
                include!("ibc.core.client.v1.rs");
                // @@protoc_insertion_point(ibc.core.client.v1)
            }
        }
        pub mod commitment {
            #[cfg(feature = "ibc-core-commitment-v1")]
            // @@protoc_insertion_point(attribute:ibc.core.commitment.v1)
            pub mod v1 {
                include!("ibc.core.commitment.v1.rs");
                // @@protoc_insertion_point(ibc.core.commitment.v1)
            }
        }
        pub mod connection {
            #[cfg(feature = "ibc-core-connection-v1")]
            // @@protoc_insertion_point(attribute:ibc.core.connection.v1)
            pub mod v1 {
                include!("ibc.core.connection.v1.rs");
                // @@protoc_insertion_point(ibc.core.connection.v1)
            }
        }
        pub mod types {
            #[cfg(feature = "ibc-core-types-v1")]
            // @@protoc_insertion_point(attribute:ibc.core.types.v1)
            pub mod v1 {
                include!("ibc.core.types.v1.rs");
                // @@protoc_insertion_point(ibc.core.types.v1)
            }
        }
    }
    pub mod lightclients {
        pub mod localhost {
            #[cfg(feature = "ibc-lightclients-localhost-v2")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.localhost.v2)
            pub mod v2 {
                include!("ibc.lightclients.localhost.v2.rs");
                // @@protoc_insertion_point(ibc.lightclients.localhost.v2)
            }
        }
        pub mod solomachine {
            #[cfg(feature = "ibc-lightclients-solomachine-v2")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.solomachine.v2)
            pub mod v2 {
                include!("ibc.lightclients.solomachine.v2.rs");
                // @@protoc_insertion_point(ibc.lightclients.solomachine.v2)
            }
            #[cfg(feature = "ibc-lightclients-solomachine-v3")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.solomachine.v3)
            pub mod v3 {
                include!("ibc.lightclients.solomachine.v3.rs");
                // @@protoc_insertion_point(ibc.lightclients.solomachine.v3)
            }
        }
        pub mod tendermint {
            #[cfg(feature = "ibc-lightclients-tendermint-v1")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.tendermint.v1)
            pub mod v1 {
                include!("ibc.lightclients.tendermint.v1.rs");
                // @@protoc_insertion_point(ibc.lightclients.tendermint.v1)
            }
        }
        pub mod wasm {
            #[cfg(feature = "ibc-lightclients-wasm-v1")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.wasm.v1)
            pub mod v1 {
                include!("ibc.lightclients.wasm.v1.rs");
                // @@protoc_insertion_point(ibc.lightclients.wasm.v1)
            }
        }
    }
}
