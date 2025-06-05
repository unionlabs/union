use std::hash::BuildHasherDefault;

use hex_literal::hex;
use rlp::{
    Decodable, RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper, RlpStream,
};
use sha3::Digest;
use unionlabs_primitives::{encoding::HexPrefixed, Bytes, FixedBytes, H160, H256, H384, H768};

#[derive(Debug, Clone, RlpDecodable, RlpEncodable)]
struct VoteAttestation {
    // The bitset marks the voted validators.
    vote_address_set: ValidatorsBitSet,
    // The aggregated BLS signature of the voted validators' signatures.
    agg_signature: H768,
    // The vote data for fast finality.
    data: VoteData,
    // Reserved for future usage.
    extra: Bytes,
}

#[derive(Clone, RlpDecodableWrapper, RlpEncodableWrapper)]
struct ValidatorsBitSet(u64);

impl ValidatorsBitSet {
    fn is_set(&self, idx: usize) -> bool {
        self.0 & (1 << idx) != 0
    }
}

impl std::fmt::Debug for ValidatorsBitSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValidatorsBitSet")
            .field(&format_args!("{:#064b}", self.0))
            .finish()
    }
}

#[derive(Debug, Clone, RlpDecodable, RlpEncodable)]
struct VoteData {
    // The source block number should be the latest justified block number.
    source_number: u64,
    // The block hash of the source block.
    source_hash: H256,
    // The target block number which validator wants to vote for.
    target_number: u64,
    // The block hash of the target block.
    target_hash: H256,
}

impl VoteData {
    fn hash(&self) -> H256 {
        sha3::Keccak256::new()
            .chain_update(rlp::encode(self))
            // same as:
            // .chain_update({
            //     let mut stream = RlpStream::new_list(4);
            //     stream.append(&self.source_number);
            //     stream.append(&self.source_hash.into_bytes());
            //     stream.append(&self.target_number);
            //     stream.append(&self.target_hash.into_bytes());
            //     stream.out()
            // })
            .finalize()
            .into()
    }
}

fn get_validator_bytes_from_header_extra_data(bz: &[u8]) -> (Vec<(H160, H384)>, VoteAttestation) {
    let extra_data = &bz[0..32];

    let num = bz[32];

    dbg!(num);

    let vals = bz[33..]
        .chunks_exact(20 + 48)
        .map(|x| {
            (
                <H160>::try_from(&x[..20]).unwrap(),
                <H384>::try_from(&x[20..]).unwrap(),
            )
        })
        .take(num.into())
        .collect::<Vec<_>>();

    dbg!(&vals.len());

    let va = rlp::decode::<VoteAttestation>(
        &bz[dbg!((32 + 1 + 1 + (68 * num as usize))..(bz.len() - 65))],
    )
    .unwrap();

    // dbg!(va);

    (vals, va)
}

