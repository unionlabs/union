#![allow(clippy::all, rustdoc::all)]
#[cfg(feature = "amino")]
pub mod amino {
    include!("amino.rs");
}
pub mod babylon {
    #[cfg(feature = "babylon+incentive")]
    pub mod incentive {
        include!("babylon.incentive.rs");
    }
    pub mod btccheckpoint {
        #[cfg(feature = "babylon+btccheckpoint+v1")]
        pub mod v1 {
            include!("babylon.btccheckpoint.v1.rs");
        }
    }
    pub mod btclightclient {
        #[cfg(feature = "babylon+btclightclient+v1")]
        pub mod v1 {
            include!("babylon.btclightclient.v1.rs");
        }
    }
    pub mod btcstaking {
        #[cfg(feature = "babylon+btcstaking+v1")]
        pub mod v1 {
            include!("babylon.btcstaking.v1.rs");
        }
    }
    pub mod checkpointing {
        #[cfg(feature = "babylon+checkpointing+v1")]
        pub mod v1 {
            include!("babylon.checkpointing.v1.rs");
        }
    }
    pub mod epoching {
        #[cfg(feature = "babylon+epoching+v1")]
        pub mod v1 {
            include!("babylon.epoching.v1.rs");
        }
    }
    pub mod finality {
        #[cfg(feature = "babylon+finality+v1")]
        pub mod v1 {
            include!("babylon.finality.v1.rs");
        }
    }
    pub mod mint {
        #[cfg(feature = "babylon+mint+v1")]
        pub mod v1 {
            include!("babylon.mint.v1.rs");
        }
    }
    pub mod monitor {
        #[cfg(feature = "babylon+monitor+v1")]
        pub mod v1 {
            include!("babylon.monitor.v1.rs");
        }
    }
}
pub mod capability {
    #[cfg(feature = "capability+v1")]
    pub mod v1 {
        include!("capability.v1.rs");
    }
}
pub mod cometbft {
    pub mod abci {
        #[cfg(feature = "cometbft+abci+v1")]
        pub mod v1 {
            include!("cometbft.abci.v1.rs");
        }
        #[cfg(feature = "cometbft+abci+v1beta1")]
        pub mod v1beta1 {
            include!("cometbft.abci.v1beta1.rs");
        }
        #[cfg(feature = "cometbft+abci+v1beta2")]
        pub mod v1beta2 {
            include!("cometbft.abci.v1beta2.rs");
        }
        #[cfg(feature = "cometbft+abci+v1beta3")]
        pub mod v1beta3 {
            include!("cometbft.abci.v1beta3.rs");
        }
    }
    pub mod blocksync {
        #[cfg(feature = "cometbft+blocksync+v1")]
        pub mod v1 {
            include!("cometbft.blocksync.v1.rs");
        }
        #[cfg(feature = "cometbft+blocksync+v1beta1")]
        pub mod v1beta1 {
            include!("cometbft.blocksync.v1beta1.rs");
        }
    }
    pub mod consensus {
        #[cfg(feature = "cometbft+consensus+v1")]
        pub mod v1 {
            include!("cometbft.consensus.v1.rs");
        }
        #[cfg(feature = "cometbft+consensus+v1beta1")]
        pub mod v1beta1 {
            include!("cometbft.consensus.v1beta1.rs");
        }
    }
    pub mod crypto {
        #[cfg(feature = "cometbft+crypto+v1")]
        pub mod v1 {
            include!("cometbft.crypto.v1.rs");
        }
    }
    pub mod libs {
        pub mod bits {
            #[cfg(feature = "cometbft+libs+bits+v1")]
            pub mod v1 {
                include!("cometbft.libs.bits.v1.rs");
            }
        }
    }
    pub mod mempool {
        #[cfg(feature = "cometbft+mempool+v1")]
        pub mod v1 {
            include!("cometbft.mempool.v1.rs");
        }
    }
    pub mod p2p {
        #[cfg(feature = "cometbft+p2p+v1")]
        pub mod v1 {
            include!("cometbft.p2p.v1.rs");
        }
    }
    pub mod privval {
        #[cfg(feature = "cometbft+privval+v1")]
        pub mod v1 {
            include!("cometbft.privval.v1.rs");
        }
        #[cfg(feature = "cometbft+privval+v1beta1")]
        pub mod v1beta1 {
            include!("cometbft.privval.v1beta1.rs");
        }
        #[cfg(feature = "cometbft+privval+v1beta2")]
        pub mod v1beta2 {
            include!("cometbft.privval.v1beta2.rs");
        }
    }
    pub mod rpc {
        pub mod grpc {
            #[cfg(feature = "cometbft+rpc+grpc+v1beta1")]
            pub mod v1beta1 {
                include!("cometbft.rpc.grpc.v1beta1.rs");
            }
            #[cfg(feature = "cometbft+rpc+grpc+v1beta2")]
            pub mod v1beta2 {
                include!("cometbft.rpc.grpc.v1beta2.rs");
            }
            #[cfg(feature = "cometbft+rpc+grpc+v1beta3")]
            pub mod v1beta3 {
                include!("cometbft.rpc.grpc.v1beta3.rs");
            }
        }
    }
    pub mod services {
        pub mod block {
            #[cfg(feature = "cometbft+services+block+v1")]
            pub mod v1 {
                include!("cometbft.services.block.v1.rs");
            }
        }
        pub mod block_results {
            #[cfg(feature = "cometbft+services+block_results+v1")]
            pub mod v1 {
                include!("cometbft.services.block_results.v1.rs");
            }
        }
        pub mod pruning {
            #[cfg(feature = "cometbft+services+pruning+v1")]
            pub mod v1 {
                include!("cometbft.services.pruning.v1.rs");
            }
        }
        pub mod version {
            #[cfg(feature = "cometbft+services+version+v1")]
            pub mod v1 {
                include!("cometbft.services.version.v1.rs");
            }
        }
    }
    pub mod state {
        #[cfg(feature = "cometbft+state+v1")]
        pub mod v1 {
            include!("cometbft.state.v1.rs");
        }
        #[cfg(feature = "cometbft+state+v1beta1")]
        pub mod v1beta1 {
            include!("cometbft.state.v1beta1.rs");
        }
        #[cfg(feature = "cometbft+state+v1beta2")]
        pub mod v1beta2 {
            include!("cometbft.state.v1beta2.rs");
        }
        #[cfg(feature = "cometbft+state+v1beta3")]
        pub mod v1beta3 {
            include!("cometbft.state.v1beta3.rs");
        }
    }
    pub mod statesync {
        #[cfg(feature = "cometbft+statesync+v1")]
        pub mod v1 {
            include!("cometbft.statesync.v1.rs");
        }
    }
    pub mod store {
        #[cfg(feature = "cometbft+store+v1")]
        pub mod v1 {
            include!("cometbft.store.v1.rs");
        }
    }
    pub mod types {
        #[cfg(feature = "cometbft+types+v1")]
        pub mod v1 {
            include!("cometbft.types.v1.rs");
        }
        #[cfg(feature = "cometbft+types+v1beta1")]
        pub mod v1beta1 {
            include!("cometbft.types.v1beta1.rs");
        }
        #[cfg(feature = "cometbft+types+v1beta2")]
        pub mod v1beta2 {
            include!("cometbft.types.v1beta2.rs");
        }
    }
    pub mod version {
        #[cfg(feature = "cometbft+version+v1")]
        pub mod v1 {
            include!("cometbft.version.v1.rs");
        }
    }
}
pub mod cosmos {
    pub mod app {
        #[cfg(feature = "cosmos+app+v1alpha1")]
        pub mod v1alpha1 {
            include!("cosmos.app.v1alpha1.rs");
        }
        pub mod runtime {
            #[cfg(feature = "cosmos+app+runtime+v1alpha1")]
            pub mod v1alpha1 {
                include!("cosmos.app.runtime.v1alpha1.rs");
            }
        }
    }
    pub mod auth {
        #[cfg(feature = "cosmos+auth+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.auth.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+auth+module+v1")]
            pub mod v1 {
                include!("cosmos.auth.module.v1.rs");
            }
        }
    }
    pub mod authz {
        #[cfg(feature = "cosmos+authz+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.authz.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+authz+module+v1")]
            pub mod v1 {
                include!("cosmos.authz.module.v1.rs");
            }
        }
    }
    pub mod autocli {
        #[cfg(feature = "cosmos+autocli+v1")]
        pub mod v1 {
            include!("cosmos.autocli.v1.rs");
        }
    }
    pub mod bank {
        #[cfg(feature = "cosmos+bank+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.bank.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+bank+module+v1")]
            pub mod v1 {
                include!("cosmos.bank.module.v1.rs");
            }
        }
    }
    pub mod base {
        #[cfg(feature = "cosmos+base+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.base.v1beta1.rs");
        }
        pub mod abci {
            #[cfg(feature = "cosmos+base+abci+v1beta1")]
            pub mod v1beta1 {
                include!("cosmos.base.abci.v1beta1.rs");
            }
        }
        pub mod node {
            #[cfg(feature = "cosmos+base+node+v1beta1")]
            pub mod v1beta1 {
                include!("cosmos.base.node.v1beta1.rs");
            }
        }
        pub mod query {
            #[cfg(feature = "cosmos+base+query+v1beta1")]
            pub mod v1beta1 {
                include!("cosmos.base.query.v1beta1.rs");
            }
        }
        pub mod reflection {
            #[cfg(feature = "cosmos+base+reflection+v1beta1")]
            pub mod v1beta1 {
                include!("cosmos.base.reflection.v1beta1.rs");
            }
            #[cfg(feature = "cosmos+base+reflection+v2alpha1")]
            pub mod v2alpha1 {
                include!("cosmos.base.reflection.v2alpha1.rs");
            }
        }
        pub mod tendermint {
            #[cfg(feature = "cosmos+base+tendermint+v1beta1")]
            pub mod v1beta1 {
                include!("cosmos.base.tendermint.v1beta1.rs");
            }
        }
    }
    pub mod circuit {
        #[cfg(feature = "cosmos+circuit+v1")]
        pub mod v1 {
            include!("cosmos.circuit.v1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+circuit+module+v1")]
            pub mod v1 {
                include!("cosmos.circuit.module.v1.rs");
            }
        }
    }
    pub mod consensus {
        #[cfg(feature = "cosmos+consensus+v1")]
        pub mod v1 {
            include!("cosmos.consensus.v1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+consensus+module+v1")]
            pub mod v1 {
                include!("cosmos.consensus.module.v1.rs");
            }
        }
    }
    pub mod crisis {
        #[cfg(feature = "cosmos+crisis+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.crisis.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+crisis+module+v1")]
            pub mod v1 {
                include!("cosmos.crisis.module.v1.rs");
            }
        }
    }
    pub mod crypto {
        #[cfg(feature = "cosmos+crypto+bn254")]
        pub mod bn254 {
            include!("cosmos.crypto.bn254.rs");
        }
        #[cfg(feature = "cosmos+crypto+ed25519")]
        pub mod ed25519 {
            include!("cosmos.crypto.ed25519.rs");
        }
        #[cfg(feature = "cosmos+crypto+multisig")]
        pub mod multisig {
            #[cfg(feature = "cosmos+crypto+multisig+v1beta1")]
            pub mod v1beta1 {
                include!("cosmos.crypto.multisig.v1beta1.rs");
            }
            include!("cosmos.crypto.multisig.rs");
        }
        #[cfg(feature = "cosmos+crypto+secp256k1")]
        pub mod secp256k1 {
            include!("cosmos.crypto.secp256k1.rs");
        }
        #[cfg(feature = "cosmos+crypto+secp256r1")]
        pub mod secp256r1 {
            include!("cosmos.crypto.secp256r1.rs");
        }
        pub mod hd {
            #[cfg(feature = "cosmos+crypto+hd+v1")]
            pub mod v1 {
                include!("cosmos.crypto.hd.v1.rs");
            }
        }
        pub mod keyring {
            #[cfg(feature = "cosmos+crypto+keyring+v1")]
            pub mod v1 {
                include!("cosmos.crypto.keyring.v1.rs");
            }
        }
    }
    pub mod distribution {
        #[cfg(feature = "cosmos+distribution+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.distribution.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+distribution+module+v1")]
            pub mod v1 {
                include!("cosmos.distribution.module.v1.rs");
            }
        }
    }
    pub mod evidence {
        #[cfg(feature = "cosmos+evidence+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.evidence.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+evidence+module+v1")]
            pub mod v1 {
                include!("cosmos.evidence.module.v1.rs");
            }
        }
    }
    pub mod feegrant {
        #[cfg(feature = "cosmos+feegrant+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.feegrant.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+feegrant+module+v1")]
            pub mod v1 {
                include!("cosmos.feegrant.module.v1.rs");
            }
        }
    }
    pub mod genutil {
        #[cfg(feature = "cosmos+genutil+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.genutil.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+genutil+module+v1")]
            pub mod v1 {
                include!("cosmos.genutil.module.v1.rs");
            }
        }
    }
    pub mod gov {
        #[cfg(feature = "cosmos+gov+v1")]
        pub mod v1 {
            include!("cosmos.gov.v1.rs");
        }
        #[cfg(feature = "cosmos+gov+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.gov.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+gov+module+v1")]
            pub mod v1 {
                include!("cosmos.gov.module.v1.rs");
            }
        }
    }
    pub mod group {
        #[cfg(feature = "cosmos+group+v1")]
        pub mod v1 {
            include!("cosmos.group.v1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+group+module+v1")]
            pub mod v1 {
                include!("cosmos.group.module.v1.rs");
            }
        }
    }
    pub mod ics23 {
        #[cfg(feature = "cosmos+ics23+v1")]
        pub mod v1 {
            include!("cosmos.ics23.v1.rs");
        }
    }
    pub mod mint {
        #[cfg(feature = "cosmos+mint+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.mint.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+mint+module+v1")]
            pub mod v1 {
                include!("cosmos.mint.module.v1.rs");
            }
        }
    }
    pub mod nft {
        #[cfg(feature = "cosmos+nft+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.nft.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+nft+module+v1")]
            pub mod v1 {
                include!("cosmos.nft.module.v1.rs");
            }
        }
    }
    pub mod orm {
        #[cfg(feature = "cosmos+orm+v1")]
        pub mod v1 {
            include!("cosmos.orm.v1.rs");
        }
        #[cfg(feature = "cosmos+orm+v1alpha1")]
        pub mod v1alpha1 {
            include!("cosmos.orm.v1alpha1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+orm+module+v1alpha1")]
            pub mod v1alpha1 {
                include!("cosmos.orm.module.v1alpha1.rs");
            }
        }
        pub mod query {
            #[cfg(feature = "cosmos+orm+query+v1alpha1")]
            pub mod v1alpha1 {
                include!("cosmos.orm.query.v1alpha1.rs");
            }
        }
    }
    pub mod params {
        #[cfg(feature = "cosmos+params+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.params.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+params+module+v1")]
            pub mod v1 {
                include!("cosmos.params.module.v1.rs");
            }
        }
    }
    pub mod reflection {
        #[cfg(feature = "cosmos+reflection+v1")]
        pub mod v1 {
            include!("cosmos.reflection.v1.rs");
        }
    }
    pub mod slashing {
        #[cfg(feature = "cosmos+slashing+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.slashing.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+slashing+module+v1")]
            pub mod v1 {
                include!("cosmos.slashing.module.v1.rs");
            }
        }
    }
    pub mod staking {
        #[cfg(feature = "cosmos+staking+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.staking.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+staking+module+v1")]
            pub mod v1 {
                include!("cosmos.staking.module.v1.rs");
            }
        }
    }
    pub mod store {
        #[cfg(feature = "cosmos+store+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.store.v1beta1.rs");
        }
        pub mod internal {
            pub mod kv {
                #[cfg(feature = "cosmos+store+internal+kv+v1beta1")]
                pub mod v1beta1 {
                    include!("cosmos.store.internal.kv.v1beta1.rs");
                }
            }
        }
        pub mod snapshots {
            #[cfg(feature = "cosmos+store+snapshots+v1")]
            pub mod v1 {
                include!("cosmos.store.snapshots.v1.rs");
            }
        }
        pub mod streaming {
            #[cfg(feature = "cosmos+store+streaming+abci")]
            pub mod abci {
                include!("cosmos.store.streaming.abci.rs");
            }
        }
    }
    pub mod tx {
        #[cfg(feature = "cosmos+tx+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.tx.v1beta1.rs");
        }
        pub mod config {
            #[cfg(feature = "cosmos+tx+config+v1")]
            pub mod v1 {
                include!("cosmos.tx.config.v1.rs");
            }
        }
        pub mod signing {
            #[cfg(feature = "cosmos+tx+signing+v1beta1")]
            pub mod v1beta1 {
                include!("cosmos.tx.signing.v1beta1.rs");
            }
        }
    }
    pub mod upgrade {
        #[cfg(feature = "cosmos+upgrade+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.upgrade.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+upgrade+module+v1")]
            pub mod v1 {
                include!("cosmos.upgrade.module.v1.rs");
            }
        }
    }
    pub mod vesting {
        #[cfg(feature = "cosmos+vesting+v1beta1")]
        pub mod v1beta1 {
            include!("cosmos.vesting.v1beta1.rs");
        }
        pub mod module {
            #[cfg(feature = "cosmos+vesting+module+v1")]
            pub mod v1 {
                include!("cosmos.vesting.module.v1.rs");
            }
        }
    }
}
pub mod cosmwasm {
    pub mod wasm {
        #[cfg(feature = "cosmwasm+wasm+v1")]
        pub mod v1 {
            include!("cosmwasm.wasm.v1.rs");
        }
    }
}
pub mod feemarket {
    pub mod feemarket {
        #[cfg(feature = "feemarket+feemarket+v1")]
        pub mod v1 {
            include!("feemarket.feemarket.v1.rs");
        }
        pub mod module {
            #[cfg(feature = "feemarket+feemarket+module+v1")]
            pub mod v1 {
                include!("feemarket.feemarket.module.v1.rs");
            }
        }
    }
}
pub mod google {
    #[cfg(feature = "google+protobuf")]
    pub mod protobuf {
        #[cfg(feature = "google+protobuf+compiler")]
        pub mod compiler {
            include!("google.protobuf.compiler.rs");
        }
        include!("google.protobuf.rs");
    }
}
pub mod ibc {
    pub mod applications {
        pub mod fee {
            #[cfg(feature = "ibc+applications+fee+v1")]
            pub mod v1 {
                include!("ibc.applications.fee.v1.rs");
            }
        }
        pub mod interchain_accounts {
            #[cfg(feature = "ibc+applications+interchain_accounts+v1")]
            pub mod v1 {
                include!("ibc.applications.interchain_accounts.v1.rs");
            }
            pub mod controller {
                #[cfg(feature = "ibc+applications+interchain_accounts+controller+v1")]
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.controller.v1.rs");
                }
            }
            pub mod genesis {
                #[cfg(feature = "ibc+applications+interchain_accounts+genesis+v1")]
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.genesis.v1.rs");
                }
            }
            pub mod host {
                #[cfg(feature = "ibc+applications+interchain_accounts+host+v1")]
                pub mod v1 {
                    include!("ibc.applications.interchain_accounts.host.v1.rs");
                }
            }
        }
        pub mod transfer {
            #[cfg(feature = "ibc+applications+transfer+v1")]
            pub mod v1 {
                include!("ibc.applications.transfer.v1.rs");
            }
            #[cfg(feature = "ibc+applications+transfer+v2")]
            pub mod v2 {
                include!("ibc.applications.transfer.v2.rs");
            }
        }
    }
    pub mod core {
        pub mod channel {
            #[cfg(feature = "ibc+core+channel+v1")]
            pub mod v1 {
                include!("ibc.core.channel.v1.rs");
            }
        }
        pub mod client {
            #[cfg(feature = "ibc+core+client+v1")]
            pub mod v1 {
                include!("ibc.core.client.v1.rs");
            }
        }
        pub mod commitment {
            #[cfg(feature = "ibc+core+commitment+v1")]
            pub mod v1 {
                include!("ibc.core.commitment.v1.rs");
            }
        }
        pub mod connection {
            #[cfg(feature = "ibc+core+connection+v1")]
            pub mod v1 {
                include!("ibc.core.connection.v1.rs");
            }
        }
        pub mod types {
            #[cfg(feature = "ibc+core+types+v1")]
            pub mod v1 {
                include!("ibc.core.types.v1.rs");
            }
        }
    }
    pub mod lightclients {
        pub mod localhost {
            #[cfg(feature = "ibc+lightclients+localhost+v2")]
            pub mod v2 {
                include!("ibc.lightclients.localhost.v2.rs");
            }
        }
        pub mod solomachine {
            #[cfg(feature = "ibc+lightclients+solomachine+v2")]
            pub mod v2 {
                include!("ibc.lightclients.solomachine.v2.rs");
            }
            #[cfg(feature = "ibc+lightclients+solomachine+v3")]
            pub mod v3 {
                include!("ibc.lightclients.solomachine.v3.rs");
            }
        }
        pub mod tendermint {
            #[cfg(feature = "ibc+lightclients+tendermint+v1")]
            pub mod v1 {
                include!("ibc.lightclients.tendermint.v1.rs");
            }
        }
        pub mod wasm {
            #[cfg(feature = "ibc+lightclients+wasm+v1")]
            pub mod v1 {
                include!("ibc.lightclients.wasm.v1.rs");
            }
        }
    }
}
pub mod interchain_security {
    pub mod ccv {
        #[cfg(feature = "interchain_security+ccv+v1")]
        pub mod v1 {
            include!("interchain_security.ccv.v1.rs");
        }
        pub mod consumer {
            #[cfg(feature = "interchain_security+ccv+consumer+v1")]
            pub mod v1 {
                include!("interchain_security.ccv.consumer.v1.rs");
            }
        }
        pub mod provider {
            #[cfg(feature = "interchain_security+ccv+provider+v1")]
            pub mod v1 {
                include!("interchain_security.ccv.provider.v1.rs");
            }
        }
    }
}
pub mod osmosis {
    #[cfg(feature = "osmosis+concentratedliquidity")]
    pub mod concentratedliquidity {
        #[cfg(feature = "osmosis+concentratedliquidity+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.concentratedliquidity.v1beta1.rs");
        }
        include!("osmosis.concentratedliquidity.rs");
        pub mod poolmodel {
            pub mod concentrated {
                #[cfg(feature = "osmosis+concentratedliquidity+poolmodel+concentrated+v1beta1")]
                pub mod v1beta1 {
                    include!("osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.rs");
                }
            }
        }
    }
    #[cfg(feature = "osmosis+ibchooks")]
    pub mod ibchooks {
        include!("osmosis.ibchooks.rs");
    }
    #[cfg(feature = "osmosis+incentives")]
    pub mod incentives {
        include!("osmosis.incentives.rs");
    }
    #[cfg(feature = "osmosis+lockup")]
    pub mod lockup {
        include!("osmosis.lockup.rs");
    }
    #[cfg(feature = "osmosis+superfluid")]
    pub mod superfluid {
        #[cfg(feature = "osmosis+superfluid+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.superfluid.v1beta1.rs");
        }
        include!("osmosis.superfluid.rs");
    }
    pub mod accum {
        #[cfg(feature = "osmosis+accum+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.accum.v1beta1.rs");
        }
    }
    pub mod cosmwasmpool {
        #[cfg(feature = "osmosis+cosmwasmpool+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.cosmwasmpool.v1beta1.rs");
            pub mod model {
                #[cfg(feature = "osmosis+cosmwasmpool+v1beta1+model+v3")]
                pub mod v3 {
                    include!("osmosis.cosmwasmpool.v1beta1.model.v3.rs");
                }
            }
        }
    }
    pub mod downtimedetector {
        #[cfg(feature = "osmosis+downtimedetector+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.downtimedetector.v1beta1.rs");
        }
    }
    pub mod epochs {
        #[cfg(feature = "osmosis+epochs+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.epochs.v1beta1.rs");
        }
    }
    pub mod gamm {
        #[cfg(feature = "osmosis+gamm+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.gamm.v1beta1.rs");
        }
        #[cfg(feature = "osmosis+gamm+v2")]
        pub mod v2 {
            include!("osmosis.gamm.v2.rs");
        }
        pub mod poolmodels {
            pub mod balancer {
                #[cfg(feature = "osmosis+gamm+poolmodels+balancer+v1beta1")]
                pub mod v1beta1 {
                    include!("osmosis.gamm.poolmodels.balancer.v1beta1.rs");
                }
            }
            pub mod stableswap {
                #[cfg(feature = "osmosis+gamm+poolmodels+stableswap+v1beta1")]
                pub mod v1beta1 {
                    include!("osmosis.gamm.poolmodels.stableswap.v1beta1.rs");
                }
            }
        }
    }
    pub mod ibcratelimit {
        #[cfg(feature = "osmosis+ibcratelimit+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.ibcratelimit.v1beta1.rs");
        }
    }
    pub mod ingest {
        #[cfg(feature = "osmosis+ingest+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.ingest.v1beta1.rs");
        }
    }
    pub mod mint {
        #[cfg(feature = "osmosis+mint+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.mint.v1beta1.rs");
        }
    }
    pub mod poolincentives {
        #[cfg(feature = "osmosis+poolincentives+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.poolincentives.v1beta1.rs");
        }
    }
    pub mod poolmanager {
        #[cfg(feature = "osmosis+poolmanager+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.poolmanager.v1beta1.rs");
        }
        #[cfg(feature = "osmosis+poolmanager+v2")]
        pub mod v2 {
            include!("osmosis.poolmanager.v2.rs");
        }
    }
    pub mod protorev {
        #[cfg(feature = "osmosis+protorev+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.protorev.v1beta1.rs");
        }
    }
    pub mod smartaccount {
        #[cfg(feature = "osmosis+smartaccount+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.smartaccount.v1beta1.rs");
        }
    }
    pub mod store {
        #[cfg(feature = "osmosis+store+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.store.v1beta1.rs");
        }
    }
    pub mod tokenfactory {
        #[cfg(feature = "osmosis+tokenfactory+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.tokenfactory.v1beta1.rs");
        }
    }
    pub mod twap {
        #[cfg(feature = "osmosis+twap+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.twap.v1beta1.rs");
        }
    }
    pub mod txfees {
        #[cfg(feature = "osmosis+txfees+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.txfees.v1beta1.rs");
        }
    }
    pub mod valsetpref {
        #[cfg(feature = "osmosis+valsetpref+v1beta1")]
        pub mod v1beta1 {
            include!("osmosis.valsetpref.v1beta1.rs");
        }
    }
}
pub mod tendermint {
    #[cfg(feature = "tendermint+abci")]
    pub mod abci {
        include!("tendermint.abci.rs");
    }
    #[cfg(feature = "tendermint+blocksync")]
    pub mod blocksync {
        include!("tendermint.blocksync.rs");
    }
    #[cfg(feature = "tendermint+consensus")]
    pub mod consensus {
        include!("tendermint.consensus.rs");
    }
    #[cfg(feature = "tendermint+crypto")]
    pub mod crypto {
        include!("tendermint.crypto.rs");
    }
    #[cfg(feature = "tendermint+mempool")]
    pub mod mempool {
        include!("tendermint.mempool.rs");
    }
    #[cfg(feature = "tendermint+p2p")]
    pub mod p2p {
        include!("tendermint.p2p.rs");
    }
    #[cfg(feature = "tendermint+privval")]
    pub mod privval {
        include!("tendermint.privval.rs");
    }
    #[cfg(feature = "tendermint+state")]
    pub mod state {
        include!("tendermint.state.rs");
    }
    #[cfg(feature = "tendermint+statesync")]
    pub mod statesync {
        include!("tendermint.statesync.rs");
    }
    #[cfg(feature = "tendermint+store")]
    pub mod store {
        include!("tendermint.store.rs");
    }
    #[cfg(feature = "tendermint+types")]
    pub mod types {
        include!("tendermint.types.rs");
    }
    #[cfg(feature = "tendermint+version")]
    pub mod version {
        include!("tendermint.version.rs");
    }
    pub mod libs {
        #[cfg(feature = "tendermint+libs+bits")]
        pub mod bits {
            include!("tendermint.libs.bits.rs");
        }
    }
    pub mod services {
        pub mod block {
            #[cfg(feature = "tendermint+services+block+v1")]
            pub mod v1 {
                include!("tendermint.services.block.v1.rs");
            }
        }
        pub mod block_results {
            #[cfg(feature = "tendermint+services+block_results+v1")]
            pub mod v1 {
                include!("tendermint.services.block_results.v1.rs");
            }
        }
        pub mod pruning {
            #[cfg(feature = "tendermint+services+pruning+v1")]
            pub mod v1 {
                include!("tendermint.services.pruning.v1.rs");
            }
        }
        pub mod version {
            #[cfg(feature = "tendermint+services+version+v1")]
            pub mod v1 {
                include!("tendermint.services.version.v1.rs");
            }
        }
    }
}
pub mod union {
    pub mod galois {
        pub mod api {
            #[cfg(feature = "union+galois+api+v3")]
            pub mod v3 {
                include!("union.galois.api.v3.rs");
            }
        }
    }
    pub mod ibc {
        pub mod lightclients {
            pub mod cometbls {
                #[cfg(feature = "union+ibc+lightclients+cometbls+v1")]
                pub mod v1 {
                    include!("union.ibc.lightclients.cometbls.v1.rs");
                }
            }
        }
    }
    pub mod ics23 {
        #[cfg(feature = "union+ics23+v1")]
        pub mod v1 {
            include!("union.ics23.v1.rs");
        }
    }
    pub mod staking {
        #[cfg(feature = "union+staking+v1")]
        pub mod v1 {
            include!("union.staking.v1.rs");
        }
    }
}
