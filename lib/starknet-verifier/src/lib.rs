use starknet_light_client_types::{ClientStateV1, Header};
use starknet_storage_verifier::{Membership, PedersenHash};
use unionlabs_primitives::{H256, U256};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("error verifying l1 gps statement verifier account proof")]
    L1GpsStatementVerifierAccountProof(evm_storage_verifier::error::Error),
    #[error("error verifying l1 block hash proof")]
    L1BlockHashProof(evm_storage_verifier::error::Error),
    #[error("error verifying l2 ibc account proof")]
    L2IbcAccountProof(starknet_storage_verifier::Error),
    #[error("expected a membership proof, but found a non-membership proof")]
    ExpectedMembershipProof,
}

/// 1. Verify the L2 block hash in the L1
/// 2. Verify the L2 IBC account in the L2 contracts root
pub fn verify_header(
    client_state: &ClientStateV1,
    header: &Header,
    l1_state_root: H256,
) -> Result<(), Error> {
    // 1.
    evm_storage_verifier::verify_account_storage_root(
        l1_state_root,
        &client_state.l1_contract_address,
        &header.l1_contract_account_proof.proof,
        &header.l1_contract_account_proof.storage_root,
    )
    .map_err(Error::L1GpsStatementVerifierAccountProof)?;

    evm_storage_verifier::verify_storage_proof(
        header.l1_contract_account_proof.storage_root,
        L2_BLOCK_HASH_SLOT,
        U256::from_be_bytes(header.l2_block.hash().to_be_bytes()),
        &header.l1_block_hash_proof,
    )
    .map_err(Error::L1BlockHashProof)?;

    // 2.
    let result = starknet_storage_verifier::verify_proof::<PedersenHash>(
        header.l2_block.contracts_trie_root,
        client_state.ibc_contract_address,
        header.l2_ibc_contract_proof.contract_leaf_data.hash(),
        &header.l2_ibc_contract_proof.nodes,
    )
    .map_err(Error::L2IbcAccountProof)?;

    match result {
        Membership::Membership => {}
        Membership::NonMembership => return Err(Error::ExpectedMembershipProof),
    }

    Ok(())
}

/// ```solidity
/// string internal constant STATE_STRUCT_TAG = "STARKNET_1.0_INIT_STARKNET_STATE_STRUCT";
/// ```
pub const STATE_STRUCT_TAG: &str = "STARKNET_1.0_INIT_STARKNET_STATE_STRUCT";

