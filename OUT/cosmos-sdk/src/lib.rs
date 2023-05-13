// @generated
pub mod cosmos {
    pub mod app {
        pub mod runtime {
            #[cfg(feature = "cosmos-app-runtime-v1alpha1")]
            // @@protoc_insertion_point(attribute:cosmos.app.runtime.v1alpha1)
            pub mod v1alpha1 {
                include!("cosmos.app.runtime.v1alpha1.rs");
                // @@protoc_insertion_point(cosmos.app.runtime.v1alpha1)
            }
        }
        #[cfg(feature = "cosmos-app-v1alpha1")]
        // @@protoc_insertion_point(attribute:cosmos.app.v1alpha1)
        pub mod v1alpha1 {
            include!("cosmos.app.v1alpha1.rs");
            // @@protoc_insertion_point(cosmos.app.v1alpha1)
        }
    }
    pub mod auth {
        pub mod module {
            #[cfg(feature = "cosmos-auth-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.auth.module.v1)
            pub mod v1 {
                include!("cosmos.auth.module.v1.rs");
                // @@protoc_insertion_point(cosmos.auth.module.v1)
            }
        }
        #[cfg(feature = "cosmos-auth-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.auth.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.auth.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.auth.v1beta1)
        }
    }
    pub mod authz {
        pub mod module {
            #[cfg(feature = "cosmos-authz-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.authz.module.v1)
            pub mod v1 {
                include!("cosmos.authz.module.v1.rs");
                // @@protoc_insertion_point(cosmos.authz.module.v1)
            }
        }
        #[cfg(feature = "cosmos-authz-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.authz.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.authz.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.authz.v1beta1)
        }
    }
    pub mod autocli {
        #[cfg(feature = "cosmos-autocli-v1")]
        // @@protoc_insertion_point(attribute:cosmos.autocli.v1)
        pub mod v1 {
            include!("cosmos.autocli.v1.rs");
            // @@protoc_insertion_point(cosmos.autocli.v1)
        }
    }
    pub mod bank {
        pub mod module {
            #[cfg(feature = "cosmos-bank-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.bank.module.v1)
            pub mod v1 {
                include!("cosmos.bank.module.v1.rs");
                // @@protoc_insertion_point(cosmos.bank.module.v1)
            }
        }
        #[cfg(feature = "cosmos-bank-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.bank.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.bank.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.bank.v1beta1)
        }
    }
    pub mod base {
        pub mod abci {
            #[cfg(feature = "cosmos-base-abci-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.abci.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.abci.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.abci.v1beta1)
            }
        }
        pub mod kv {
            #[cfg(feature = "cosmos-base-kv-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.kv.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.kv.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.kv.v1beta1)
            }
        }
        pub mod node {
            #[cfg(feature = "cosmos-base-node-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.node.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.node.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.node.v1beta1)
            }
        }
        pub mod query {
            #[cfg(feature = "cosmos-base-query-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.query.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.query.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.query.v1beta1)
            }
        }
        pub mod reflection {
            #[cfg(feature = "cosmos-base-reflection-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.reflection.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.reflection.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.reflection.v1beta1)
            }
            #[cfg(feature = "cosmos-base-reflection-v2alpha1")]
            // @@protoc_insertion_point(attribute:cosmos.base.reflection.v2alpha1)
            pub mod v2alpha1 {
                include!("cosmos.base.reflection.v2alpha1.rs");
                // @@protoc_insertion_point(cosmos.base.reflection.v2alpha1)
            }
        }
        pub mod snapshots {
            #[cfg(feature = "cosmos-base-snapshots-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.snapshots.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.snapshots.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.snapshots.v1beta1)
            }
        }
        pub mod store {
            #[cfg(feature = "cosmos-base-store-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.store.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.store.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.store.v1beta1)
            }
        }
        pub mod tendermint {
            #[cfg(feature = "cosmos-base-tendermint-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.base.tendermint.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.base.tendermint.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.base.tendermint.v1beta1)
            }
        }
        #[cfg(feature = "cosmos-base-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.base.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.base.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.base.v1beta1)
        }
    }
    pub mod capability {
        pub mod module {
            #[cfg(feature = "cosmos-capability-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.capability.module.v1)
            pub mod v1 {
                include!("cosmos.capability.module.v1.rs");
                // @@protoc_insertion_point(cosmos.capability.module.v1)
            }
        }
        #[cfg(feature = "cosmos-capability-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.capability.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.capability.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.capability.v1beta1)
        }
    }
    pub mod consensus {
        pub mod module {
            #[cfg(feature = "cosmos-consensus-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.consensus.module.v1)
            pub mod v1 {
                include!("cosmos.consensus.module.v1.rs");
                // @@protoc_insertion_point(cosmos.consensus.module.v1)
            }
        }
        #[cfg(feature = "cosmos-consensus-v1")]
        // @@protoc_insertion_point(attribute:cosmos.consensus.v1)
        pub mod v1 {
            include!("cosmos.consensus.v1.rs");
            // @@protoc_insertion_point(cosmos.consensus.v1)
        }
    }
    pub mod crisis {
        pub mod module {
            #[cfg(feature = "cosmos-crisis-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.crisis.module.v1)
            pub mod v1 {
                include!("cosmos.crisis.module.v1.rs");
                // @@protoc_insertion_point(cosmos.crisis.module.v1)
            }
        }
        #[cfg(feature = "cosmos-crisis-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.crisis.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.crisis.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.crisis.v1beta1)
        }
    }
    pub mod crypto {
        #[cfg(feature = "cosmos-crypto-bn254")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.bn254)
        pub mod bn254 {
            include!("cosmos.crypto.bn254.rs");
            // @@protoc_insertion_point(cosmos.crypto.bn254)
        }
        #[cfg(feature = "cosmos-crypto-ed25519")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.ed25519)
        pub mod ed25519 {
            include!("cosmos.crypto.ed25519.rs");
            // @@protoc_insertion_point(cosmos.crypto.ed25519)
        }
        pub mod hd {
            #[cfg(feature = "cosmos-crypto-hd-v1")]
            // @@protoc_insertion_point(attribute:cosmos.crypto.hd.v1)
            pub mod v1 {
                include!("cosmos.crypto.hd.v1.rs");
                // @@protoc_insertion_point(cosmos.crypto.hd.v1)
            }
        }
        pub mod keyring {
            #[cfg(feature = "cosmos-crypto-keyring-v1")]
            // @@protoc_insertion_point(attribute:cosmos.crypto.keyring.v1)
            pub mod v1 {
                include!("cosmos.crypto.keyring.v1.rs");
                // @@protoc_insertion_point(cosmos.crypto.keyring.v1)
            }
        }
        #[cfg(feature = "cosmos-crypto-multisig")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.multisig)
        pub mod multisig {
            include!("cosmos.crypto.multisig.rs");
            // @@protoc_insertion_point(cosmos.crypto.multisig)
            #[cfg(feature = "cosmos-crypto-multisig-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.crypto.multisig.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.crypto.multisig.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.crypto.multisig.v1beta1)
            }
        }
        #[cfg(feature = "cosmos-crypto-secp256k1")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.secp256k1)
        pub mod secp256k1 {
            include!("cosmos.crypto.secp256k1.rs");
            // @@protoc_insertion_point(cosmos.crypto.secp256k1)
        }
        #[cfg(feature = "cosmos-crypto-secp256r1")]
        // @@protoc_insertion_point(attribute:cosmos.crypto.secp256r1)
        pub mod secp256r1 {
            include!("cosmos.crypto.secp256r1.rs");
            // @@protoc_insertion_point(cosmos.crypto.secp256r1)
        }
    }
    pub mod distribution {
        pub mod module {
            #[cfg(feature = "cosmos-distribution-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.distribution.module.v1)
            pub mod v1 {
                include!("cosmos.distribution.module.v1.rs");
                // @@protoc_insertion_point(cosmos.distribution.module.v1)
            }
        }
        #[cfg(feature = "cosmos-distribution-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.distribution.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.distribution.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.distribution.v1beta1)
        }
    }
    pub mod evidence {
        pub mod module {
            #[cfg(feature = "cosmos-evidence-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.evidence.module.v1)
            pub mod v1 {
                include!("cosmos.evidence.module.v1.rs");
                // @@protoc_insertion_point(cosmos.evidence.module.v1)
            }
        }
        #[cfg(feature = "cosmos-evidence-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.evidence.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.evidence.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.evidence.v1beta1)
        }
    }
    pub mod feegrant {
        pub mod module {
            #[cfg(feature = "cosmos-feegrant-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.feegrant.module.v1)
            pub mod v1 {
                include!("cosmos.feegrant.module.v1.rs");
                // @@protoc_insertion_point(cosmos.feegrant.module.v1)
            }
        }
        #[cfg(feature = "cosmos-feegrant-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.feegrant.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.feegrant.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.feegrant.v1beta1)
        }
    }
    pub mod genutil {
        pub mod module {
            #[cfg(feature = "cosmos-genutil-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.genutil.module.v1)
            pub mod v1 {
                include!("cosmos.genutil.module.v1.rs");
                // @@protoc_insertion_point(cosmos.genutil.module.v1)
            }
        }
        #[cfg(feature = "cosmos-genutil-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.genutil.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.genutil.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.genutil.v1beta1)
        }
    }
    pub mod gov {
        pub mod module {
            #[cfg(feature = "cosmos-gov-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.gov.module.v1)
            pub mod v1 {
                include!("cosmos.gov.module.v1.rs");
                // @@protoc_insertion_point(cosmos.gov.module.v1)
            }
        }
        #[cfg(feature = "cosmos-gov-v1")]
        // @@protoc_insertion_point(attribute:cosmos.gov.v1)
        pub mod v1 {
            include!("cosmos.gov.v1.rs");
            // @@protoc_insertion_point(cosmos.gov.v1)
        }
        #[cfg(feature = "cosmos-gov-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.gov.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.gov.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.gov.v1beta1)
        }
    }
    pub mod group {
        pub mod module {
            #[cfg(feature = "cosmos-group-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.group.module.v1)
            pub mod v1 {
                include!("cosmos.group.module.v1.rs");
                // @@protoc_insertion_point(cosmos.group.module.v1)
            }
        }
        #[cfg(feature = "cosmos-group-v1")]
        // @@protoc_insertion_point(attribute:cosmos.group.v1)
        pub mod v1 {
            include!("cosmos.group.v1.rs");
            // @@protoc_insertion_point(cosmos.group.v1)
        }
    }
    pub mod mint {
        pub mod module {
            #[cfg(feature = "cosmos-mint-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.mint.module.v1)
            pub mod v1 {
                include!("cosmos.mint.module.v1.rs");
                // @@protoc_insertion_point(cosmos.mint.module.v1)
            }
        }
        #[cfg(feature = "cosmos-mint-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.mint.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.mint.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.mint.v1beta1)
        }
    }
    pub mod nft {
        pub mod module {
            #[cfg(feature = "cosmos-nft-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.nft.module.v1)
            pub mod v1 {
                include!("cosmos.nft.module.v1.rs");
                // @@protoc_insertion_point(cosmos.nft.module.v1)
            }
        }
        #[cfg(feature = "cosmos-nft-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.nft.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.nft.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.nft.v1beta1)
        }
    }
    pub mod orm {
        pub mod module {
            #[cfg(feature = "cosmos-orm-module-v1alpha1")]
            // @@protoc_insertion_point(attribute:cosmos.orm.module.v1alpha1)
            pub mod v1alpha1 {
                include!("cosmos.orm.module.v1alpha1.rs");
                // @@protoc_insertion_point(cosmos.orm.module.v1alpha1)
            }
        }
        pub mod query {
            #[cfg(feature = "cosmos-orm-query-v1alpha1")]
            // @@protoc_insertion_point(attribute:cosmos.orm.query.v1alpha1)
            pub mod v1alpha1 {
                include!("cosmos.orm.query.v1alpha1.rs");
                // @@protoc_insertion_point(cosmos.orm.query.v1alpha1)
            }
        }
        #[cfg(feature = "cosmos-orm-v1")]
        // @@protoc_insertion_point(attribute:cosmos.orm.v1)
        pub mod v1 {
            include!("cosmos.orm.v1.rs");
            // @@protoc_insertion_point(cosmos.orm.v1)
        }
        #[cfg(feature = "cosmos-orm-v1alpha1")]
        // @@protoc_insertion_point(attribute:cosmos.orm.v1alpha1)
        pub mod v1alpha1 {
            include!("cosmos.orm.v1alpha1.rs");
            // @@protoc_insertion_point(cosmos.orm.v1alpha1)
        }
    }
    pub mod params {
        pub mod module {
            #[cfg(feature = "cosmos-params-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.params.module.v1)
            pub mod v1 {
                include!("cosmos.params.module.v1.rs");
                // @@protoc_insertion_point(cosmos.params.module.v1)
            }
        }
        #[cfg(feature = "cosmos-params-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.params.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.params.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.params.v1beta1)
        }
    }
    pub mod reflection {
        #[cfg(feature = "cosmos-reflection-v1")]
        // @@protoc_insertion_point(attribute:cosmos.reflection.v1)
        pub mod v1 {
            include!("cosmos.reflection.v1.rs");
            // @@protoc_insertion_point(cosmos.reflection.v1)
        }
    }
    pub mod slashing {
        pub mod module {
            #[cfg(feature = "cosmos-slashing-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.slashing.module.v1)
            pub mod v1 {
                include!("cosmos.slashing.module.v1.rs");
                // @@protoc_insertion_point(cosmos.slashing.module.v1)
            }
        }
        #[cfg(feature = "cosmos-slashing-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.slashing.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.slashing.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.slashing.v1beta1)
        }
    }
    pub mod staking {
        pub mod module {
            #[cfg(feature = "cosmos-staking-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.staking.module.v1)
            pub mod v1 {
                include!("cosmos.staking.module.v1.rs");
                // @@protoc_insertion_point(cosmos.staking.module.v1)
            }
        }
        #[cfg(feature = "cosmos-staking-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.staking.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.staking.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.staking.v1beta1)
        }
    }
    pub mod tx {
        pub mod config {
            #[cfg(feature = "cosmos-tx-config-v1")]
            // @@protoc_insertion_point(attribute:cosmos.tx.config.v1)
            pub mod v1 {
                include!("cosmos.tx.config.v1.rs");
                // @@protoc_insertion_point(cosmos.tx.config.v1)
            }
        }
        pub mod signing {
            #[cfg(feature = "cosmos-tx-signing-v1beta1")]
            // @@protoc_insertion_point(attribute:cosmos.tx.signing.v1beta1)
            pub mod v1beta1 {
                include!("cosmos.tx.signing.v1beta1.rs");
                // @@protoc_insertion_point(cosmos.tx.signing.v1beta1)
            }
        }
        #[cfg(feature = "cosmos-tx-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.tx.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.tx.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.tx.v1beta1)
        }
    }
    pub mod upgrade {
        pub mod module {
            #[cfg(feature = "cosmos-upgrade-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.upgrade.module.v1)
            pub mod v1 {
                include!("cosmos.upgrade.module.v1.rs");
                // @@protoc_insertion_point(cosmos.upgrade.module.v1)
            }
        }
        #[cfg(feature = "cosmos-upgrade-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.upgrade.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.upgrade.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.upgrade.v1beta1)
        }
    }
    pub mod vesting {
        pub mod module {
            #[cfg(feature = "cosmos-vesting-module-v1")]
            // @@protoc_insertion_point(attribute:cosmos.vesting.module.v1)
            pub mod v1 {
                include!("cosmos.vesting.module.v1.rs");
                // @@protoc_insertion_point(cosmos.vesting.module.v1)
            }
        }
        #[cfg(feature = "cosmos-vesting-v1beta1")]
        // @@protoc_insertion_point(attribute:cosmos.vesting.v1beta1)
        pub mod v1beta1 {
            include!("cosmos.vesting.v1beta1.rs");
            // @@protoc_insertion_point(cosmos.vesting.v1beta1)
        }
    }
}