#[test]
fn vote_attestation_rlp() {
    let bz_50932001 = hex!("d88301050c846765746888676f312e32332e38856c696e75780000003bfc8c16f8b5830effffb8608d7b9931586d8830af54e8cc2b594a1871eaedc1e12c28b5c4a1ade2b0ed99d327ba34c0be93cbd618a3a06e7d7892c517ef674c2b0dddbcdb3fa9430df052e7c50301aecb06d5b09bb10a9efd253e218a59c72595e486429ec71d6b434ed203f84c840309291fa0b9315153878916bc5e2667854280314bc639fe7ba9c9407f39f8f920442164f98403092920a0cba6fcf3acadf8fd3c129956f54ebbc7a876f0fb1abfb85a85a6c38328e47e2580bd7bfafc794e0594103beb8d555f318d9f0bf3c835e989dab7ac45432b50238e6be6333159f0121f8af24090d789e1b3b87d2004a8d1b0a91d0cb60dab8f49fb00");
    let bz_50932000 = hex!("d88301050c846765746888676f312e32332e38856c696e75780000003bfc8c16151cfdbd2dff70c6e2e30df5012726f87731f381648e9879d77f0f25c8f6348135cef7477c2455f516bec180921d4c669eae7258857327674ace724b598c0df3fd41068ef837e9627a91dd13e453246856d58797ad6583d762abd04e3688a7c071dbc7eb3d0ace1c06baf163fdc8ffc742fec16f09fa468d30778a3c533b944899d33ae3225a3aee0738944092685a336cb6b9ea58836436709a2adc89afc1c041d36ee43ed51b1cb17b9dff14068e594b79a3c401a0bcf9fae9fd86324822fc0bb768f0b7dae76927b23d3954460a252b4feefa821d3351731220627d7b7d1f3db3e34a6e7967c4da80dd3e5227acb02c92f33a026bbce5e52c19b7d8746b7e55d3e29b9083de0bf334fdf8ac91bc14854e5acf9684652bea56f2f01b7101a225ee33d23f8bcfeba8fcafdc6b6f9016d5a0dd08e4685a13bffd8c2087f66bf7ca2dace7fbbc40c40824e30a84d3fe62a2ddcd52175009317fd4f6f8feea9dae41e5f0a4737bb7a7d5b3a0de43e5a979f8d7a9ad04f8f3f102bdbb17ef0bf6ac9a8fce3f110f409d99828be80295da56c2c7a7becd3647ce40502aecfe253e6aa0e8d2a06e12438ffed0fe16a0b15df58914a6b751909f0558a9f9af8efca7d46e480fc24478d977dafe7daf5161b38c72e9e1ea4865c288ef5b8054ab58567f7a51a58708c8b40ec592a38ba64c0697de8d78def84b10ab93dbfff6980d1054a2bc561bcf0abf3daf6096849bf03744fd4a49392e4813dc2251d68f6f95f27ba775b851a27d7101438f45fce31816501193239a83ad9a5f4ae5ec7dd886b09a47021461ec6f1971b3558f31e622311e94714398c80573fd531e0e8b4c4c3b456c4a5b9bf67e1fdf03eb3ac35bf0256694d7fbe6b6d7b3e0c8a066981de27634c2d17a68333f6d9b0c8cd7e08882c397c6ad92d95f8c279d6ef9ea04a1e2f4c4cdfe7ea6015f367ccb8a239732871adc8829ea2f47e94087c5fbad47b6adc9ae11a5f0da15082a4ded8abaeb73338984c06e2c1af2eb24232e00511e95b24e89291d689f6bedb13d5a398af2ec9bb56c2b4dbe5a06d79911c9899b6f817696acfc90ba8c56b1a3c032a3a0395d0f42f3a00b4f50ce9dc41f819a601e8f17b00e7a4eba8a9af4c4b3fdd31d555ccf70058d9f1b7fae54be07f4fee34eb1aacb39a1f7b6fc92abfb714bc2daced46244ccab91917be3d8271a995636b24aa44ff97e3033f41262b2dad5ce2b31734e4dd3912c5838eea015d9e9206859c13201bb3d6b324d6634276534a516c1b45b6dbf1333a8b5accf95c7c33c67348370f918126c7dde43f16c9b14782548464da1bd09084bf2efd915e6fab4647b856cb9c3856d559c885bed8b43e0846a47a663982486c84b2f66d9391efe6875d30be1d907e55d9c4a5a224de92a5d8ff180cc4ebca44253fa5a9730cc89d61994bdcc079bbb23c1d9a6f36aa31309676c258abac7a2564fd6f7101c1fdb441018a24d25672826953caa03b6717e26e8af1c38507dd570bdedf1cb270c376630823f101577c2d534f079444e6e7ff9dabb3fd8a26c607932c88633993fd05c4b6293e6f2b8429a628cfa00ac7551eff4940eb56f1f480094b1eff8eb73a9f958f164e944c27e00565cca503a7ed99eca485da2e875aedf7758472c378c91dce99bbdc44ee9500ed1e5c864bc88ba518585c7e6de5e94d26ee216dd8a5e06c5d2fc740123976c9588787b54998cccb42a9b8d6c46468900527bc741938e78ab4577b1e211be938b9f77888cbdc7bd3787148a6d1653eb0e17603df69d67db30fa4857e11edec461e01e3e02cea2c7d2c5ddd3b0d838ccceae7ebf1781d11d1bb741db7fe1a7b39bdc22b7275d8456ee325424d33829d364270b1ecebb389318c7f0cb1f1c334f05d1d5e26472f2cd07f5d8025ea266f8de5e61322302b2c6e0a525cc842f10332811bf8e69853df9edb142b5d596f93bfa14253a733cb9d2d5a7ed1fc345e248a8cae7f23f438930123eebf61c98785d846a8b08f8b5830effffb860a88d688acf891d7403c5e34174d806f5b795171b68c70cc846521865ee3c96dcfc53e150a2223bf691e6038b5034af1a00194c90b16fc97c0282671b6774e6c2934ef33ff8c5ce39847d04dade6b00d7d1bc13f60b98a6be08be3eb090afda3cf84c840309291ea0547c0c9a3fb58c15a359e0a1ca7059e3b69876320c19bf1d1a57edb2497f76f5840309291fa0b9315153878916bc5e2667854280314bc639fe7ba9c9407f39f8f920442164f9808cb73a09848491f151ff0e979124f2474d71a32a5ff091fc7832e4658132fa4b2debe45bf4059e228fbf5d11b10ff86627d14c0ed1323346720f0322e08825e700");

    let va = rlp::decode::<VoteAttestation>(&bz_50932001[32..(bz_50932001.len() - 65)]).unwrap();

    let (vals, _) = get_validator_bytes_from_header_extra_data(&bz_50932000);

    dbg!(&vals);

    dbg!(&va);

    let vals = vals
        .iter()
        .enumerate()
        .filter(|(idx, _)| va.vote_address_set.is_set(*idx));

    assert!(vals.clone().is_sorted_by(|a, b| a < b));

    let pkeys = vals
        .map(|(_, (_, pkey))| blst::min_pk::PublicKey::uncompress(pkey.get()).unwrap())
        .collect::<Vec<_>>();

    dbg!(pkeys.len());
    let raw_pkeys = pkeys
        .iter()
        .map(|k| <H384>::new(k.to_bytes()))
        .collect::<Vec<_>>();
    dbg!(&raw_pkeys);

    let agg_sig = blst::min_pk::Signature::uncompress(va.agg_signature.get()).unwrap();

    dbg!(<H768>::new(agg_sig.to_bytes()));

    let res = agg_sig.fast_aggregate_verify(
        true,
        dbg!(va.data.hash()).get(),
        b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_",
        &pkeys.iter().collect::<Vec<_>>(),
    );

    dbg!(res);

    let agg_key = <H384>::new(
        cosmwasm_crypto::bls12_381_aggregate_g1(
            &raw_pkeys.iter().flatten().copied().collect::<Vec<_>>(),
        )
        .unwrap(),
    );

    dbg!(agg_key);

    let hashed_msg = <H768>::new(cosmwasm_crypto::bls12_381_hash_to_g2(
        cosmwasm_crypto::HashFunction::Sha256,
        dbg!(va.data.hash()).get(),
        b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_",
    ));

    dbg!(hashed_msg);

    let res = cosmwasm_crypto::bls12_381_pairing_equality(
        &BLS12_381_G1_GENERATOR,
        va.agg_signature.get(),
        agg_key.get(),
        hashed_msg.get(),
    )
    .unwrap();

    dbg!(res);

    // for (addr, pkey) in vals {}
}

pub const BLS12_381_G1_GENERATOR: [u8; 48] = [
    151, 241, 211, 167, 49, 151, 215, 148, 38, 149, 99, 140, 79, 169, 172, 15, 195, 104, 140, 79,
    151, 116, 185, 5, 161, 78, 58, 63, 23, 27, 172, 88, 108, 85, 232, 63, 249, 122, 26, 239, 251,
    58, 240, 10, 219, 34, 198, 187,
];