/// Slot of the `StarknetState.State.blockHash` value in storage.
// TODO: Once we have U256::from_be_bytes as a const fn, use that instead of this monstrosity
pub const L2_BLOCK_HASH_SLOT: U256 = {
    let mut hash = keccak_const::Keccak256::new()
        .update(STATE_STRUCT_TAG.as_bytes())
        .finalize();

    hash.reverse();

    let mut limbs = unsafe { core::mem::transmute::<[u8; 32], [u64; 4]>(hash) };

    limbs[0] += 2;

    U256::from_limbs(limbs)
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use ibc_union_spec::ClientId;
    use starknet_light_client_types::header::{AccountProof, ContractProof, L1DaMode, L2Block};
    use starknet_types::{Felt, MerkleNode};
    use unionlabs_primitives::H160;

    use super::*;

    #[test]
    fn l2_block_hash_slot() {
        assert_eq!(
            <H256>::new(L2_BLOCK_HASH_SLOT.to_be_bytes()),
            <H256>::new(hex!(
                "71a8ef1b1265359d77973c3524afac225c0a0d829a0d4da5cac3b34532019fee"
            ))
        );
    }

    #[test]
    fn verify_header_works() {
        let felt = |s| Felt::from_hex(s).unwrap();

        let l2_block = L2Block {
            block_number: 4174049,
            parent_block_hash: felt(
                "0x6de8c5b146b0127e0a7842ffa97931850339016cc6fa2083f8817a8fef3260e",
            ),
            classes_trie_root: felt(
                "0x5e4ce3094b99bc894ff49cb80319f494384158834b69fc8b396f28b65386d49",
            ),
            contracts_trie_root: felt(
                "0x8eb9daf1010e9400d1549fe06ae61e8bb5b52a85b784a26e4353294b69db0f",
            ),
            sequencer_address: felt(
                "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8",
            ),
            block_timestamp: 1765453275,
            transaction_count: 6,
            events_count: 4 + 4 + 11 + 5 + 7 + 5,
            state_diff_length: 39,
            state_diff_commitment: felt(
                "0x700341e929df7d4cd196e97bfe84b13bc65a981d93229dfe249b2ab80a9be37",
            ),
            transactions_commitment: felt(
                "0x7646130360d27c6e5de4d09f5e8141301a20aa66fc4c8c17189be631466a264",
            ),
            events_commitment: felt(
                "0x31571f5901e6081cb7b10c64b44df4407f56b4de12c98d98aba7854c4f99cd4",
            ),
            receipts_commitment: felt(
                "0x7a1f73a67ca072a6d7e61f80252ab5c1e4ec3fd916398b26e11fc92ce55a1e2",
            ),
            l1_gas_price: (0x42f7e3a4, 0x1eafaf65743a),
            l1_data_gas_price: (0xef9bf, 0x6dcb3ada5),
            l2_gas_price: (0x410a0, 0x1dcd65000),
            l1_da_mode: L1DaMode::Blob,
            protocol_version: "0.14.1".to_owned(),
        };

        let nodes = [
            MerkleNode::BinaryNode {
                left: felt("0x305ab7d618a9e856a5b187d809a7f306cdf3f3d33d36bc2c2968aeab533cda2"),
                right: felt("0x6d1edbfa23c5bf679f08ab9d54d8dad98b628487edd1f5cc5fcea524d90b4f7"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x33604196e53cc124af3f050167854bb1868fa495ef66c0c90fb2a5fcda2c83c"),
                right: felt("0x3de8920b04a078cb1f4b9a28446fc23ca1b54fa9515e4d9824a47603fedd4a5"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x101ce766e32315b6cc47883ecee2b5a9fd4c7557958aedff4f329d180220d3a"),
                right: felt("0xebe1287fb5a25ac632a53fb9e54ce256fcd7533133c4ff0cd70886595d99b7"),
            },
            MerkleNode::BinaryNode {
                left: felt("0xb6b1c5d7dc2c596942b2a17810156a4d08bc11d448645840d0ab637ae38484"),
                right: felt("0x4c76037107f82eba1bd6d9993971f16eff3468b64ee4e9b17d37d1527040896"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x533017e434b033689f12b06fb9e855ed00c78a8cdfddc903432cee60f849366"),
                right: felt("0x6dc53ffe318ab307ae6354335ce35be76530599f7ca3a8ce6fc18dc9d8de74e"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x4257e8c08a26ee33a64622cffff95a83932ac81246c3ffbb2d8361aaae2d2b1"),
                right: felt("0x6e0f0db09122b2b46deb5fd2269b4587947b937cf14874bbac7c15883ca3bd8"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x70a94c7a5666b3d2148a6bdee5469081798b1adad7c3c4ef8c25c891503a1a6"),
                right: felt("0x36584e72c2006e479a87af2ee971d4ae9c7f756b81627105c6f7412bd0db448"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x5609ed4b81a781af59aefbaf205b61576a29911a7c9a3fa1c1710690d5867f"),
                right: felt("0x2da53eab882376d8f48b9e4be19cc7c49b467164f541afe395622938fc94a03"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x179869a71950d867e9cfdcaad742c46bd7da32a48ed2628679c5ef35dec8fdd"),
                right: felt("0x733f7a7ba94681b6622f0c65451bf9233f97d993b28753bd87febfadffae489"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x7fbba0a33569ad039875cae97dae56ea99c713a63ddfa1537e2e514b776d9e1"),
                right: felt("0x104e502581db03927dc9db020f64b599a4baf4cb8da191d55eaabf6a317283d"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x6d840221616a26a92acb112a1582a85644cbb35d7a3cf603343be089ed00ec9"),
                right: felt("0x44e1a78d7acd1bbd66e604288faf9a07bed58964ec6f465c7c5209cd7e8ea92"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x5eb2d805c3d7d34584572f7b77419fc815139a52a680af56d05bdc738afef18"),
                right: felt("0x1811d45d4d14a44857f3a14841f14139acfbc49d08a612acbbebd2563ece92f"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x6c613354260a00fb7931522f4194ae69159cab032b0299c8c6c7cf328a5617f"),
                right: felt("0xaa035333b79d9800b66150936daddda15d98b0dc34ab212e09da12386db907"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x506ced2ba38897f4cf279202baf072b5973dc31bc53e66344157b6700f17d3c"),
                right: felt("0x4fca360db940e4e47e640e336723bd3bd78934ad2af520f15f3e0b52166f5fb"),
            },
            MerkleNode::EdgeNode {
                child: felt("0x4b6a88ec5cc90586fd5c285ae20224503306e5b304c5ddb26e643a53add589b"),
                length: 1,
                path: felt("0x0"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x5e29544c3c9d6c4c972e5ff910354e82a0e7ebe890823784f813c9875a042ce"),
                right: felt("0x7866ca74f720092fa6b16c9a9f2c5618c89efa35a29ccc1b472cdafb94d75cb"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x1eb10eac884e2d2bde5aaa6bac9adce4ba54f0961a4c00c21676fba0d8852fb"),
                right: felt("0xbce16eef6886a0ae81de15690d53085d74e35d4978305b1089c17fd6a4597d"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x5c9a95965d8c446e9bc5e968d33ce077d1e705c0fcdefb4f836254843f63c21"),
                right: felt("0x6cbbf2e225b06c542d324a9c3a55d792a5c95cb98543bfdd9d086e227f015d9"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x4f492e971e246ddbd1c3b0b1357eedb37bf2607c9525f978aa030e22319d9c5"),
                right: felt("0x2d0f3a2b0e9fd31188cebe84634636ac6691b03f6174a3c4883b3bc3b041762"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x3b8af7a5e04753f361e87b29efc2a381cbb2385db4a23cd0ca369a8836d5046"),
                right: felt("0x6c900eb816f9c72490ed8f4e5d212e4a97f5f455b499d1108073a4fa00e66ad"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x6c8e798c2f31a2da9904fb6ed1b8c818daa58fde22811a4b80b6593d285877"),
                right: felt("0xb92e98647c0c0bdb3712940643eeb11ca3918a04f7bd9abf2545c9a2fbd340"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x6bb29dd06cfca37492631004ea10b670b6ac104f2ccbe40e0889c790ff8faa3"),
                right: felt("0x4b2e6799b800af6b60ee81abb0f27c9fd42bb728b7ad60cdf56b3be4ddc5f4b"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x5595953da4821e203284a77330b290d121340f93ec829334aacf46e968af880"),
                right: felt("0x50337184a321e3b087bef0290a8749f51a1bde4b07208efbd37cf1bb875cf96"),
            },
            MerkleNode::EdgeNode {
                child: felt("0x7ab8b6072e9d0957760c6d309c5ae7f27edaa5a85d1a05ee2ce1383872970ca"),
                length: 226,
                path: felt("0x32c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x14d8b2f2795217c6ee4ac6a8e3fb9bde9baa4680411ec63f73b887f040fd50e"),
                right: felt("0x8eeb881ead580d16eebc8b3ecebc3816eb4bab2573442668242eb9fd73ac57"),
            },
            MerkleNode::BinaryNode {
                left: felt("0x7beb3f225b68f50c2891b2689a0ee01d2adb69e97f486a6970c803a77531b95"),
                right: felt("0x2a50d7e63c0dfe7e90ef2409e0e2cd062d8912b9f58fe9bc72bf1d94e6036ab"),
            },
        ]
        .to_vec();

        let header = Header {
            l1_height: 23990413,
            l1_contract_account_proof: AccountProof {
                storage_root: hex!("5091e8afb812f9c63db3d1e54a604d85b57e686edf3086a611ae9974d94abd12").into(),
                proof: [
                    hex!("f90211a03e595771950b867b55ab9fef4b36f1564b8a4c1ce18830e9f926223c6aa3da21a0bd1afcd3c17f33c7e740adb9261a757fa0dfd8a3a862be0b9f0d870a74eacfdda04da3fa652bf7653a415a611e06ef867c71888b8c37f63c08bfbb9af92fd52b27a01471afe3c98aa82ef03c28f7cc2151ec609fce184391dd813f4c70ec0b3eb9a2a04ea9c9744ba0d4344efd3fa534f79c23f6aec3648942d8925193f3cd66bb2e76a006ed4779d6efc7cd91de110b263a129609b21eb558957c270f372d778f5227dca0dbc0435fa8265d476ad21cd2e58f8a8fa98aab2b45ac8977d17a1bfa2d88568da072b072a897ef4020dd7424f3fd104b316aecea9e293f489a86ae133c5fcdfc72a0304c9fc95e4f723681cdea067ae515e7cdafa2b232bc743ba11fea7f1d4d38b4a0afa2dde73b74db55bcfe7cb19519a7c0b4c6d14ba2fe90680775f22f7a52caeaa00a0ed18f1c3e332f935054aa99e4697b9bdd0da1fb87c7395d27e2f12f872995a094be62452a47f226bae6f4427f5938dfb22fb22412c11eed2c0f9ac1e7eeb678a0fab94bc0bd582753fa4a1ba054a02f8dba4cd1aabcc64c6c9caebebec5713950a0ef45fbeda7ec6a6105602b559a0fbdbaec82a266ba9cf522c525858575eff739a0308624acd791a027048a79937fcbf6d80b59a78b84778a506a5fa250b88565bfa0761ace2d061c764cf1f84c26e072a670846ccdb82cfb687ec1b0907029dcb7ab80").into(),
                    hex!("f90211a0449b65ba65298468536d892ff17f1cc3d68023d42d7c35b5b07ab60bb0905cbba0197eef034779b1081a12dee6a7992957842b2947452d347f93b13dbee525cd47a0ce11c2fb70a19215d329923c5489b94e9a8719dff730105a7185012f1e5475b6a05a14b21bbbe5125c458af8f8482453825de58f8db8d4ac3a2f9399d206fad8b6a07ab1100d1b227d4fbff4e6256209cf7ab5e5a1939486ea9adf75e451be3404b3a0d38b5356a173e938c17c1414303f4ea503868a7762c0476a76b66022dea3dbefa07388b616e6bf4bb3e57b3b814b18c2fff113400117305bc2972e4c9f4e53859aa0f47dd6321ac87d299ba924279d9922f5359551d54314ffe5d954637bcece2666a0f5f93b2d2eec85f32cf830042eb34065e7a505c91450f6b409d8fa4fabf0331ca08ca18aef4b9af22ae05b1a5428c6b180926a1f93c66bda9619c8b02506f8cd62a067319affbe5c772b3e55e9a8f4e5bc13547212d8a9d4efdc0d6a6ece9a60c5dea0af7d92ddd8b2d2ee6a2d4dcc47930a99f2027967487778602765e45996caaf7ea0c9b89c1adfc4b508b165cdc5d266a580e4f221ecb5031101d25d1a8a38f49e35a085e2cad7ddab7e0d070f7207769c7a062e59407379a428e6f39d78dd3cf7d06aa0c9ed6a341e148984794f703e4c55225083343172c9cf380b74dde012285f2923a03d5da7c4393d57a45a4d949d96ee75d80f6ea6337cd7d2584bd3d92015747c4780").into(),
                    hex!("f90211a0ee5104fdf2a8b282f7f494a119521928ec2d179b70d3a8cbbbdb3aefd5a147b0a073eca6df5b3b16abb8184d61140af61779f261bd39f91387d6c37889e8717df8a0a5fa40e04b5594149f464ea303608203e980ad75602afd1c1a2435f0644dd986a0710d328bf62d1bb633242a127dd7ed191f7f823d65df75ed529d65d68510f391a0f876c200e95114d4af0eb803ddf97760b3ea9126fc89e180a11fcf5a669eb42fa03cfc9f70295ac53f21a1196abfe7fddf5d702fcb874787bab887fc06a61b1ae5a067ec87474a3a8e842bafeb2c6f3ea0389fa37ed02981401fa7e20e62861aa04ba056819e4d71e68627ea8b07f45695b7a389fe5f93dfca970abba0584d73bb1e58a076585e5ed18c5b1b3f5a458a23c4947ebc2cc31441fb7365ff82d7fae6c890d8a00240b6b07d0f8750d9291124eb1e87cd1828c6aa9f1be38e96de5e2133aa2453a0bd8e613eb8a9c27a851cf687d1ec5d7a5cec89bef103bebff65f1669a49e2231a0f6eda6f0fb6aa954b0bdbd3fb746dd3403c3b0964ae17206abf0bd0ddea4ce8aa0b6af5e19cd1274130fd7dfe98a9a1d8890d3167b909e8523ec6aa6eaeb85a1a6a0409d685159a04dad449c61a2bbabc1d666362ca3280959d5302b7bc62b226192a041339e3b4a77ebffd3e24235e19c676a2766d5974032a0e49c363553d82d1f41a05c8a1c7ad156e8873d874e587c2307258e31f593f1761eb5bec02219469a511880").into(),
                    hex!("f90211a0ee459789c04f180799bcaf0460d3ea08d03d6046c8c5f4cf1e95033cde484a61a044cff7f31d5f8827bbcb10a2b2091f351de83b1ca058bdb318f2c958e79a154da06771a9490f4a868cc5aafebef6e6425acc2d5210bf3696f0c19c97d0120ba3b4a0a2cc3f7f8fbbfbaa16e75c6293e136d64de868cc5460b6893dd275f439b7613aa07796e5aff768e052d3fc9eba311a3285e061fe6b756bd0690bc041975af3799da05a2e52647cac3f85ddc63e75e55f01ab52b606f9831106da141330dba4094503a014fe992fc9619b545c8bc99cb28cc78301c5d2215447d756e32ff96ff79d65daa0308ca28d6eae2df2329ead707fff9eddd585ececdf975e5c5ba2c35134ad7905a0c054d1f7ef28cc154042f17618e5c16f73b0b55ee63168d495ac82fd2fb12390a02fe2381a7c2861c70dc446cb04f18ae1bb4ed08e3a021227c2edad2ba8e08726a0f62a728ebff2fcc922f022093c2d8d89c1d7fff07ac8a415229e6e18bb1edc9fa0a46375aeec7da0ab831ca221512c3374ece7cdcd3fc3cee4ab42512ffdab1a58a082045f1daf961809941f39020e54d300e6e3e3e35f0373fd7f8a54e97fa6c872a029afeafcfad7e34f920e9021a31bea99dcb84f550e7ae8500415af459e1e1218a06d7714c3e6dc3d110eb2f5ccba8e012e797d813ebeeb59f549fcbe7a4662929aa0245c1f2076eb39161cdfb79ad5f601bd69fe3782aa72224b656aef33aab57d4880").into(),
                    hex!("f90211a0bc290355b1444639bdb8373b4db36f02eb8e70e40de8b598b52c7e44fec5b5a8a07f100776f18df1230187ca91853d725dbf10a1cedd0660c8465e40a5175e4147a0ec76d99914b03c81fd1e211bcb33ad7234d9b6e5dde334a0d1419b72a03543c4a0153ad31617fa1ea6ed68c348f8ccb1a05155555bca3af86ac8f2ed166abfc62ba05ade8da365cd1507a0b243e0aafa1f070e5e03446550a36b43bda4c5298ed517a030f544886c963f5ded3fd5f30b7c6a9848973bda42f88085fbfa17b13d35c899a05f4140ff406952a0e0b4bfc1cee653436353d903c4a7293423632909d065d9d5a08e5f82a20b88e4b58ef9a9478213907ba783de0d9ea628c2b6931e4c30840cd8a0b6d5d6bb04f97902a7302fc19633c9f2a55959c65c1be57b62e0f9e95e8ec3f9a0e7b2868ba40a93f36c6734b150472066bba7699a17bb1221eb8f0e59e19448e4a094e7b1543b602c8a1ed5d176d5c1f74791cf225c55eb121d3bed769eef511fada0a666f1425eee957fe5c0cc58b07704ad083530b31b4573a9b320cf97b89793e9a0f498c4378bb808353a09c56c07f661e6bf24941c7d6c17aba7f9e529907eae27a02bae967a9ed3737c7f0ff57bcf3ff303dcce14bf7c8066d45629d484fbd6b7eba05f681ab0d9d679ad45c105b41211538fd8e0174f77c0b26d0e4b52d20616c1cfa0f0af938a28a4d9fec146aa2d6564d599f7364ccb0d799fc0cbba634b9c7b06c180").into(),
                    hex!("f90211a0c2690e05ee400d3821aa8b02f00278b2c914ddd1ee98c3ab08b9a5ed396f2a76a06eb76932643b5522ffe7dee050719ef4c108de2e45dfdb7f46af4b335e953c38a0d9a4856d97ebc8f3f9fe9fc2fa2fd1a1e4e6d6e007de8e41dec557108e9405bca0bf280bfa88ac2e82cd457779b1f75c01e4b20b6e205fc99dfca401cabcb5f771a05ddcfd9420716dbcbe1db6327149e53a4eb994eb1173aba938de7716ac51b8c5a0af58096c6e6b65421582224dac49d2b3233681a314e0c65c9de78c15073731bfa07fdcb5088dc5f44eadf66f282733315f7b6390efc6c55ec9fd0fdeae1df285e8a07a6a0b694b03afe9039901489d6855c1b540a097fbe93766eff2c10c70ec35d8a0192db5cb96512327890d1583fe76f2bd773db0edf92f78fa8fede109d2019794a01b085a612ee42af9c4e744f3d47e367725b5ac53368c743a5e73678d9421ac7ea03a41eab04af9ced63c2e2090e30c8300746193a9934ef87e671b66721b87b3c7a08078971bdb6bfc1c85417018af9cd57c333c65a84b2f9e4ca0dc2a165b54b51ba04e5ba27c81ee80e74ccfb2bfbcfdda19d90daf980423345dba67b7c505a84bf9a0d16aea477c5f47a524848749596d5214a70c5ada4d1399c29718c47c25e49cc4a001eae41eee3289077e693ca1d78bdf713e6c28dd561f7df32810bbc181c4c4b4a0be97e340305c8f56f7be48b38927cf34695e1b95e8ae1f5ec62ac5d7d37d8a2f80").into(),
                    hex!("f901b1a0c72aeb188d7cd6741489ef5460480de56a13efa12e958d431361cc5dbfd8512da03173fa72da502e38933a7880e0d11ad359c349ed114622b648fc341c1b9db583a04bdeb919619279e2d29216a75984703b0c54acd97a2397779cdaccd63aa469fda004ff6049b5d49cbffcf04eef98d58a01390521a92a6be2eb9a3b202cfc771d97a021c85ae0dc5cd5580412da6382bf056fabbd90b7d4c97b984a41c2f2924d4007a06e3ab8543da8ae335e2e6ecc6f2e002e618eebef0ad6494b5e543dbe20517801a01cc712310d45e087795101c1799a648d0d9f0208734502bdcfdb9aedc52c1dbe80a0940d53ac9828d4798298e408f8e27736b4ff7e0e96bb8e95e72115d2f294c97fa07ff22562db4aa2796bb2c52a2eb7d33d619684ac8f34bedf4e38634fc3ce4fc78080a0f6e5f5264fcb01f565f1893672d545f7e26ba7349cf33e5083de2201192fcceaa0506f354afa7e62f5efd6ae46fd65174fcea9557f1fc352dadd55ed704e0747f8a0d05c26a817a7fbae067e79249c08e8674df5694fc465b3a73887c1166c346237a053e92c2946ce9febc76672c85442bcad290c4f114604ebcc0d98a83d489464e180").into(),
                    hex!("f871a024c800360ee20aa4e882ea742c38435d6ae2a4dac63d0c54c46ae39ad6dd414a8080a08565b0e0452ab5a5581acd46f18a0c3476f9a77df3eb3ebb41d950ae8605aca98080808080a0e7d7bdf5202403e405168010300e9ae1ad4c6c512d01938664d11cde2d2c963b80808080808080").into(),
                    hex!("f85180808080808080a05193909176ca6af398a84f40d58e1dd816119381fbef535cbb9d4a7cdae86889808080808080a096948b84dfe566746ca09dbb5609b1462faeafa486307d44260dc25ded12024c8080").into(),
                    hex!("f86c9c3cac147edcaf01485fc330baaa3a14097ebd75da6e61cffcbe83a07db84df84b01876a124e1747287aa05091e8afb812f9c63db3d1e54a604d85b57e686edf3086a611ae9974d94abd12a04ffd6c414d52ca0bfd8855fe16f448d32a37d72192fb3a72a7a5811935345e5e").into(),
                ].to_vec(),
            },
            l1_block_hash_proof: [
                hex!("f90211a0583d36368c1f5477f093f33e55d40ad5a118147847b1cd68b3d486907b47d182a07f27ff7f61f08a6ce1c1d217855e731073e9b7d969aec8abe0a184545eab51aca02d3c185f6e08b2394444e836bb4b668ac55baf649442278a0cc57b2bdf4449f5a023a4faabad295e8a4caccc7e00b45303cf2b4e2a0579d4bfb873e58340f36ebea0f8ca7cc363d40801e0b0b2109114004e079da15c114537a57d2d51f3ec3d4e56a0eb23e560048e44611f6a679ca5202bb76bbcf1dc5006e358400be2cf60a96a2ca0a14466ff61f3f4074b38341ebf0dfd493d1f459a71ab9fedbd2ba97b89792d59a0f6b13e60c337f605d58ff143cd9b21d1331e4479502b914aca075c7e7ff14b6ba0a4ca083c4289962304e3e1481645980f88cbda9181290c04d3b0d5b192447be1a0b09a8bee71cbbf6fb53f1e41b964766ef4a969d8b74e84b1f8998cee7e1dbed6a0041a03f5cb048e1fcfa8ff9735eafaa2efe54ebb6e8f823ae41f3f4481b5ec78a09afd43bdc2ba6ea8a7998b9e2e9b6cf773f7e1dfb6bdc72281e619af90a1ad8aa06443f08f5546c17b7e81a2766ef4427683b4ba6f0f7d116cb73cb23196ed6907a0a73fc513909b6b5f5e90cd8aa0e76cb6640eb2703fbecfbcf0d35c305e02d904a051298cd8203ca84cd582097a5993f33fb204e08363a4caada418f0b99e69f3f1a0e423c489919502d593a504411c2bb831612ae1f19358bf39bf2026c169edc32a80").into(),
                hex!("f90211a0090c287ea962a21823eea59fd0b6269f05ab109812cd9135ced20cf471c46d54a081f129930a4f1b63fca98154f5eb014810285e02c5d4bf13e6b81e4a9edd5edca084fa7abcb2e99589af47519b95145481cd91b9638bfa63d7ccf6afed560c3e69a0c350cdcb332ba0982b234cabcfbc253e8932605c5e19a33d9fce264640155b90a0d369bdb5164717d3bea735a0f78f2a5ecc488f5dad0118865530d05dd4f0509ea0dfa4641f31de666bad4a7b5d476586321223b8f22db395a111126832fae69af6a06f45b766b761350eb72b1ca252c170ddeab6e87b9b2c4e03cdefe760b0e56626a065270cdc0c448bd6e3fab6002aa582ea2191ae0d2498e7c77d1b9f938c98e02ba089f7d99291980e8ab9549e522591f3de47d82b7a0691889dde258f6219d14e9ea0a7820e4c80ba26c137692ce2ef6853fa7f35203b10ef9aaba067d55fcfef463ca0a1b6582eb71c5880e8b6d35f3dde1a228533a289bd219d84f2604d030c2fff36a0dcdab5f4ab143517f859e97e049a4e3a6b14882df496cfd9761202c56816f193a0698b5819aaf0ed47812c7d016e6c731dc6d477f27ebc5793fd86cfc4814a88caa09914cffd9784b73f23b6f64bf7b8186652ddea93e2811a97a819a05045865f28a0f87562e4410f1d65c10ea9fa638de34e272b71809c3c9c3afb6f6dd1404a2a06a003bc929ce766845cf6aa415d409d1f5853f36bb9f235f789a7852422cbc0d9d680").into(),
                hex!("f90211a01cc2de60cf5b8e2bc98964e8e881e25991598b583325eb055ee1fe0a4514eccfa06ffc5ba0c164d5298b5ae21f8a289a65d48bce7a9cbe117efb5638aa82136c32a0fc106db0e99021f818852943c9258b7a8b6c358c8716405a4a7250330ef1e88ea08912c32bc2b9fe163d134aaa4423f497bfcd3954bb8174340bbfc7ef518af4bca0782329cc528964573c1461b5eecdf62737202a5db06d38b1d566bde52fd11be6a0cf087dfa40544d213b8fd50031cbea189fff67fe0090134a926637b5805942b9a0723ab14a8108e2c213a773c6c76dbebed2cf22d98ff830458c67029692705b72a0cc842c1a7171e6bd068d01ae3a5d61a83c784163b4beabc79ce7e8d9295ebbf8a0ceede33b00f71b7bf0f5ca0cfe3679c16f61f3d5ac9deb1ff5b3e2e47273ecffa0b4b1baaa90a2249a06c457b11930871f62b0a98946be60b77cfe9308ad480b9ba0a07e4d1e50ff7a0df4c32ccac682e33198958091f8d0091687a43ab62c150853a05195a228a1eba2a239cd8c326c45ced6d00dc39fb4b1962feaf77f8902b56870a0969467b9cb6be3e6f5392645950d3acc7defbb62cc6d0078aaf10831df685b0aa02f4dd11d173081571593c62baf6ab440c0031e844eafcbffa6b21c59f592fd3ca04d72eb8823c91ad1238f218429514b704df4886cf0f0e4f94121aa469d3bf1f1a0c10a29dd373e05f7cf0c0fad487390dc45c12ddba738aa3e668f3bd069d7b8d680").into(),
                hex!("f901d1a09888e7f467aa7102bfb8e12eba680affaec196d4367a604227525bebfc481d55a04cd3bb41d9e1421f18171024fb298ef336956a76d9c1b1b21d0b7f981672c218a016237700134d4b0d2b74aabfc6c87c6c3c041f60d35075a2bed1e59f0556f7c3a086174c6c0236c93805c697e7c95af1412440294f389dd6a9a4b1b335fa4ca1f1a0ee8ddd4b1dab1b067d4e6fbb168e85c0fc965810c6e524d1d204a83000e4a4bda02ff83e3ba14486c08b834b921b5bb1b1d18e1a7073e0ec04a1594f65f20db00980a0d8b5da3b472fa8abb8cc8c7cd615ee46eff77de32ccdfedf00d938ce7b0abd61a057364f7d7897a4809d35f847c959901ada44ada96e451fdd5850ae3d1e15b916a0aa6375dc4677aa7705d05ac753aea9889420826e533b52a6ca713312b04b76c6a0f92395ddae2d93f6539be5834617c59149409d0f648772940551aad834a2db13a0c15e237bc52d93797840492ee364a6346ef3d4b1db6df14024f15e26d823601680a01334bda47273395a27d8b6b55fae2e1e0666d9d711b04ace936830484649d4cea096ed273056ab8f8d2d216629b4121a53214bd02bddd24a21edec45ea49c808a9a093dd5984df02f3dfc9312487e20bd847a8a80308111111475a9089864999841180").into(),
                hex!("f8419f205f3c1e0a04583bb4cbd325509f33f4f67e97aec722cca0265adf6f0c1217a09f81b8642c7ee73a7dfdf40963a6d2d2c67ad116504992c1e2d52869f864c12e").into()
            ].to_vec(),
            l2_block,
            l2_ibc_contract_proof: ContractProof {
                nodes,
                contract_leaf_data: starknet_light_client_types::header::ContractLeafData {
                    nonce: Felt::ZERO,
                    // REVIEW: Should we verify the class in the classes trie?
                    class_hash: felt(
                        "0x69b893a8b6e1bf94740e33d9584a01295510f3b51f024d9833b2acaf1be4045"
                    ),
                    storage_root: felt(
                        "0x2c8771df74e758b1fed285eef0cd07cb84b55abfabfb0d6a0f1b7b3aff761fa"
                    ),
                },
            },
        };

        let l1_state_root = <H256>::new(hex!(
            "800c716ee21145783ef258df832d9501a02dc1fad1d4c03af08f790ad5548693"
        ));

        let client_state = ClientStateV1 {
            chain_id: Felt::ZERO,
            latest_height: 1,
            l1_client_id: ClientId!(1),
            ibc_contract_address: felt(
                "0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e",
            ),
            l1_contract_address: H160::new(hex!("c662c410C0ECf747543f5bA90660f6ABeBD9C8c4")),
        };

        verify_header(&client_state, &header, l1_state_root).unwrap();
    }
}
