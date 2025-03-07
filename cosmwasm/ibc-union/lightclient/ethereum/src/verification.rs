use cosmwasm_std::{Deps, Empty, HashFunction, BLS12_381_G1_GENERATOR};
use ethereum_sync_protocol::{BlsVerify, DST_POP_G2};
use unionlabs::primitives::{H384, H768};

use crate::errors::Error;

pub struct VerificationContext<'a> {
    pub deps: Deps<'a, Empty>,
}

impl BlsVerify for VerificationContext<'_> {
    fn fast_aggregate_verify<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk H384>,
        msg: Vec<u8>,
        signature: H768,
    ) -> Result<(), ethereum_sync_protocol::error::Error> {
        let pubkeys = public_keys
            .into_iter()
            .flat_map(|x| *x)
            .collect::<Vec<u8>>();

        let pubkey = self
            .deps
            .api
            .bls12_381_aggregate_g1(&pubkeys)
            .map_err(|e| {
                ethereum_sync_protocol::error::Error::ClientSignatureVerification(e.to_string())
            })?;

        let hashed_msg = self
            .deps
            .api
            .bls12_381_hash_to_g2(HashFunction::Sha256, &msg, DST_POP_G2)
            .map_err(|e| {
                ethereum_sync_protocol::error::Error::ClientSignatureVerification(e.to_string())
            })?;

        let valid = self
            .deps
            .api
            .bls12_381_pairing_equality(
                &BLS12_381_G1_GENERATOR,
                signature.as_ref(),
                &pubkey,
                &hashed_msg,
            )
            .map_err(|e| {
                ethereum_sync_protocol::error::Error::ClientSignatureVerification(e.to_string())
            })?;

        if valid {
            Ok(())
        } else {
            Err(ethereum_sync_protocol::error::Error::Crypto)
        }
    }
}

