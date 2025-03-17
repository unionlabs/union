use unionlabs::ibc::core::client::height::Height;

use crate::{AccountProof, LightClientUpdate};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    /// The currently trusted height of the light client to apply this update against.
    pub trusted_height: Height,

    /// The actual update data to be applied.
    pub consensus_update: LightClientUpdate,

    /// Proof of the IBC handler contract against the execution state root provided in `consensus_update`.
    pub ibc_account_proof: AccountProof,
}

#[cfg(test)]
mod tests {
    use beacon_api_types::{
        altair::{SyncAggregate, SyncCommittee},
        deneb::ExecutionPayloadHeader,
        phase0::BeaconBlockHeader,
        slot::Slot,
    };
    use ethereum_sync_protocol_types::LightClientHeader;
    use unionlabs::{
        encoding::{Bincode, DecodeAs, Json},
        primitives::{H160, H256, H384, H768, U256},
        test_utils::assert_codec_iso,
    };

    use super::*;
    use crate::{LightClientUpdateData, SyncCommitteePeriodChangeUpdate};

    fn mk_header() -> Header {
        Header {
            trusted_height: Height::new(123),
            consensus_update: LightClientUpdate::SyncCommitteePeriodChange(Box::new(
                SyncCommitteePeriodChangeUpdate {
                    next_sync_committee: SyncCommittee {
                        pubkeys: vec![H384::new([0xAA; 48])],
                        aggregate_pubkey: H384::new([0xAA; 48]),
                    },
                    next_sync_committee_branch: vec![H256::new([0xAA; 32]); 5],
                    update_data: LightClientUpdateData {
                        attested_header: LightClientHeader {
                            beacon: BeaconBlockHeader {
                                slot: Slot::new(123),
                                proposer_index: 456,
                                parent_root: H256::new([0xAA; 32]),
                                state_root: H256::new([0xBB; 32]),
                                body_root: H256::new([0xCC; 32]),
                            },
                            execution: ExecutionPayloadHeader {
                                parent_hash: H256::new([0xAA; 32]),
                                fee_recipient: H160::new([0xAA; 20]),
                                state_root: H256::new([0xAA; 32]),
                                receipts_root: H256::new([0xAA; 32]),
                                logs_bloom: b"bloom".into(),
                                prev_randao: H256::new([0xAA; 32]),
                                block_number: 69,
                                gas_limit: 1_987_654_321,
                                gas_used: 987_654_321,
                                timestamp: 123_456_789,
                                extra_data: b"extra".into(),
                                base_fee_per_gas: U256::from(1u64),
                                block_hash: H256::new([0xAA; 32]),
                                transactions_root: H256::new([0xAA; 32]),
                                withdrawals_root: H256::new([0xAA; 32]),
                                blob_gas_used: 100,
                                excess_blob_gas: 100,
                            },
                            execution_branch: vec![H256::new([0xAA; 32]); 4],
                        },
                        finalized_header: LightClientHeader {
                            beacon: BeaconBlockHeader {
                                slot: Slot::new(123),
                                proposer_index: 456,
                                parent_root: H256::new([0xAA; 32]),
                                state_root: H256::new([0xBB; 32]),
                                body_root: H256::new([0xCC; 32]),
                            },
                            execution: ExecutionPayloadHeader {
                                parent_hash: H256::new([0xAA; 32]),
                                fee_recipient: H160::new([0xAA; 20]),
                                state_root: H256::new([0xAA; 32]),
                                receipts_root: H256::new([0xAA; 32]),
                                logs_bloom: b"bloom".into(),
                                prev_randao: H256::new([0xAA; 32]),
                                block_number: 69,
                                gas_limit: 1_987_654_321,
                                gas_used: 987_654_321,
                                timestamp: 123_456_789,
                                extra_data: b"extra".into(),
                                base_fee_per_gas: U256::from(1u64),
                                block_hash: H256::new([0xAA; 32]),
                                transactions_root: H256::new([0xAA; 32]),
                                withdrawals_root: H256::new([0xAA; 32]),
                                blob_gas_used: 100,
                                excess_blob_gas: 100,
                            },
                            execution_branch: vec![H256::new([0xAA; 32]); 4],
                        },
                        finality_branch: vec![H256::new([0xAA; 32]); 6],
                        sync_aggregate: SyncAggregate {
                            sync_committee_bits: [1, 2, 3].to_vec(),
                            sync_committee_signature: H768::new([0xAA; 96]),
                        },
                        signature_slot: Slot::new(123),
                    },
                },
            )),
            ibc_account_proof: AccountProof {
                storage_root: H256::new([0xAA; 32]),
                proof: vec![b"ooga".into(), b"booga".into()],
            },
        }
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_header());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_header());
    }

    #[test]
    fn bincode_decode() {
        let bz = alloy::hex!("008001000000000000000000002000000000000000894798d09babc765b3ba22473d820465e713c1d7f78f3eaeade3d957bd412a742f498a9b91e55cba8e08c36c8ad4788fa1c76af1545d7901214bb6be06be5d9e458f8e989c19373a920f0018327c83982f6a2ac138260b8def732cb366411ddcab8d3a9bcc160e518fac0756d3e192c74789588ed4a2b1debf0c78f78479ca8edb05b12ce21103076df6af4eb8756ff98a298ee1ac0466ecaa04d5798048c6e192409af63217f32fd7e07794cfcdcd8deca055b9782dd1ad45a578a9ec10606c9698d9519a02b64f230e5a2520401799c2ca7d69ab23a6d9817943147264bf00d409264b928718245efff4f7ee97dd5c903e2989e7442ee0a8958d020507a8bd985d3974f5e8273093be00db3935f0500e141b252bd09e3728892c7a8443863cae4d49364e4a36760cc74a675500055b9aed99bc19d31abb953ea156bb5a76dcf36769d15341b850114a30ffc805778098f620aadc4e58392b5b583fed96c452b54c39ba3a9fe8c277f625fae7e1317d034f732995fd88c1461463edd0f2b86da34febc12af07316580b480364f90a76313ccce7927bbe263e27ea270853b02ad4d1428caf55363f3ebebac622cb9fd6a8be337b3d0e6be415dcb037b246831f9966aacef62b69d6b609e4ff8208bc536c6473bc9fe9e3bec9a8665c8caa05c5a48cc260df1df875176cb17493a5b53d669c091da74d5075acb8952a641b1b7ef68d01f009c1a365d2fa80937c79dd6b96ef954b331a534199f4f113d993a50ec7a781fc5aa2a181ea0bdbfd4c5c557abfebfcc02604d5aef52ba64afbe0ff18afc0fa2ed6a270de6122a19d4600380b7f9b5e974d16f095f1702f55792ecab0128b155a69f17ad64a6de0a7063642ec8dde8306920812b32def3b663f7c540b49180345d3bcb8d3770790b7dc80030ebc06497feebd1bcf017d918f00bfa88f974b2aed17665e51c1c091998ca9649875330947de3d2733a5bd2eda69b0c593cdac2e416993a87f9a17aec1ccdc236892a93728c252a45ef587ca53a037593912599d82e2b8aa1b734b99d500a0ac8c142092ea8b3c2c34a28dc8ddf337a249a98ed496c2f464226500a6ce04602ff9ef133ed6316f372f6c744aee165149f7e578b12780e0eacec307ae6907351d99aca5e4979f281b5ab0ea0f549d6dcc34989607c335e94efedeffc7e73b393f42c7b11d76144a750f82600b21d10b6777afe6eface52fb6de91055a81abf9aa6e42ce2ef36fd8ae0d09aec6e5d8bd40a065dfccda6104af94df3f7a5854559ef4b363a57c600a0037d54d738037358aa686e27da3ea65be95f95fc04d5736fba6338c5d544c3cf2b11262bd20e7a42dd1b41a0d9f8f19be13395aa09711b492d20eaf4a56d2360cd6daa2fd665532d852cb9224a5a39e5abff389882f961f12a6a3177a98f653cea646f525f0f13348efb27e0d3d0cd824704c91d8d959096d259c9e577298f444acc629920c9619be5096746aaba64dc87835ba709332f4d5d7837ada092b439c49d251aecf92aab5dc132e917bf6f59799bc093f976a7bc021ab0bdda0f85f842f431beaccf1250bf1fd7ba51b4100fd64364b6401fda85bb0069b3e715b58819684e7fc0b10a72a34b5bb0162a4f27d1bab4c7dc3d20f5a75d6ee98c56bcd309a1f0f307685ad47ffb8a35bfdf8431b9b954b59662a74c478a626de0451397075bf145e720691c9d5ed92eddf1f4e48155b455aac7a8e920d042f5635c7a74fe3a9175ffbfb7ce12e9243ef5ed3bd28892d1ef4f7aaf29faeb9c0e725673cd38e308bd756f20a9ee09de5cd9822e5e77bd03b734ef8a9269583802cd575a3cea7e3e38fc1a73d94a9e4fdb999b8494e7929309c009d79a23edb1ba091ac02588f130e0585fb10654096c8d3dd08724624017f178393d176b425dab9dfa1cc3f62c7669337446baa601e0aa261c00c76bde07ba9a1a3582c0a93bb1c86717fa7303f65cb8c45c9fcc8fecb88428b7cd1dd59967a132109c25ab5c97888e46c5d471ff911c573f45a3493c65ba88f12ad22c761003cef7ffb155b9b17134ed871c0703fac60e80dbd2dd8d163bd28eba9dff88b1e9bd1ae4a76876dd4705157eb66dc71bc2e07fb151ea53e1a62a0bb980a7ce72d15f58944a8a3752d754f52f4a60dbfc7b18169f2688bfd46db38ea23a0659a66a5bf06b883c58c9742eac24b6f6a4941d833bf6257abde574c8eb4f6077d281e865281ac02050000000000000098c21f0a54cd3b1fe58297b6bc5f49bc581a7fd5bedb78aa532a7292aae9fd1c9224be291ec24f71c8be63f535767e84c4af1a4436b004066bfd6c14a98d0c0ac45dcc91242596c687e00112018479e6ae9abeb631c053d28408801d7256c8529b1133fbcc75f6585fa0ca86d7e7218abf9246060885fd04be3a8f28bc8bcbea35d1f03b1d04118f95a051d1dd0fc5814e962e0378d728e49c3a702f52872ea9d0010000000000007200000000000000a69b16b275db38a5f5397df8877c39aed0d004c3d9777c7200269ff2a476a8759272648b58dd55e98e451afdd4198a1d02312287af798265323f942995da381e57fe0d4067a6b0305050ddc9b86950a24b067de17e1104207ef44807e1e1d5ab127a251281bcacc34c66732ae1df56dcffe43c7cf4da0a3160638c39230602eb00000000000000000000000000000000000000007e475561e8c09ade90b7c186e8ecb8838074bef371dc26b5bcd29209f77f75a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4210001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000007e565eeb13d4525c8a481162eca8630790e6407b770fc05ffc60ce48085c8e75d00100000000000080c3c901000000000000000000000000d441d867000000001900000000000000d883010d0c846765746888676f312e32302e34856c696e757807000000000000000000000000000000000000000000000000000000000000007c3133898211094ba2acbbb765e542f64389b4efa919370ec723a0e518f3be327ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede128ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec3000000000000000000000000000000000040000000000000058d29e3c3ba766fa2a51f99906c816d39600e095403a63fccf572c8f9ce9c7da6c6dd63656639d153a2e86a9cab291e7a26e957ad635fec872d2836e92340c23db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d7147d78a836e566a864c1bd2873219a87326cddc7a37b2115c3a4f38970d2a78fec0010000000000006800000000000000c3e1378bef28748470278fe7b18a68081fa38f54649bd870ff9115c142ba17158859a5577917c71138c0e8509e6372ab5a81612571e8e67430c04a478d1ecc203cd2b347f78b499956055e6d803a25ec5a7cc55b8bd4e60017be1cd1bfc9eff09dd75572fa3106202e5eae70c08a37e561e5385ad2ce5d3439b12d4f2f329e8000000000000000000000000000000000000000007e475561e8c09ade90b7c186e8ecb8838074bef371dc26b5bcd29209f77f75a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000817446d78cf8f1dd8c5b417aa841d20d5427230e7f4eb55567bbd8cbd38d323fc00100000000000080c3c9010000000000000000000000007441d867000000001900000000000000d883010d0c846765746888676f312e32302e34856c696e75780700000000000000000000000000000000000000000000000000000000000000bd7d5fba54fea40bda158c3f61385a61d3a004364c93c21f8a476b9b239cda707ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede128ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30000000000000000000000000000000000400000000000000d3b76986734005a571fb53ac587897f91c681ea005d84e34ad34721ac240f6526c6dd63656639d153a2e86a9cab291e7a26e957ad635fec872d2836e92340c23db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d7183a9d69c69b276b0de01e06798968edecd836998c7c98f656cb709da01c16d0f06000000000000003800000000000000000000000000000000000000000000000000000000000000974ea8bd2b7b773225885c2a945143947ce814ad673925e69cd7f548b5dda2b2a3998c8d9940a69b92c833af218718cf185833ae355c5d0cf9d22b19269ea53fc45dcc91242596c687e00112018479e6ae9abeb631c053d28408801d7256c8529b1133fbcc75f6585fa0ca86d7e7218abf9246060885fd04be3a8f28bc8bcbea35d1f03b1d04118f95a051d1dd0fc5814e962e0378d728e49c3a702f52872ea90400000000000000ffffffff8b10eaed887e6f11754ac43a7bcd9989cff811cc4a1a832587f08799d5442f224fa4a66477dded44b4c7877208df6e3712f0a79aa955165ca1d2cf45939e6e21c55ce4456036b4715db28cf75d392f2f88669672d8682c5423b9e382216a8e57d101000000000000d5b2470daf6391f53de1169928f0a7b064fea6270a7ccca821dac5546f851d8d03000000000000001402000000000000f90211a02e8a5ae0114f9e87cfce7f748e78b30c3c8e657afa573873d764390965f9bbeea027cce7ec5bd0d64c3c4c43a34ac8a0ebdf61044c1c8e9b3765804ef98c12b606a0d0b58b83a80d7ef8b34c2b7259e636f0f52ad2127dc1821c090a558140a61c3aa0474d34441aae8ca966ed5a4d4b951ab060072e97d46b7013809286d65c7e02d4a0910512d1a5b283f6ca4937fdb7e6aa426e9c5c3e8e3b971bbea89859dcb33522a0df39ce4adc1522bdbdb6800a1670962e46f498e34b9f040b973e341bd8431a19a081aa05ba6e956d19ce38943727a944107a5b96008bb0ec4a9c1cd06f5d3909a9a04e08a6f1dff4965e506b19bd874f4717db1a0b8015dae5124504a0db09706e06a0bb3f1d6940b6cb261ebebc7b9581f3ce2caa33d7553b307301cf678ff0608971a080cab207fb3d07baa1bb8aa398e4cb97d38fac06994e85d22aa34d04f6fb4e0ca0ac84d3d9b4cc7e06aba002987e0b23f396ee325bc86f6e8c4739c10c5dd2958aa0c52469f342b22fb92b7e099d038fe9bda84958a2508bdb093d85ad1073e72c0da04f1967913b504d0abb84ae3e381fe8273536715e4985c17a7b398d533706a704a0952ab47e47bbe7482300a31485d6607ad5aa479851c7cfcd37f78ebe1f6cf9e0a0213ebe73c28294598d6317486f3cdecd5409542531cf7dfbba30cc1710dda25fa06c33db1fe2e851acf97b60f00abe5ce3d8a4c17735b9ef8106b8d73aadcf78d980b300000000000000f8b180a0de5d0f8d14997487a666b6a3c137e46422e0c327d2bf20c87467ad7c5f5ea02580a08922c62e1ca0589385a71c9431a675d6bafa4ff034c6284aba6db89e8cf3e0e68080a0557a3dd917abc049f5cbf3c308799f78332ce7058a75598e6f21fe0498c5e59e8080a00a0e576564c351990f2393de596a429df581ac49557a4ff88f7291c5ccd9d5c280a0e5a89c8e3f42bcc1d5453161697e15f39746ee4cec65367a7f483ca902b892e880808080806b00000000000000f869a020fa8211e9294f5039596478f2268d90b881d2de706d04dcf3e2e1a87f451f51b846f8440180a0d5b2470daf6391f53de1169928f0a7b064fea6270a7ccca821dac5546f851d8da0f1da0a71fdc1bdb5a344cb66eb8a98a3fa0aef81853ca761b387738e4c1899a4");

        let header = Header::decode_as::<Bincode>(&bz).unwrap();

        dbg!(&header);
    }
}
