use core::fmt::Debug;

use ethereum_verifier::{
    verify_account_storage_root, verify_storage_proof, VerifyAccountStorageRootError,
    VerifyStorageProofError,
};
use sha3::Digest;
use unionlabs::{
    hash::{H160, H256},
    ibc::lightclients::scroll::{client_state::ClientState, header::Header},
    scroll::account::Account,
    uint::U256,
};
use zktrie::{decode_smt_proofs, Byte32, Database, Hash, MemDB, PoseidonHash, TrieData, ZkTrie};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    InvalidContractAddressProof(#[from] VerifyAccountStorageRootError),
    #[error("{0}")]
    InvalidRollupProof(#[from] VerifyStorageProofError),
    #[error("invalid zktrie")]
    ZkTrie(zktrie::Error),
    #[error("node value mismatch")]
    ValueMismatch,
}

pub fn verify_header(
    scroll_client_state: ClientState,
    scroll_header: Header,
    l1_state_root: H256,
) -> Result<(), Error> {
    // Verify that the rollup account root is part of the L1 root
    verify_account_storage_root(
        l1_state_root,
        &scroll_client_state.rollup_contract_address,
        &scroll_header.l1_account_proof.proof,
        &scroll_header.l1_account_proof.storage_root,
    )?;

    // Verify that the latest batch index is part of the rollup account root
    verify_storage_proof(
        scroll_header.l1_account_proof.storage_root,
        scroll_client_state.latest_batch_index_slot,
        &rlp::encode(&scroll_header.last_batch_index),
        &scroll_header.last_batch_index_proof.proofs[0].proof,
    )?;

    // Verify that the rollup finalized state root is part of the rollup account root
    verify_storage_proof(
        scroll_header.l1_account_proof.storage_root,
        finalized_state_root_key(
            scroll_client_state.rollup_finalized_state_roots_slot,
            scroll_header.last_batch_index.into(),
        ),
        &rlp::encode(&scroll_header.l2_state_root),
        &scroll_header.l2_state_proof.proofs[0].proof,
    )?;

    // Verify that the ibc account root is part of the rollup root
    scroll_verify_zktrie_account_storage_root(
        scroll_header.l2_state_root,
        &scroll_client_state.ibc_contract_address,
        &scroll_header.l2_ibc_account_proof.proof,
        &scroll_header.l2_ibc_account_proof.storage_root,
    )?;
    Ok(())
}

pub fn finalized_state_root_key(slot: U256, batch_index: U256) -> U256 {
    U256::from_be_bytes(
        H256::from(
            sha3::Keccak256::new()
                .chain_update(batch_index.to_be_bytes())
                .chain_update(slot.to_be_bytes())
                .finalize(),
        )
        .0,
    )
}

pub fn get_zktrie_node(
    root: H256,
    key: impl AsRef<[u8]>,
    proof: &[impl AsRef<[u8]>],
) -> Result<TrieData<PoseidonHash>, Error> {
    let mut db = MemDB::<PoseidonHash>::default();
    for raw_proof in proof.iter() {
        if let Some(node) = decode_smt_proofs(raw_proof.as_ref()).map_err(Error::ZkTrie)? {
            db.update_node(node).map_err(Error::ZkTrie)?;
        }
    }
    ZkTrie::<PoseidonHash>::new(256, Hash::from(Byte32::from(root.0)))
        .get_data(&mut db, key.as_ref())
        .map_err(Error::ZkTrie)
}

pub fn verify_zktrie_storage_proof(
    root: H256,
    key: H256,
    expected_value: &[u8],
    proof: &[impl AsRef<[u8]>],
) -> Result<(), Error> {
    match get_zktrie_node(root, key.as_ref(), proof)? {
        TrieData::Node(node) if node.data() == expected_value => Ok(()),
        _ => Err(Error::ValueMismatch),
    }
}

pub fn verify_zktrie_storage_absence(
    root: H256,
    key: H256,
    proof: &[impl AsRef<[u8]>],
) -> Result<(), Error> {
    match get_zktrie_node(root, key.as_ref(), proof)? {
        TrieData::NotFound => Ok(()),
        _ => Err(Error::ValueMismatch),
    }
}

pub fn scroll_verify_zktrie_account_storage_root(
    root: H256,
    address: &H160,
    proof: &[impl AsRef<[u8]>],
    expected_storage_root: &H256,
) -> Result<(), Error> {
    match get_zktrie_node(root, address.as_ref(), proof)? {
        TrieData::NotFound => Err(Error::ValueMismatch),
        TrieData::Node(node) => {
            let account = Account::decode(node.data()).map_err(|_| Error::ValueMismatch)?;
            if &account.storage_root == expected_storage_root {
                Ok(())
            } else {
                Err(Error::ValueMismatch)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use unionlabs::{
        hash::{H160, H256},
        ibc::{
            core::client::height::Height,
            lightclients::{
                ethereum::{self, proof::Proof},
                scroll::{self, client_state::ClientState, header::Header},
            },
        },
    };

    use crate::{verify_header, verify_zktrie_storage_absence, verify_zktrie_storage_proof};

    #[test]
    #[ignore = "testdata is invalid"]
    fn test_update_header() {
        let scroll_client_state = ClientState {
            l1_client_id: "08-wasm-0".into(),
            chain_id: 534351.into(),
            latest_batch_index: 65031,
            latest_batch_index_slot: 156.into(),
            frozen_height: Height::default(),
            rollup_contract_address: H160::from_str("0x2D567EcE699Eabe5afCd141eDB7A4f2D0D6ce8a0")
                .unwrap(),
            rollup_finalized_state_roots_slot: 158.into(),
            ibc_contract_address: H160::from_str("0xE52c957533bd932E357046bF721D2Bf2368ef1B7")
                .unwrap(),
            ibc_commitment_slot: 0.into(),
        };
        let scroll_header: Header =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_header.json").unwrap())
                .unwrap();
        let l1_state_root =
            H256::from_str("0x4d47173201f8ded2c250d7f7f572a22d13061ed83009f451d271e0fabfa44425")
                .unwrap();
        assert_eq!(
            verify_header(scroll_client_state, scroll_header, l1_state_root),
            Ok(())
        );
    }

    #[test]
    fn test_l2_contract_slot_exist() {
        let proof: Proof =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_proof.json").unwrap())
                .unwrap();
        assert_eq!(
            verify_zktrie_storage_proof(
                H256::try_from(
                    hex::decode("1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0")
                        .unwrap()
                )
                .unwrap(),
                proof.key.to_be_bytes().into(),
                &proof.value.to_be_bytes(),
                &proof.proof
            ),
            Ok(())
        )
    }

    #[test]
    fn test_l2_contract_slot_absent() {
        let proof: Proof =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_absent.json").unwrap())
                .unwrap();
        assert_eq!(
            verify_zktrie_storage_absence(
                H256::try_from(
                    hex::decode("1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0")
                        .unwrap()
                )
                .unwrap(),
                proof.key.to_be_bytes().into(),
                &proof.proof
            ),
            Ok(())
        )
    }

    #[test]
    fn broken() {
        let header_json = serde_json::json!({
            "l1_height": {
              "revision_number": 0,
              "revision_height": 4637995
            },
            "l1_account_proof": {
              "storage_root": "0xc38eb6a242688def560307e353f47ce03c64bee1efda8e80d5a494876204bbd8",
              "proof": [
                "0xf90211a0a58f66ec49f830a7b6111aa74c875549ef1b0c3f0cfab25bd0c633ed127626cea0ab4c165428957894c458f0e79880a4704233b5ff113d877629512608520e2f7ea035a64d456485de695cf7c6fc5c0244527cd74c7e242dbe2ca13a36c48796731ca01c0a4f070e905fe5c73f072d022df23fcd600f1e381cfa23e6ce41addb5b0db7a05c7c45d7422c7eb43316397ab93cb10cb429abfb7adb3b48717a0983b9490050a02e3b64df22910e32402f6f2c89cb6a66d4caf7f3deb2844e5d8087a28df9a7bea0448a76ac44a8cbbc929b667191835bbc4ca2e17a92027ab9dc0e4d1b303a6ebda062c544b06a6a48c2c41e8a2a063d0cbb87eef945073973c7095df56b4edf2ca6a09cf0e4ec8aa155348bfcafd32d5631f04a3c57b89e2a4ad30fc0ed39d4bb8634a01f7581f4d35abe7bdc246a2b84ceb3abe6c36cf83e70ce64e6a184c0ec43d10ba00c807d0f42ba8bad4ff1f38ea805ecc6179b8db52e13295ada4c59abbba25e4ca0e5dde40287737f975529e61bd32006f8bb998752df2e0b5c451a18206e0260a0a094dcb584a453d13d4dbe37c560d66ffd72196cd844e04457e7616b9e216ddc92a043c2bd0b8c9e7a3a690ab549d6f2591251673c2100e2e387fdc8e5f59154359ba01f7bd3938154feff1c0db7c6ea83ef548d695f7961147aabb956d54ecbb5dfb7a03dc231066e5c4e41f1d7c27e817e01a05cb2c2425f9f795dde2a4c3db97081b980",
                "0xf90211a04100bbb62aa7fcb52b1e283421daa4d5b914588b0c384763153072a7b81fedf9a031c21e66bd86ca3fd55b00bdb7abbdea62a121b30b6c53dd65c76295af6fcf56a08155835933c3bf098b0546fafb67b9dc042a5d467306de31fb565115c9c7ae35a0f281f8caf361a2930d06f634142ba19efb841ee693ef08de77f8af4bed9fb795a094c840808cb93faf504c43de097c577c254c5d7298893c84c9838c006367800da01b0e6e86ec8add91a8b7e3e664ed132e45c4d11b85e4c027ca023728b9941656a01aa70dfd4789cf3b7def6a2915e5b121bc65fea68e695f60b66721096d67e3dda02dc77bcab57e005eaad658e24ea68a7064634cc1185680082b3c1666ba691cf4a0c08bb855eff68e28515ec2d8fca67f423ab0bb53ac7fddfffd36de3cdacc1814a02ffd8d6a46c63d180b4db23ea0f1f6a1e2422109c89c947f24108688b447ac24a04a77fd79d9c28a365da7081464f8603009bbbc79cf64c59e2679d8b8a40fd2eca0bd80f34bc973f7f153e9b76dcfd1cf60396a5189d7b3f98f1cffa53e7cc098dea09382182ff667108203fad6751e9f08c9d78b48ba058d679205de936c4ca62eaaa04ae61a7a196a3835af5193bafd34b9655041f867fcf0cbd75497503e0aa12907a0f04426caaa099498de4888ec8f5c58a4866eabd713c15ee3e443b01c8f5edf77a0de10bb131951794b77b9908619e07828ae0d59ccdea229e644b499328b1aa88b80",
                "0xf90211a0671341e4406640305c02d1bced9422611d549da92b43b1bb34c1c834b830a814a0657a0639957191474ab2624cd0d2221237bb7d87f1525fdcc80b80db080a4fd5a04001cb5dc153d7c16c0d4247f5a4673abdfd75c3c93cb9aded2ff23291e95e82a0e100bd37f99a68d3431d4a2d42e2b640b1565ea1529e1b99fe51e233eb818dbea0244a05127d0948280ea87ae202fa12ab33757a6c19013b01f8e72b4f8f540c2aa021b40d81d948854c35f8511f4cd61ba0c92322ac21dab7c266e0a2e6af4ced8fa08d3da57dfeb682bcf4156c5f6aeb07dda1df9f5e376299946dcba4ef3f3b6d78a0716c541f8f3dc04ad244abdce6724b894a2143951684c3cbb49cbbb37a6b3efca011923db60fa8b86979a5b71882e17544cb2a2e803c398e6809ff50c91a46d481a01c7554aef4992df1a456781effc1192af4f857d1e4d18d490f5735aa9a04ee1ba01c7672ff5029ba2e31b025b475283f8fabceec30f9b9f08b2d5f73d213334876a045cffbc5010d4927e9ec2b5a6362fcb447ca3b08ac6f56f27a2fe9c170505168a090ca1c577065acb4477de8829b1c5e883f9f2554382d3d5c1725a51c7000970ca033bbb72a15b3de529f15a18f7c0d637470f7ac83bb5ebbb37ddb782be53c0b19a0bfc09176b802af0aebbcd49408e8876ad724f7f3130c9e74ffc8947185aef864a027b160cd3c50d6e705d7adf747c372e7328537252a6f4fc67a78830b50a4097580",
                "0xf90211a044b47a242f6c7ed0f5fa23241206e72b73a5821283b188d3b54caf33e247c20fa0f9e7e18d22d625bfc207558d29a23c63227ad907d3a1070b11519da94069a69fa0b2926f8be8cb2c53e76e44520800f861d82362c4c106a2bf220ddc74686c34cda0ecad0f2c551058bedcbd614cd0040092419d1f5d1d658d5e9fbee34bed71ce07a06e182319bbf519930f2acd75dacac37ee427893093c2b53c1590c9a62c9d2abea0d7e16b671886dfaccd74163e3773786a7c18a95c2c20e3029e341f5b335b6f58a0d533e639cbba4fdcfe455814666a072f991d6e9f11b31f5b034698fc5abd1ad2a06a5bcf85bd108decd392bcc774c26a53e40b1fc82c4c8cf9a61f2daf65c671dea00abaae9008bf9d360b7347330a83e581345b525e2cb3e8fbf02dad2ffff977a8a04d3b7cc55cdfcde9978a7fb10755e012fb02eecd73c13621e03a13ce66bbbf39a050464dddab79d00549e827c1b3c3a31b5634155934afba80fa5fdce37f6dde74a06cfd88a2b17fd7f201f4f2ab26098c6b651db5c674b48ea4246b2f3c413f54fda038b5797ee479fc2efb7ae7972ff9ed0a3852ecf81accf2f271022220b36216bba064f094d02e0fc581e4f2ca6e7d4096bffee4d930d779de8f69f81115123d90eaa0499b30344021c1ed22f8acb7b38814bd5d39dae9d32361cbae7d791eac4a6b3aa0536f596a3422bad4087ccfd98f0ab94a9cf4ac10ea7ded41f91d9a04022e379f80",
                "0xf90211a0846ed4027f4ec434d67b345599f4a06758ae1f9cce10c7295b06da89d3fcdea5a0dbc19ceab12e962c61bb51e65ef2a0aa3069983321629345056a582646d903aba0e93a7f3d109585740a1602c2fa366f9b57830771bb55309b291934d8ec34844da096fb77310a4ad476757b7c18a8ad9dfb58d4675873a6464b45a548d559889558a0646cc8cb10356f9290378ea7d183f29b9a6561addf2619f1ba922085c196e8eaa03e20e096bc45fecc4b1334ff7e2c7644f105451cb57ac0b052e7673af8b06505a00c6d289344ccccd31825ecc729fd89f762897ffbb1d038646c00a21faf087145a06397f2bf58c0ed43617044141b704c991f43adb2d971c6ed60b05f16d11a8913a02f33fa3a08071bc44d9cf173318932eff2b63170c2451ec5d031d52413a6676ea0d97f601dfdf7f8145d442f6ce99b0c57cc672fc99a883283f16634e995dc7e2da0208c8d9781bc3b437cae67352f8090f3b186ea98dc51f02b6c633d895cd25f69a0c173cf79089c1da18342f6e300f039f32a0f17034f063a8a71fd9198939c5ae6a0aa762991e48bb6f9889aebd601a26a4727365ac90f20a3be8c4b1f7cf4e1fd62a023e464bb3e6fd240ccaa89f89ac6605aa897fc38731b6534c883d7754ba42b32a0d95800c528bd89e04faf83b88004120df87f736909a572ed7c9500980a06c0c0a067ea0131b01019c09cbf242de8adde4c340dd03ef45a3ad92e17399bba124bc080",
                "0xf901d1a066d423d34e5a9b16cb92a7b2cdf53f05ab66d3f18aeb38d365777649d13208bba0ef8bd9c5041c37d210b732b513b2fd218ce38526db918ba35089364409b8aabba0130f2594c868ebd78c401c6618a7d602d80e0197d5129ee07d0bef880059a9b5a0afcd23ba8fe2f7c882188bd4a5a8cb7ce87299736495d309b90d92ff87b8aff6a0cd6ba81a6514383d48a5462a7a5367178ef1ad8c9edcbb1377f30522709ea771a0cc1e2bf25de83822d1391886be1e93256be68c6123f19df2c2e38a3f3377016ea096ee188171d9b276721f98abcab6488d7b65c563af3803627dd1c405e1706dd6a0567d9b4478d01a0be34653015dc55f113bcefab5827314ae7a76f04df7facfd4a063855a2ef44156d7e415e7aca2dff91e73eb882388012b2b943126b715ff46fb80a0acfe458fff6a2edadb8cb326cd29ea15f4dc992b4df67d15668f7f5e9859922ca0ca477a6f6494f2973d7aebf600e47061ba702fa28469272cdd48df9f38416a46a09d8fcb47e73f0ec91b70e7d1d25ea8b832f538848079e2df60be7fdfc5978a1e80a010d68c307a7c7227437c9271d90fce3494319650d78bce7a6f35847880cb5e9ba05edcf0a5cde11abb08c394c81d1e78d8063809f806b551c1f2e4a7910c5013a980",
                "0xf871a098dff7457df691287379f14f3c86c17181a01e98865801c1faeea0752e6113cfa05825c5dadaec2cbc329e20866073b32e850c67627d0a46f8ecb568605edaa72580808080808080a0e8f258feb499fb1af8dd8a29aff43cc5209eda6512cff9c38705bdf3281a799980808080808080",
                "0xf8669d30f80e30c43f7af65d01662a3164f088eb29e38f1a3a3295a28878b474b846f8440180a0c38eb6a242688def560307e353f47ce03c64bee1efda8e80d5a494876204bbd8a023c736713c762f5d684fdda5244e49dc182aa801b78383ddf51015c1597c446c"
              ]
            },
            "l2_state_root": "0x10a74cabfb7b7ff657a6a5ab27692a2d370b89f6a51f5ce2c3fd4b90431ca970",
            "l2_state_proof": {
              "proofs": [
                {
                  "key": "0x0aa575884ef0b2ca545005ea8b6057ce8aff1fb494f9f50527d9d089c4839f6b",
                  "value": "0x10a74cabfb7b7ff657a6a5ab27692a2d370b89f6a51f5ce2c3fd4b90431ca970",
                  "proof": [
                    "0xf90211a0e50f29460845868e76db2a31104bfd4009b3643a2104a95b1c44eecc871ee24fa0a7884441ba95efa151846084d77c40fb865ef316978fcc7a77a567fb0ba53e8da077150ea6a10606b4a284ca1af35a053ed8e022ad22749cf30db5311ff0ec321ba0cfa4f12ac7d9a091048b313c65a8912f4a570ee703dea65f2e1ebac55cce0b85a08387f7891bb7f0b914ba011be65f24432f13c5bd36d3802ef06d062426093e6ea0f3cc2a253a5aa53b1ccbf7698049edca4590a63927f01c31954548605550c85fa0a92399f12680ffcb7e6bc3c7f3d60af0f881f22f3901254f5bbb7cee1b819612a0e8316848ca848eb98dfb47e030dd94585c59286a9aff17fd6b5f2114dba23196a029822aae9a5f771daa7a05c9c7853ad172494cdfd3b30893077486fb5f48af16a07eb7eb115205f9480c782d9d1abe4972fd85f042aae59272e8fde95ccc7c4890a07ca9696dd5c47181917e2e116ea6333256dee1c264728c1ffb47f73b3188135aa017b46380c5ecc3648fdd20c3716ccafd3dff66629a0441a950d4d95243b37570a0a8b02537727067aa2b853d591be12e27177f2ce3d56ffe8bc00f7095781daaa3a07d4c3be6eb1045169edd0843200567956eec5fe21959b3071aaa8bda8ea6dc6fa05911db50054d2c174dff5f65d18ba9b83374893ded6fced273d8e6a7c6fba35ca0fe95d78e6c5f28c1698ea7c1260bd0bd1bb39d6bdd3513f45347d7a46c8cfe5d80",
                    "0xf90211a0bdc0410b5430c81975909457ea3fdf32618e4926195251798abf35033f0ee9aba0513d5529ec80e8e3613ef7ea2369b8b3134585f43452a762278638e57bf39661a0227fb36cf39ee30e8b96a502386c360859fd7f0065dc6fc75d6f9716e6e798b1a0ffad91dcdf3553a29783db41600cba1b4d6ea9af18522048cfd657b9b6ea3dafa0af5ad9a8848f8a4ddf748747abe9c888bda4f0b02c27f6ee472343d0d4f58a07a0a6841b48aa94557c3fce246ed78b11181c30f1cc8e2254e0cc5adb4437d902bda00beeab463a615cd8689a13fcba696a676cabb4a1a723ea26d4d7e5f9117b0324a0807b0b8fdf4e4d8600329bdf4793196f2dc226567dc0099cfb3ab7ebe5db9aa0a0b33fc6db738dc82b5330119d1f0106e2fe3c8327991471354a24524f8abfc87ba0cdaadac94f69f5cf4c9ca3ad050baa2072243ad7464fc8cbc463358cc22efb40a0ea4be36b4f6b4bf9a3d406c74991d2ec77022adc8989a26d9286bfd5e7f90aa8a0aec6f0a803872bb3f94afd49a50afbac6220ff4f61f7d1355edb4dbc8ac6821ca003f83722a53fad443d36ba4f30d7b8cfe9c886d2ac411f76cef10174dd404c8ca05b65e68e13cac46b52b583b2e7dc7bb1aa1ebaa658860b43327029b9e1bdf3d3a084058fa7c7187256bbd1c0f3bc4395655e9a57fc259f01821677f4955e69b246a073b1e9ca822bbda47ec36fc7d4d520d6f39872ecd69f39c63579939cd581a98f80",
                    "0xf90211a015f2d444750c098623946f0706fd614583c8fecd14dbbede1a12583f3637cce5a0c7bacafe8672b306973a89be420ba3d4e0ab6cd1328769a561097fb5bc368f38a07ab2969699848a6be874d5853fbf12939e3ef53ab2a2b4a94f5c28a6ceda8f9ca0135842fe349e8131e101890b19b62322e7b6ee1966ecd48a2e5a587b0b8572a0a02499737a49eb3a4387c586c37407f3d8557a780ced5dfb660404b53dd5086b36a0ec3ff96b5388e981a5f051bbd1d4ae2f40c9aee0a9c7348351c1714d024fded7a06d6a5e50a1a164c6c96e864a7f69b93249accd2a692acc71735aeb75fcd7eed9a002ece77be62ccfb77c1e1234eef28c96cd2443ae062d1bdaa477ccfbd190b735a0b9d957d4e7165127e5865456d157b0b162b68171ab616f2c36bd01c53a3923dfa09d83d8839bb9ea71d1d9fae21bb3446021b1f06014950063042ccadd5845bd3ba0b9e03dacd9e881f21bef76d3de3d5970b1f181eb4b8e0fb9b12fe8666c9e533fa0420005111d256b72f468b57ba1e383842ba3c6122e79d164daf8db40736bc2d3a07184133b5150a0986169c720a5c445193e00c3c2ed823257590a7741bbcab2c4a054f927670e6d83e0b7962d3e647f71430cebca9bb1607655ad7d033c7f7fe52ca0b1bf5ef6c680524e86760bd237bd7c4a78e3aa1eb3b8379c081865b72204d0faa0692bb14742a53b81094178c5d4564a79b913b72ee3d3fbf33b6bbf9d8dfafb0b80",
                    "0xf90211a0f62f3a3bba41178433ff8e881694c01ed5b4e611280b31cfcc05b63c7365552fa04e4a9eb6d8cf7df5f11705b560a9bf735c78c057343d57fe60da34fd6e9ab6f4a0d5dd50d55c6d229cd14f6c081f394910a3aab29ad152c9605b751fc16ac73eaba027b08ba949652c33d82a018d16eb8fc4b9a772f1edc6046f899d4ad03ae992d0a08e8a9f4ed5fe027ef33c8317e350c48f726824af27c819196885369fe2d8a9d1a053636fc736b5459c3ff26154144698b6c1ee6a571e9133ecfe29c360da6f55eba0c10846eb2ef85e81382f1c2658ebbf79c85633343e24ace7d7dc02048089a196a079e30eff818849aee8fbd5876a3a0e24cb236087423f5e1035a775653c253192a021e978ee53fb44a0b163a627f89cd340d3bcf52ddf6ce3c6a71dc75fad80e2b9a042129463343fcf7f3b97df0f0ac0012a463ee4d36e67d30fd4222d9e30fd021ba0da44a9b5fd878182364b7a148b3b2ba22da2541115a0f5e88cdca93a56a5e2d3a0edaf9db13ff1cc0a86872f794123873ed971da6aa6c231c0d95fc214c286da20a048e7060ecb3088f830f860f354361174c21993b9737aebfac77a626cbfdbd8a0a01b13db87f563053d987eacae70da31323962b13805a912e9b8e992a16b421c3ea03822a3b0c09dadf62f114dc86adb6186c9b3dca6e5f5e82c1d2a1ab913e8d564a016b7610a20dfc4b4f4b3d53d67b192e4288d73e4da8f98f111682c09fd2f1d9080",
                    "0xf871808080a034e32e474e004303e0de51acd3b45c73a86e7b690b7292ef4701fda6f5e171e2808080a00aa102899d3d9b3617b22f34275d8cdb99ce8736f9843f6c6321b49e8416ae6c80808080808080a0454cb251e2e9bdaa18d8c42b56347ae4da83c4c99de2b6715ef9853229150d6580",
                    "0xf8419e3c8a5800ca2fc884386f6f2f1840770425ee28de0bdecee827a6724b05baa1a010a74cabfb7b7ff657a6a5ab27692a2d370b89f6a51f5ce2c3fd4b90431ca970"
                  ]
                }
              ]
            },
            "last_batch_index": 68318,
            "last_batch_index_proof": {
              "proofs": [
                {
                  "key": "0x000000000000000000000000000000000000000000000000000000000000009c",
                  "value": "0x0000000000000000000000000000000000000000000000000000000000010ade",
                  "proof": [
                    "0xf90211a0e50f29460845868e76db2a31104bfd4009b3643a2104a95b1c44eecc871ee24fa0a7884441ba95efa151846084d77c40fb865ef316978fcc7a77a567fb0ba53e8da077150ea6a10606b4a284ca1af35a053ed8e022ad22749cf30db5311ff0ec321ba0cfa4f12ac7d9a091048b313c65a8912f4a570ee703dea65f2e1ebac55cce0b85a08387f7891bb7f0b914ba011be65f24432f13c5bd36d3802ef06d062426093e6ea0f3cc2a253a5aa53b1ccbf7698049edca4590a63927f01c31954548605550c85fa0a92399f12680ffcb7e6bc3c7f3d60af0f881f22f3901254f5bbb7cee1b819612a0e8316848ca848eb98dfb47e030dd94585c59286a9aff17fd6b5f2114dba23196a029822aae9a5f771daa7a05c9c7853ad172494cdfd3b30893077486fb5f48af16a07eb7eb115205f9480c782d9d1abe4972fd85f042aae59272e8fde95ccc7c4890a07ca9696dd5c47181917e2e116ea6333256dee1c264728c1ffb47f73b3188135aa017b46380c5ecc3648fdd20c3716ccafd3dff66629a0441a950d4d95243b37570a0a8b02537727067aa2b853d591be12e27177f2ce3d56ffe8bc00f7095781daaa3a07d4c3be6eb1045169edd0843200567956eec5fe21959b3071aaa8bda8ea6dc6fa05911db50054d2c174dff5f65d18ba9b83374893ded6fced273d8e6a7c6fba35ca0fe95d78e6c5f28c1698ea7c1260bd0bd1bb39d6bdd3513f45347d7a46c8cfe5d80",
                    "0xf90211a0117af500abbb7ee05c7d02d14b4d368dadf63e3aaf0a6679c83f2978d603bea4a04769a4923a16ad6e06af1317cfeae5f484940fa6f761c8877848c38d4840adada0222c01af482d1545188b2fc59e893255446be8fa1b99e0a29d8f30ab53b4a7b2a0cdb8202f52643ecea6c1c7fcda93bf12b3c71dd066e3983b2b461ff9f586e9dfa0a03678fd974cf75a62728ff7bddefb6207e48ce71ebbbf96be4dc669c9fd0484a0bdff08d23176acb4e392475a7d0086c38000b4cea1d45534f0f9929e88e9dac3a0f90e07879270698d02f332b842bae3ed0f66d97808c2d70aa8c870a127394b08a0a8c412c3f622137a71c87d3f959686612495192a8cdbab004a7d51c52db98477a0f3a3116c591f20f9cfef2ff8ed6e470a7679202f6655d9f801807393d3468b71a0644f8dbd5dba161a863db25c33ef5be185e768dae7fa2ce6cacfdc56d6331f52a0f18405c00f3b936b127f3d35bc325458c26201a640628569d36d0bc46b84936ea0b43cfefe430b9ee59be1205cffb437308308a747afba7db9cf48bf18143c9d32a0c9b1423537ff7947ec8fba49a76bb2e86f33bc013b0795e94512fcff354d829ea07ee73a42259ea845c9a1a3e0b6570c1f7cbfe5ec19a8d00d96ea5affcc6f3a76a01f0612abae80a74e7e5636254d778201111bb0443ac96b5647b1944301ede6cfa08ddc2e8bf59cbae859ad218f9f48abd2df242a61aa02b1921b9262b7a213738b80",
                    "0xf90211a091fcb30501931490fa04e1136b1155a66a3f64a5e672c7ce84f0f0590f0ba8d2a0ab2f37a143d91bd04a402a15bfc9cd46d7b0f3447a2d3038e30c3b0afbb3d4eea0bcf08d72387052e973840262180f6ca47724ddf4424794b84a60ad3f977d8772a0bbb50ada0fb95908d26ef9e53c4d234305b48ee8e5abe5b1c20c0636796b5619a0440da2f6a47bc20d024e95ad9f5217ff9cc1e16bd4529cdb9adf4ded6bab2c64a00b3d1afe2729c8dd9eb505d044120cada4d9f8078cc6c9670daf95b53c5e1ecea0d9921f99c85e2339f1987c6a7ef359809ca3e7e7262d5c7dc7267cbbf5d55b0fa0cc28828888833de95499f3c3a0cb859440806b641e35321ad6f80b463fc9bc0aa0956f05e3914e1f8381678c077959c8cd67c2459b4d37d149ff3235a914cc9f72a0a0a53c0fe312554c85ffdf81ad2fc7478b7a5c673e9f23ab48cc68457c632ea0a08e93e16e98a4de9d34b99a83a5d8b589f86263cb082ef54dd8e4db78fa50db37a04cbdcc8f7ac350c307540db99b0b8a9730b57034b869d3b85394a1a2f2f93d83a0e31862a7c047e2ae7f4764f10f1442ca1ce385b13a564cce09b5e34cf9c451dba079b0473d01bc1e1bb50d7b6d7ec315be20b7bb56ce13b06cafda0bc5e2880dd9a092ce7581471ce38e9b368e7f08e20296ae05ad674c37d024ec1dbd1959efcdf4a0ae818a365f0b6a5bf051d15e76cf53522855514b8770499802b797b60b351f6380",
                    "0xf90211a09a3c2ec24073de2a6356fe12ff5ff5402a5632ac5d877c4998ddd03ad9f9d993a082fe50db7111be23176def6ada1e0952be361d04f73f09d9d2481dfb1010c3eba0882abfbf12225fc442e1a89de4c560a0f959256255c7f3c1861f0eefc7ef2a94a03c4e6f65f17ab56fdcf908ed3adc800ceb0b876e10b91a511d58ca250d4f826ca0c52a317ea244f426c1afd24db6dc400326510341e83ad040ad87da83215a9777a032ab23536069f7548bc259aed117ac090cad6c5ca56df99e5e2ee20e9489006ea057a44a068321fdfc3bc9e5035116a507729b6ed4d14fbddb14585c5ee2f7e5daa090935c7bf50aaeacd868db04440c21210f4d1f3dddc82dfe4e22c91985c25ca8a0a77894b6ccfa6e524e3a42e820da14edee8dd8c1531038f93448f778fac337a9a0456e980c5f0926f191efa5eb82e8d24243ea8cb07996375677f01029a0c903daa0b033c89299efa1d44e37f7a8ae8882c594ef5c789cff1b0ada90cab2f65e9d3ea034602d86244b1da55b25fa63b912dd78fe2cee4f2539fb46e5130708420a30cea08af45a854a4ff29a43f52ab43f05de265e81663cdac471643698f1e5afa31c30a0e668e4ce7014bc1f2f9c7cae76d218971e886463f1915fc18bbda5f08023bda6a05be22cb81afc07e3038091c5de6d7935521901a649743b0f840b91187ab228a0a0af9953223cdd4bfd048310e585587a099761b6ed4ce64142630125c9bab198d880",
                    "0xf871a0aa1ea7ff8c84dc737bed63bdccbc0437b588b4b6e2b208a65c7ce1256ae9b8ae80808080808080808080a0a5aa2e381e588edced9537af2af5423d4261d446bfb7816439fb05dc6230d0d8808080a09ee310043956460be2c3e3b1da189b120e7a1c9c24d8892238c71f6ec4e1a50b80",
                    "0xe49e39071dfafeac1409d3f1d19bafc9bc7c37974cde8df0ee6168f0086e539c8483010ade"
                  ]
                }
              ]
            },
            "l2_ibc_account_proof": {
              "storage_root": "0x146bb7b1a9bbba737c173d6bf1d607baf54d160e56f0777be635813ce4b6e9e6",
              "proof": [
                "0x090dca03abff6ca0c8924902680f71885fa30de6e761ed5fde49c3ef1254e09d7b255c7c9353bf9936b4d4bb25b7b26a0db9ed69d8ac1421272b6737be0bb24c1e",
                "0x092119497b0a5fc0685fea0ca88bc91a70a68ed2db6c752baed60b9bd19ffd08a5103af1bdb62a2cce98fdeba2236b195559f8a84df6527f8bd0da2e2bba2b71ca",
                "0x091979b50fb630539d408b29df99dad8d0139703510e3b4c575131253aef1528d703b062db638891d8503b9452da2af15c08af87e991c068cf399bb7ea62376c69",
                "0x090bda5d5734fcb02e6b617ca23892c00ef27b3be021186005564db216e9cad73408f51240c5d31d6798d95959fc85bdac18252784d0ad4c6c40bd351752432cd5",
                "0x090f8cbb64e596cecd53a2c839ac4ad9b8b6fe7977938d75c26a60165b1fd4b0c603ae65be43de30d8af634e263d3b9d14df2502e2d1cb662a0d9c5a6467e14a6b",
                "0x091ec61180784dbba3a956071b0ae34d2aabb485ec799ea8c87497e5b49f3480b3007cd0f7702e102a0ccb6539f18b5fc1890d0b16f73e5653b524cf01c094a40f",
                "0x090a1f5f1dba1fe03a8086f089da4a0598caa5e27562da2e859b3f5846551dea950f7f1f0cf53f6310be3150161472c4d859d75f3085859e821d930e1912efe9f7",
                "0x0915c3636da96654adcefeb13d5c579f87742843b1fef93089f35b76c3d5e8617c2b090d6f677baa37c1f42c62f104c1f07f90e6ea1076cc1515c8cd2d752fbfe1",
                "0x091c992e8436ac50399a608fd72f85d31112b037e78fee3c761b76de785963943a252216a400a1520a7b0343192aebcd5ea9b8d698cdafd5dbb0a77199796fa837",
                "0x091dfc7737cf2922e805b4e86981952397bab897f3bec600005cc01466824cf0b40353260c1c3fae58b6b61ca9d623e25fe2d2816a75edab69dfb3980056f85f7d",
                "0x092de0f178aa5aced12ef3723d5c74994c9fd9e2982b030db9d0924650714db0f50c5979a11518a3bbb16888fd0ee1b551ee36b123f1d904d0a1d9ce8f129b75ad",
                "0x0915dc4f474b065c8f919dc73a463cab83f80c9396e5b4461a195c991503364797146eb4cc40fb1ce8358b89727668737032373a069d80a9214bf56bd2cff1b7a4",
                "0x091cf61858b4105415752514323fc2e0f3849016981492e7ea8427f04f0b658a7a0fad58da03c5eac5e84d351443e19615c75e3d2ea33dd3539daa23b401411821",
                "0x090e2be4cf433dd8273c630c5813667a2a41a050f76351331638e8d4db19b611fa024b68242ade5935ac923e7dbb77ac4e153a73decf0be165715fb545098cfc99",
                "0x09168f67249a461870a3c13485fe77292bbd64edfd13c25952d39c3684c6e992920cd1eb03cc09e63021664c950b2c4c76d808b0b2afd6054983e9e4e2bbff748e",
                "0x092c6833746aab87ee760369019425a41eea6cec2231fdfc78469f89ab1555eb3f19d3c6b18492524cad91350af5ac86fc01bcfab13f8d770e787b6d3bf9015522",
                "0x092bbe415fb61486ed17bfc693d06585df5b80eff3cea0e2f5fd35d835a4b77af1120f5e71898595fe287c4aac6e14fe1f97e66bdc19e12b1c65dfa71b25b8ef65",
                "0x09243e024ee2fcbbb6c1bc338623418814c3890eadbe4b6af0dead7eea7a437cd208326c4f5d43856e8f7185030724eea1c1770467b78895bdb062c5798243dffa",
                "0x092a5474a35902198d2a8a2b412cd0f36532a9a7759b21f945c8afda06600c7de104ce6de9722d65c18f8f5304f25bbd54016a6c493e686b2583f83d8e6c42463f",
                "0x07000000000000000000000000000000000000000000000000000000000000000010128cafe12bacd0babcaead148e502ff80917dd24298b752d0dd7f533478992",
                "0x0917a0af8213ce8324052e8ed20e2075f9885017bcbd7f33e142ddb08d412b398c043ab7c377e7bc66c6ca88c075123a6cba4604f14ad3991a3cc82d255119b993",
                "0x07000000000000000000000000000000000000000000000000000000000000000008ac7416f17dc7f6eece7327a483fed3cd9ca3960363c392c2884e4a3b5ab75e",
                "0x062a97f79d7279c8cbf8f1f5974f7c874a96b945d7563f1229bb16399464fdc6232fc5abf0d4fd40d3a2c6bee956a730333d7a3f4eb264a075172383515a4ade56",
                "0x041e3d0ba31a2c81257b52d33aaa99cfa8ebb0f2bbfacb1940f5aaecf8e3be93140508000000000000000000000000000000000000000000000000025300000000000000010000000000000000000000000000000000000000000000000000000000000000146bb7b1a9bbba737c173d6bf1d607baf54d160e56f0777be635813ce4b6e9e62429e9d186d0250e5a1cc6b390be8e4891ecf0d146cd606beeaa2cd086b732901301231e4942105a9bc60ab0c3124e6d666727182cc9665c133cb9bbc4c5e475206a2c5e2b519b07e6939363f44d9df4e23af73b86000000000000000000000000",
                "0x5448495320495320534f4d45204d4147494320425954455320464f5220534d54206d3172525867503278704449"
              ]
            }
        });

        let header = serde_json::from_value::<scroll::header::Header>(header_json).unwrap();

        dbg!(&header);

        let client_state_json = serde_json::json!({
            "l1_client_id": "08-wasm-0",
            "chain_id": "534351",
            "latest_batch_index": 68317,
            "latest_batch_index_slot": "156",
            "frozen_height": {
                "revision_number": 0,
                "revision_height": 0
            },
            "rollup_contract_address": "0x2d567ece699eabe5afcd141edb7a4f2d0d6ce8a0",
            "rollup_finalized_state_roots_slot": "158",
            "ibc_contract_address": "0x6a2c5e2b519b07e6939363f44d9df4e23af73b86",
            "ibc_commitment_slot": "0"
        });

        let client_state =
            serde_json::from_value::<scroll::client_state::ClientState>(client_state_json).unwrap();

        dbg!(&client_state);

        let l1_consensus_state_json = serde_json::json!({
            "slot": 4637995,
            "state_root": "0x40e0a844eddfc32addb999d11ff1adeefdf9283943e148e29834c90b8f386a19",
            "storage_root": "0x34a5fe80619df22681e82784ec22b0abbb5a59608be10feb829d2f9adda429c6",
            "timestamp": 1711389540,
            "current_sync_committee": "0x8b7bec1fb2897203d4f1ad5f68bac7ff9a4dda84f2b88df6284574788f310e759dbc64ff1629dabbad95bcba39d81000",
            "next_sync_committee": "0x975901985d1ca8eb87255eb62285eac465bb2b35e44a23a800da2c798a61944c6795741631f0572241293cf35cf48fab"
        });

        let l1_consensus_state =
            serde_json::from_value::<ethereum::consensus_state::ConsensusState>(
                l1_consensus_state_json,
            )
            .unwrap();

        dbg!(&l1_consensus_state);

        verify_header(client_state, header, l1_consensus_state.state_root).unwrap();
    }
}