/// Checks whether the public keys match the aggregate pubkey
pub fn check_aggregate_pubkey(
    deps: Deps,
    public_keys: &[H384],
    aggregate_pubkey: H384,
) -> Result<(), Error> {
    let pubkeys = public_keys.iter().flat_map(|x| *x).collect::<Vec<u8>>();

    let pubkey = deps.api.bls12_381_aggregate_g1(&pubkeys)?;

    if &pubkey != aggregate_pubkey.get() {
        return Err(Error::AggregatePubkeyMismatch);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use beacon_api_types::altair;
    use cosmwasm_std::testing::mock_dependencies;

    use super::*;

    #[test]
    fn test_check_aggregate_pubkey() {
        let sync_committee: altair::SyncCommittee = serde_json::from_str(
            r#"{"pubkeys":["0xa8d4c7c27795a725961317ef5953a7032ed6d83739db8b0e8a72353d1b8b4439427f7efa2c89caa03cc9f28f8cbab8ac","0xb7e6e187ed813d950a9a17d1e70c03e4de2903596c4c5ff326848515c985deee38198efebc265300cd4f1d6bd7b5d264","0x914b56f41c411fbfca9dc9763f44daf253c103b162457d07954fd0af768b5e74692b4639c22455fb81d71f7ed6144514","0x8b47707a1f563d3b1034e20be2a663587f17fece6581fca156cf660575fde4b8de4d45f1fda7ade9167b953d4c93417d","0xb89bebc699769726a318c8e9971bd3171297c61aea4a6578a7a4f94b547dcba5bac16a89108b6b6a1fe3695d1a874a0b","0x9314c6de0386635e2799af798884c2ea09c63b9f079e572acc00b06a7faccce501ea4dfc0b1a23b8603680a5e3481327","0xb0af0bfa83f0922e6cbfd2bc8ec19ff0f692fcb87c4e35f30e1353b342ae2fdaea6056bc2759970fc2a1f561826f564e","0x8bb9e1693eab1496d7583bf22fb1f2a475934c63b4d94118940617aa187bc277f738223e0ec1ce8a5566035d9bcc5470","0xb242e56475dca34fe92de09daee3951d647c04ed7a483a5c5c5613676f5ca88d54ec64d1aee81fb0f085aa67c88ee6db","0x8be11e9ead2e1bb5be7e2ec066ff83589558a5d9373666b3fc518a6a6639b3baecb87f8f34895f63e8d09d270d93ce04","0xafe6eface52fb6de91055a81abf9aa6e42ce2ef36fd8ae0d09aec6e5d8bd40a065dfccda6104af94df3f7a5854559ef4","0x87bbd5574c17dbf80463d11f812a77306f67913c510b1b234f5bd80478c7da8e69476cd6711cd1f4c0e228a4e2e99636","0xa3a32b0f8b4ddb83f1a0a853d81dd725dfe577d4f4c3db8ece52ce2b026eca84815c1a7e8e92a4de3d755733bf7e4a9b","0x83802cd575a3cea7e3e38fc1a73d94a9e4fdb999b8494e7929309c009d79a23edb1ba091ac02588f130e0585fb106540","0x8ce551755078927147bae52f683f962ca09cd68e2a14dc7444f98739fe5d27e3596314d78deedc87beb705bcf9532182","0x963528adb5322c2e2c54dc296ffddd2861bb103cbf64646781dfa8a3c2d8a8eda7079d2b3e95600028c44365afbf8879","0x88c141df77cd9d8d7a71a75c826c41a9c9f03c6ee1b180f3e7852f6a280099ded351b58d66e653af8e42816a4d8f532e","0x96746aaba64dc87835ba709332f4d5d7837ada092b439c49d251aecf92aab5dc132e917bf6f59799bc093f976a7bc021","0xa99a76ed7796f7be22d5b7e85deeb7c5677e88e511e0b337618f8c4eb61349b4bf2d153f649f7b53359fe8b94a38e44c","0xb41a0d9f8f19be13395aa09711b492d20eaf4a56d2360cd6daa2fd665532d852cb9224a5a39e5abff389882f961f12a6","0x96ef954b331a534199f4f113d993a50ec7a781fc5aa2a181ea0bdbfd4c5c557abfebfcc02604d5aef52ba64afbe0ff18","0xaecc56f2b1c4011d450214d3e1254479d583a6a5c2c06fbc049512731f76227d140df9f36a3f76b4ccb4df1342403573","0x9893413c00283a3f9ed9fd9845dda1cea38228d22567f9541dccc357e54a2d6a6e204103c92564cbc05f4905ac7c493a","0xb08f7feb86786c37661afb9951a959c9b465fd11ca98fcbc908fcf49144084051f6c363e2eb4459da2c2d03d84175692","0xad01d0f23cb74fcc4c39a2d0827d22f4722f02076196350dff5dcc6be765009c66e29001001959d77b277c2f0fba0425","0xa1c76af1545d7901214bb6be06be5d9e458f8e989c19373a920f0018327c83982f6a2ac138260b8def732cb366411ddc","0xb300303a03b8eff26a25449169d1946b208d5240f011ca6f5db23cd7f2c004b63f60afe3c9e047b67f9e4c8970c71cf0","0xae4d49364e4a36760cc74a675500055b9aed99bc19d31abb953ea156bb5a76dcf36769d15341b850114a30ffc8057780","0x8d58f7e2e58471b46d20a66a61f4cde3c78ab6c0505517c615e08d8ef5adf59b65fa2b01ea2395c84584a6f10d6cee2f","0xaa241b2afbb33f92a5d281aec9c8bac8997c1dddc051455fc0f334de48320f160b5029b552495aed21ed9ce252aab499","0x8a298ee1ac0466ecaa04d5798048c6e192409af63217f32fd7e07794cfcdcd8deca055b9782dd1ad45a578a9ec10606c","0x81283b7a20e1ca460ebd9bbd77005d557370cabb1f9a44f530c4c4c66230f675f8df8b4c2818851aa7d77a80ca5a4a5e"],"aggregate_pubkey":"0xb99ecc9ee5b6954058d69d0f4bb18e19b5732057133229919233dcfb5626efbc68b2d4fc6663063dfc762479520312bd"}"#,
        ).unwrap();

        assert!(check_aggregate_pubkey(
            mock_dependencies().as_ref(),
            &sync_committee.pubkeys,
            sync_committee.aggregate_pubkey,
        )
        .is_ok(),);
    }
}
