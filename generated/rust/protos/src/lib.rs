#![allow(clippy::all, rustdoc::all)]
// @generated
pub mod capability {
    #[cfg(feature = "capability+v1")]
    // @@protoc_insertion_point(attribute:capability.v1)
    pub mod v1 {
        include!("capability.v1.rs");
        // @@protoc_insertion_point(capability.v1)
    }
}
pub mod cosmos {
    pub mod app {
        pub mod runtime {
            #[cfg(feature = "cosmos+app+runtime+v1alpha1")]
            // @@protoc_insertion_point(attribute:cosmos.app.runtime.v1alpha1)
            pub mod v1alpha1 {
                include!("cosmos.app.runtime.v1alpha1.rs");
                // @@protoc_insertion_point(cosmos.app.runtime.v1alpha1)
            }
        }
        #[cfg(feature = "cosmos+app+v1alpha1")]
        // @@protoc_insertion_point(attribute:cosmos.app.v1alpha1)
        pub mod v1alpha1 {
            include!("cosmos.app.v1alpha1.rs");
            // @@protoc_insertion_point(cosmos.app.v1alpha1)
        }
    }
    pub mod auth {
        pub mod module {
            #[cfg(feature = "cosmos+auth+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.auth.module.v1)
            pub mod v1 {
                include!("cosmos.auth.module.v1.rs");
                // @@protoc_insertion_point(cosmos.auth.module.v1)
            }
        }
        #[cfg(feature = "cosmos+auth+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.auth.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.auth.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.auth.v1beta1)
        }
    }
    pub mod authz {
        pub mod module {
            #[cfg(feature = "cosmos+authz+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.authz.module.v1)
            pub mod v1 {
                include!("cosmos.authz.module.v1.rs");
                // @@protoc_insertion_point(cosmos.authz.module.v1)
            }
        }
        #[cfg(feature = "cosmos+authz+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.authz.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.authz.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.authz.v1beta1)
        }
    }
    pub mod autocli {
        #[cfg(feature = "cosmos+autocli+v1")]
        // @@protoc_insertion_point(attribute:cosmos.autocli.v1)
        pub mod v1 {
            include!("cosmos.autocli.v1.rs");
            // @@protoc_insertion_point(cosmos.autocli.v1)
        }
    }
    pub mod bank {
        pub mod module {
            #[cfg(feature = "cosmos+bank+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.bank.module.v1)
            pub mod v1 {
                include!("cosmos.bank.module.v1.rs");
                // @@protoc_insertion_point(cosmos.bank.module.v1)
            }
        }
        #[cfg(feature = "cosmos+bank+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.bank.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.bank.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.bank.v1beta1)
        }
    }
    pub mod base {
        pub mod abci {
            #[cfg(feature = "cosmos+base+abci+v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.abci.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.abci.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.abci.v1beta1)
            }
        }
        pub mod node {
            #[cfg(feature = "cosmos+base+node+v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.node.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.node.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.node.v1beta1)
            }
        }
        pub mod query {
            #[cfg(feature = "cosmos+base+query+v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.query.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.query.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.query.v1beta1)
            }
        }
        pub mod reflection {
            #[cfg(feature = "cosmos+base+reflection+v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.reflection.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.reflection.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.reflection.v1beta1)
            }
            #[cfg(feature = "cosmos+base+reflection+v2alpha1")]
            // @@protoc_insertion_point(attribute:cosmos.base.reflection.v2alpha1)
            pub mod v2alpha1 {
                include!("cosmos.base.reflection.v2alpha1.rs");
                // @@protoc_insertion_point(cosmos.base.reflection.v2alpha1)
            }
        }
        pub mod tendermint {
            #[cfg(feature = "cosmos+base+tendermint+v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.tendermint.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.tendermint.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.tendermint.v1beta1)
            }
        }
        #[cfg(feature = "cosmos+base+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.base.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.base.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.base.v1beta1)
        }
    }
    pub mod circuit {
        pub mod module {
            #[cfg(feature = "cosmos+circuit+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.circuit.module.v1)
            pub mod v1 {
                include!("cosmos.circuit.module.v1.rs");
                // @@protoc_insertion_point(cosmos.circuit.module.v1)
            }
        }
        #[cfg(feature = "cosmos+circuit+v1")]
        // @@protoc_insertion_point(attribute:cosmos.circuit.v1)
        pub mod v1 {
            include!("cosmos.circuit.v1.rs");
            // @@protoc_insertion_point(cosmos.circuit.v1)
        }
    }
    pub mod consensus {
        pub mod module {
            #[cfg(feature = "cosmos+consensus+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.consensus.module.v1)
            pub mod v1 {
                include!("cosmos.consensus.module.v1.rs");
                // @@protoc_insertion_point(cosmos.consensus.module.v1)
            }
        }
        #[cfg(feature = "cosmos+consensus+v1")]
        // @@protoc_insertion_point(attribute:cosmos.consensus.v1)
        pub mod v1 {
            include!("cosmos.consensus.v1.rs");
            // @@protoc_insertion_point(cosmos.consensus.v1)
        }
    }
    pub mod crisis {
        pub mod module {
            #[cfg(feature = "cosmos+crisis+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.crisis.module.v1)
            pub mod v1 {
                include!("cosmos.crisis.module.v1.rs");
                // @@protoc_insertion_point(cosmos.crisis.module.v1)
            }
        }
        #[cfg(feature = "cosmos+crisis+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.crisis.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.crisis.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.crisis.v1beta1)
        }
    }
    pub mod crypto {
        #[cfg(feature = "cosmos+crypto+bn254")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.bn254)
        pub mod bn254 {
            include!("cosmos.crypto.bn254.rs");
            // @@protoc_insertion_point(cosmos.crypto.bn254)
        }
        #[cfg(feature = "cosmos+crypto+ed25519")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.ed25519)
        pub mod ed25519 {
            include!("cosmos.crypto.ed25519.rs");
            // @@protoc_insertion_point(cosmos.crypto.ed25519)
        }
        pub mod hd {
            #[cfg(feature = "cosmos+crypto+hd+v1")]
            // @@protoc_insertion_point(attribute:cosmos.crypto.hd.v1)
            pub mod v1 {
                include!("cosmos.crypto.hd.v1.rs");
                // @@protoc_insertion_point(cosmos.crypto.hd.v1)
            }
        }
        pub mod keyring {
            #[cfg(feature = "cosmos+crypto+keyring+v1")]
            // @@protoc_insertion_point(attribute:cosmos.crypto.keyring.v1)
            pub mod v1 {
                include!("cosmos.crypto.keyring.v1.rs");
                // @@protoc_insertion_point(cosmos.crypto.keyring.v1)
            }
        }
        #[cfg(feature = "cosmos+crypto+multisig")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.multisig)
        pub mod multisig {
            include!("cosmos.crypto.multisig.rs");
            // @@protoc_insertion_point(cosmos.crypto.multisig)
            #[cfg(feature = "cosmos+crypto+multisig+v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.crypto.multisig.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.crypto.multisig.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.crypto.multisig.v1beta1)
            }
        }
        #[cfg(feature = "cosmos+crypto+secp256k1")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.secp256k1)
        pub mod secp256k1 {
            include!("cosmos.crypto.secp256k1.rs");
            // @@protoc_insertion_point(cosmos.crypto.secp256k1)
        }
        #[cfg(feature = "cosmos+crypto+secp256r1")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.secp256r1)
        pub mod secp256r1 {
            include!("cosmos.crypto.secp256r1.rs");
            // @@protoc_insertion_point(cosmos.crypto.secp256r1)
        }
    }
    pub mod distribution {
        pub mod module {
            #[cfg(feature = "cosmos+distribution+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.distribution.module.v1)
            pub mod v1 {
                include!("cosmos.distribution.module.v1.rs");
                // @@protoc_insertion_point(cosmos.distribution.module.v1)
            }
        }
        #[cfg(feature = "cosmos+distribution+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.distribution.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.distribution.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.distribution.v1beta1)
        }
    }
    pub mod evidence {
        pub mod module {
            #[cfg(feature = "cosmos+evidence+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.evidence.module.v1)
            pub mod v1 {
                include!("cosmos.evidence.module.v1.rs");
                // @@protoc_insertion_point(cosmos.evidence.module.v1)
            }
        }
        #[cfg(feature = "cosmos+evidence+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.evidence.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.evidence.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.evidence.v1beta1)
        }
    }
    pub mod feegrant {
        pub mod module {
            #[cfg(feature = "cosmos+feegrant+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.feegrant.module.v1)
            pub mod v1 {
                include!("cosmos.feegrant.module.v1.rs");
                // @@protoc_insertion_point(cosmos.feegrant.module.v1)
            }
        }
        #[cfg(feature = "cosmos+feegrant+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.feegrant.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.feegrant.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.feegrant.v1beta1)
        }
    }
    pub mod genutil {
        pub mod module {
            #[cfg(feature = "cosmos+genutil+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.genutil.module.v1)
            pub mod v1 {
                include!("cosmos.genutil.module.v1.rs");
                // @@protoc_insertion_point(cosmos.genutil.module.v1)
            }
        }
        #[cfg(feature = "cosmos+genutil+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.genutil.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.genutil.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.genutil.v1beta1)
        }
    }
    pub mod gov {
        pub mod module {
            #[cfg(feature = "cosmos+gov+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.gov.module.v1)
            pub mod v1 {
                include!("cosmos.gov.module.v1.rs");
                // @@protoc_insertion_point(cosmos.gov.module.v1)
            }
        }
        #[cfg(feature = "cosmos+gov+v1")]
        // @@protoc_insertion_point(attribute:cosmos.gov.v1)
        pub mod v1 {
            include!("cosmos.gov.v1.rs");
            // @@protoc_insertion_point(cosmos.gov.v1)
        }
        #[cfg(feature = "cosmos+gov+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.gov.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.gov.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.gov.v1beta1)
        }
    }
    pub mod group {
        pub mod module {
            #[cfg(feature = "cosmos+group+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.group.module.v1)
            pub mod v1 {
                include!("cosmos.group.module.v1.rs");
                // @@protoc_insertion_point(cosmos.group.module.v1)
            }
        }
        #[cfg(feature = "cosmos+group+v1")]
        // @@protoc_insertion_point(attribute:cosmos.group.v1)
        pub mod v1 {
            include!("cosmos.group.v1.rs");
            // @@protoc_insertion_point(cosmos.group.v1)
        }
    }
    pub mod ics23 {
        #[cfg(feature = "cosmos+ics23+v1")]
        // @@protoc_insertion_point(attribute:cosmos.ics23.v1)
        pub mod v1 {
            include!("cosmos.ics23.v1.rs");
            // @@protoc_insertion_point(cosmos.ics23.v1)
        }
    }
    pub mod mint {
        pub mod module {
            #[cfg(feature = "cosmos+mint+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.mint.module.v1)
            pub mod v1 {
                include!("cosmos.mint.module.v1.rs");
                // @@protoc_insertion_point(cosmos.mint.module.v1)
            }
        }
        #[cfg(feature = "cosmos+mint+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.mint.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.mint.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.mint.v1beta1)
        }
    }
    pub mod nft {
        pub mod module {
            #[cfg(feature = "cosmos+nft+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.nft.module.v1)
            pub mod v1 {
                include!("cosmos.nft.module.v1.rs");
                // @@protoc_insertion_point(cosmos.nft.module.v1)
            }
        }
        #[cfg(feature = "cosmos+nft+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.nft.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.nft.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.nft.v1beta1)
        }
    }
    pub mod orm {
        pub mod module {
            #[cfg(feature = "cosmos+orm+module+v1alpha1")]
            // @@protoc_insertion_point(attribute:cosmos.orm.module.v1alpha1)
            pub mod v1alpha1 {
                include!("cosmos.orm.module.v1alpha1.rs");
                // @@protoc_insertion_point(cosmos.orm.module.v1alpha1)
            }
        }
        pub mod query {
            #[cfg(feature = "cosmos+orm+query+v1alpha1")]
            // @@protoc_insertion_point(attribute:cosmos.orm.query.v1alpha1)
            pub mod v1alpha1 {
                include!("cosmos.orm.query.v1alpha1.rs");
                // @@protoc_insertion_point(cosmos.orm.query.v1alpha1)
            }
        }
        #[cfg(feature = "cosmos+orm+v1")]
        // @@protoc_insertion_point(attribute:cosmos.orm.v1)
        pub mod v1 {
            include!("cosmos.orm.v1.rs");
            // @@protoc_insertion_point(cosmos.orm.v1)
        }
        #[cfg(feature = "cosmos+orm+v1alpha1")]
        // @@protoc_insertion_point(attribute:cosmos.orm.v1alpha1)
        pub mod v1alpha1 {
            include!("cosmos.orm.v1alpha1.rs");
            // @@protoc_insertion_point(cosmos.orm.v1alpha1)
        }
    }
    pub mod params {
        pub mod module {
            #[cfg(feature = "cosmos+params+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.params.module.v1)
            pub mod v1 {
                include!("cosmos.params.module.v1.rs");
                // @@protoc_insertion_point(cosmos.params.module.v1)
            }
        }
        #[cfg(feature = "cosmos+params+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.params.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.params.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.params.v1beta1)
        }
    }
    pub mod reflection {
        #[cfg(feature = "cosmos+reflection+v1")]
        // @@protoc_insertion_point(attribute:cosmos.reflection.v1)
        pub mod v1 {
            include!("cosmos.reflection.v1.rs");
            // @@protoc_insertion_point(cosmos.reflection.v1)
        }
    }
    pub mod slashing {
        pub mod module {
            #[cfg(feature = "cosmos+slashing+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.slashing.module.v1)
            pub mod v1 {
                include!("cosmos.slashing.module.v1.rs");
                // @@protoc_insertion_point(cosmos.slashing.module.v1)
            }
        }
        #[cfg(feature = "cosmos+slashing+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.slashing.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.slashing.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.slashing.v1beta1)
        }
    }
    pub mod staking {
        pub mod module {
            #[cfg(feature = "cosmos+staking+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.staking.module.v1)
            pub mod v1 {
                include!("cosmos.staking.module.v1.rs");
                // @@protoc_insertion_point(cosmos.staking.module.v1)
            }
        }
        #[cfg(feature = "cosmos+staking+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.staking.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.staking.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.staking.v1beta1)
        }
    }
    pub mod store {
        pub mod internal {
            pub mod kv {
                #[cfg(feature = "cosmos+store+internal+kv+v1beta1")]
                // @@protoc_insertion_point(attribute:cosmos.store.internal.kv.v1beta1)
                pub mod v1beta1 {
                    include!("cosmos.store.internal.kv.v1beta1.rs");
                    // @@protoc_insertion_point(cosmos.store.internal.kv.v1beta1)
                }
            }
        }
        pub mod snapshots {
            #[cfg(feature = "cosmos+store+snapshots+v1")]
            // @@protoc_insertion_point(attribute:cosmos.store.snapshots.v1)
            pub mod v1 {
                include!("cosmos.store.snapshots.v1.rs");
                // @@protoc_insertion_point(cosmos.store.snapshots.v1)
            }
        }
        pub mod streaming {
            #[cfg(feature = "cosmos+store+streaming+abci")]
            // @@protoc_insertion_point(attribute:cosmos.store.streaming.abci)
            pub mod abci {
                include!("cosmos.store.streaming.abci.rs");
                // @@protoc_insertion_point(cosmos.store.streaming.abci)
            }
        }
        #[cfg(feature = "cosmos+store+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.store.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.store.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.store.v1beta1)
        }
    }
    pub mod tx {
        pub mod config {
            #[cfg(feature = "cosmos+tx+config+v1")]
            // @@protoc_insertion_point(attribute:cosmos.tx.config.v1)
            pub mod v1 {
                include!("cosmos.tx.config.v1.rs");
                // @@protoc_insertion_point(cosmos.tx.config.v1)
            }
        }
        pub mod signing {
            #[cfg(feature = "cosmos+tx+signing+v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.tx.signing.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.tx.signing.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.tx.signing.v1beta1)
            }
        }
        #[cfg(feature = "cosmos+tx+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.tx.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.tx.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.tx.v1beta1)
        }
    }
    pub mod upgrade {
        pub mod module {
            #[cfg(feature = "cosmos+upgrade+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.upgrade.module.v1)
            pub mod v1 {
                include!("cosmos.upgrade.module.v1.rs");
                // @@protoc_insertion_point(cosmos.upgrade.module.v1)
            }
        }
        #[cfg(feature = "cosmos+upgrade+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.upgrade.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.upgrade.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.upgrade.v1beta1)
        }
    }
    pub mod vesting {
        pub mod module {
            #[cfg(feature = "cosmos+vesting+module+v1")]
            // @@protoc_insertion_point(attribute:cosmos.vesting.module.v1)
            pub mod v1 {
                include!("cosmos.vesting.module.v1.rs");
                // @@protoc_insertion_point(cosmos.vesting.module.v1)
            }
        }
        #[cfg(feature = "cosmos+vesting+v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.vesting.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.vesting.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.vesting.v1beta1)
        }
    }
}
pub mod cosmwasm {
    pub mod wasm {
        #[cfg(feature = "cosmwasm+wasm+v1")]
        // @@protoc_insertion_point(attribute:cosmwasm.wasm.v1)
        pub mod v1 {
            include!("cosmwasm.wasm.v1.rs");
            // @@protoc_insertion_point(cosmwasm.wasm.v1)
        }
    }
}
pub mod google {
    #[cfg(feature = "google+protobuf")]
    // @@protoc_insertion_point(attribute:google.protobuf)
    pub mod protobuf {
        include!("google.protobuf.rs");
        // @@protoc_insertion_point(google.protobuf)
        #[cfg(feature = "google+protobuf+compiler")]
        // @@protoc_insertion_point(attribute:google.protobuf.compiler)
        pub mod compiler {
            include!("google.protobuf.compiler.rs");
            // @@protoc_insertion_point(google.protobuf.compiler)
        }
    }
}
pub mod ibc {
    pub mod applications {
        pub mod fee {
            #[cfg(feature = "ibc+applications+fee+v1")]
            // @@protoc_insertion_point(attribute:ibc.applications.fee.v1)
            pub mod v1 {
                include!("ibc.applications.fee.v1.rs");
                // @@protoc_insertion_point(ibc.applications.fee.v1)
            }
        }
        pub mod interchain_accounts {
            pub mod controller {
                #[cfg(feature = "ibc+applications+interchain_accounts+controller+v1")]
                // @@protoc_insertion_point(attribute:ibc.applications.interchain_accounts.controller.v1)
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.controller.v1.rs");
                    // @@protoc_insertion_point(ibc.applications.interchain_accounts.controller.v1)
                }
            }
            pub mod genesis {
                #[cfg(feature = "ibc+applications+interchain_accounts+genesis+v1")]
                // @@protoc_insertion_point(attribute:ibc.applications.interchain_accounts.genesis.v1)
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.genesis.v1.rs");
                    // @@protoc_insertion_point(ibc.applications.interchain_accounts.genesis.v1)
                }
            }
            pub mod host {
                #[cfg(feature = "ibc+applications+interchain_accounts+host+v1")]
                // @@protoc_insertion_point(attribute:ibc.applications.interchain_accounts.host.v1)
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.host.v1.rs");
                    // @@protoc_insertion_point(ibc.applications.interchain_accounts.host.v1)
                }
            }
            #[cfg(feature = "ibc+applications+interchain_accounts+v1")]
            // @@protoc_insertion_point(attribute:ibc.applications.interchain_accounts.v1)
            pub mod v1 {
                include!("ibc.applications.interchain_accounts.v1.rs");
                // @@protoc_insertion_point(ibc.applications.interchain_accounts.v1)
            }
        }
        pub mod transfer {
            #[cfg(feature = "ibc+applications+transfer+v1")]
            // @@protoc_insertion_point(attribute:ibc.applications.transfer.v1)
            pub mod v1 {
                include!("ibc.applications.transfer.v1.rs");
                // @@protoc_insertion_point(ibc.applications.transfer.v1)
            }
            #[cfg(feature = "ibc+applications+transfer+v2")]
            // @@protoc_insertion_point(attribute:ibc.applications.transfer.v2)
            pub mod v2 {
                include!("ibc.applications.transfer.v2.rs");
                // @@protoc_insertion_point(ibc.applications.transfer.v2)
            }
        }
    }
    pub mod core {
        pub mod channel {
            #[cfg(feature = "ibc+core+channel+v1")]
            // @@protoc_insertion_point(attribute:ibc.core.channel.v1)
            pub mod v1 {
                include!("ibc.core.channel.v1.rs");
                // @@protoc_insertion_point(ibc.core.channel.v1)
            }
        }
        pub mod client {
            #[cfg(feature = "ibc+core+client+v1")]
            // @@protoc_insertion_point(attribute:ibc.core.client.v1)
            pub mod v1 {
                include!("ibc.core.client.v1.rs");
                // @@protoc_insertion_point(ibc.core.client.v1)
            }
        }
        pub mod commitment {
            #[cfg(feature = "ibc+core+commitment+v1")]
            // @@protoc_insertion_point(attribute:ibc.core.commitment.v1)
            pub mod v1 {
                include!("ibc.core.commitment.v1.rs");
                // @@protoc_insertion_point(ibc.core.commitment.v1)
            }
        }
        pub mod connection {
            #[cfg(feature = "ibc+core+connection+v1")]
            // @@protoc_insertion_point(attribute:ibc.core.connection.v1)
            pub mod v1 {
                include!("ibc.core.connection.v1.rs");
                // @@protoc_insertion_point(ibc.core.connection.v1)
            }
        }
        pub mod types {
            #[cfg(feature = "ibc+core+types+v1")]
            // @@protoc_insertion_point(attribute:ibc.core.types.v1)
            pub mod v1 {
                include!("ibc.core.types.v1.rs");
                // @@protoc_insertion_point(ibc.core.types.v1)
            }
        }
    }
    pub mod lightclients {
        pub mod localhost {
            #[cfg(feature = "ibc+lightclients+localhost+v2")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.localhost.v2)
            pub mod v2 {
                include!("ibc.lightclients.localhost.v2.rs");
                // @@protoc_insertion_point(ibc.lightclients.localhost.v2)
            }
        }
        pub mod solomachine {
            #[cfg(feature = "ibc+lightclients+solomachine+v2")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.solomachine.v2)
            pub mod v2 {
                include!("ibc.lightclients.solomachine.v2.rs");
                // @@protoc_insertion_point(ibc.lightclients.solomachine.v2)
            }
            #[cfg(feature = "ibc+lightclients+solomachine+v3")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.solomachine.v3)
            pub mod v3 {
                include!("ibc.lightclients.solomachine.v3.rs");
                // @@protoc_insertion_point(ibc.lightclients.solomachine.v3)
            }
        }
        pub mod tendermint {
            #[cfg(feature = "ibc+lightclients+tendermint+v1")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.tendermint.v1)
            pub mod v1 {
                include!("ibc.lightclients.tendermint.v1.rs");
                // @@protoc_insertion_point(ibc.lightclients.tendermint.v1)
            }
        }
        pub mod wasm {
            #[cfg(feature = "ibc+lightclients+wasm+v1")]
            // @@protoc_insertion_point(attribute:ibc.lightclients.wasm.v1)
            pub mod v1 {
                include!("ibc.lightclients.wasm.v1.rs");
                // @@protoc_insertion_point(ibc.lightclients.wasm.v1)
            }
        }
    }
}
pub mod tendermint {
    #[cfg(feature = "tendermint+abci")]
    // @@protoc_insertion_point(attribute:tendermint.abci)
    pub mod abci {
        include!("tendermint.abci.rs");
        // @@protoc_insertion_point(tendermint.abci)
    }
    #[cfg(feature = "tendermint+blocksync")]
    // @@protoc_insertion_point(attribute:tendermint.blocksync)
    pub mod blocksync {
        include!("tendermint.blocksync.rs");
        // @@protoc_insertion_point(tendermint.blocksync)
    }
    #[cfg(feature = "tendermint+consensus")]
    // @@protoc_insertion_point(attribute:tendermint.consensus)
    pub mod consensus {
        include!("tendermint.consensus.rs");
        // @@protoc_insertion_point(tendermint.consensus)
    }
    #[cfg(feature = "tendermint+crypto")]
    // @@protoc_insertion_point(attribute:tendermint.crypto)
    pub mod crypto {
        include!("tendermint.crypto.rs");
        // @@protoc_insertion_point(tendermint.crypto)
    }
    pub mod libs {
        #[cfg(feature = "tendermint+libs+bits")]
        // @@protoc_insertion_point(attribute:tendermint.libs.bits)
        pub mod bits {
            include!("tendermint.libs.bits.rs");
            // @@protoc_insertion_point(tendermint.libs.bits)
        }
    }
    #[cfg(feature = "tendermint+mempool")]
    // @@protoc_insertion_point(attribute:tendermint.mempool)
    pub mod mempool {
        include!("tendermint.mempool.rs");
        // @@protoc_insertion_point(tendermint.mempool)
    }
    #[cfg(feature = "tendermint+p2p")]
    // @@protoc_insertion_point(attribute:tendermint.p2p)
    pub mod p2p {
        include!("tendermint.p2p.rs");
        // @@protoc_insertion_point(tendermint.p2p)
    }
    #[cfg(feature = "tendermint+privval")]
    // @@protoc_insertion_point(attribute:tendermint.privval)
    pub mod privval {
        include!("tendermint.privval.rs");
        // @@protoc_insertion_point(tendermint.privval)
    }
    pub mod rpc {
        #[cfg(feature = "tendermint+rpc+grpc")]
        // @@protoc_insertion_point(attribute:tendermint.rpc.grpc)
        pub mod grpc {
            include!("tendermint.rpc.grpc.rs");
            // @@protoc_insertion_point(tendermint.rpc.grpc)
        }
    }
    #[cfg(feature = "tendermint+state")]
    // @@protoc_insertion_point(attribute:tendermint.state)
    pub mod state {
        include!("tendermint.state.rs");
        // @@protoc_insertion_point(tendermint.state)
    }
    #[cfg(feature = "tendermint+statesync")]
    // @@protoc_insertion_point(attribute:tendermint.statesync)
    pub mod statesync {
        include!("tendermint.statesync.rs");
        // @@protoc_insertion_point(tendermint.statesync)
    }
    #[cfg(feature = "tendermint+store")]
    // @@protoc_insertion_point(attribute:tendermint.store)
    pub mod store {
        include!("tendermint.store.rs");
        // @@protoc_insertion_point(tendermint.store)
    }
    #[cfg(feature = "tendermint+types")]
    // @@protoc_insertion_point(attribute:tendermint.types)
    pub mod types {
        include!("tendermint.types.rs");
        // @@protoc_insertion_point(tendermint.types)
    }
    #[cfg(feature = "tendermint+version")]
    // @@protoc_insertion_point(attribute:tendermint.version)
    pub mod version {
        include!("tendermint.version.rs");
        // @@protoc_insertion_point(tendermint.version)
    }
}
pub mod tokenfactory {
    #[cfg(feature = "tokenfactory+v1beta1")]
    // @@protoc_insertion_point(attribute:tokenfactory.v1beta1)
    pub mod v1beta1 {
        include!("tokenfactory.v1beta1.rs");
        // @@protoc_insertion_point(tokenfactory.v1beta1)
    }
}
pub mod union {
    pub mod galois {
        pub mod api {
            #[cfg(feature = "union+galois+api+v1")]
            // @@protoc_insertion_point(attribute:union.galois.api.v1)
            pub mod v1 {
                include!("union.galois.api.v1.rs");
                // @@protoc_insertion_point(union.galois.api.v1)
            }
            #[cfg(feature = "union+galois+api+v2")]
            // @@protoc_insertion_point(attribute:union.galois.api.v2)
            pub mod v2 {
                include!("union.galois.api.v2.rs");
                // @@protoc_insertion_point(union.galois.api.v2)
            }
            #[cfg(feature = "union+galois+api+v3")]
            // @@protoc_insertion_point(attribute:union.galois.api.v3)
            pub mod v3 {
                include!("union.galois.api.v3.rs");
                // @@protoc_insertion_point(union.galois.api.v3)
            }
        }
    }
    pub mod ibc {
        pub mod lightclients {
            pub mod arbitrum {
                #[cfg(feature = "union+ibc+lightclients+arbitrum+v1")]
                // @@protoc_insertion_point(attribute:union.ibc.lightclients.arbitrum.v1)
                pub mod v1 {
                    include!("union.ibc.lightclients.arbitrum.v1.rs");
                    // @@protoc_insertion_point(union.ibc.lightclients.arbitrum.v1)
                }
            }
            pub mod cometbls {
                #[cfg(feature = "union+ibc+lightclients+cometbls+v1")]
                // @@protoc_insertion_point(attribute:union.ibc.lightclients.cometbls.v1)
                pub mod v1 {
                    include!("union.ibc.lightclients.cometbls.v1.rs");
                    // @@protoc_insertion_point(union.ibc.lightclients.cometbls.v1)
                }
            }
            pub mod ethereum {
                #[cfg(feature = "union+ibc+lightclients+ethereum+v1")]
                // @@protoc_insertion_point(attribute:union.ibc.lightclients.ethereum.v1)
                pub mod v1 {
                    include!("union.ibc.lightclients.ethereum.v1.rs");
                    // @@protoc_insertion_point(union.ibc.lightclients.ethereum.v1)
                }
            }
            pub mod linea {
                #[cfg(feature = "union+ibc+lightclients+linea+v1")]
                // @@protoc_insertion_point(attribute:union.ibc.lightclients.linea.v1)
                pub mod v1 {
                    include!("union.ibc.lightclients.linea.v1.rs");
                    // @@protoc_insertion_point(union.ibc.lightclients.linea.v1)
                }
            }
            pub mod scroll {
                #[cfg(feature = "union+ibc+lightclients+scroll+v1")]
                // @@protoc_insertion_point(attribute:union.ibc.lightclients.scroll.v1)
                pub mod v1 {
                    include!("union.ibc.lightclients.scroll.v1.rs");
                    // @@protoc_insertion_point(union.ibc.lightclients.scroll.v1)
                }
            }
        }
    }
    pub mod ics23 {
        #[cfg(feature = "union+ics23+v1")]
        // @@protoc_insertion_point(attribute:union.ics23.v1)
        pub mod v1 {
            include!("union.ics23.v1.rs");
            // @@protoc_insertion_point(union.ics23.v1)
        }
    }
    pub mod staking {
        #[cfg(feature = "union+staking+v1")]
        // @@protoc_insertion_point(attribute:union.staking.v1)
        pub mod v1 {
            include!("union.staking.v1.rs");
            // @@protoc_insertion_point(union.staking.v1)
        }
    }
}
