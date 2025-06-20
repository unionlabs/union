#![feature(array_chunks)]

use std::ops::{Mul, Sub};

use consensus_primitives::{Duration, Timestamp};
use parlia_types::{ParliaHeader, Valset, VoteAttestation, VoteData};
use unionlabs_primitives::{ByteArrayExt, H160, H384, H768, U256};

// post-maxwell
pub const TURN_LENGTH: u64 = 16;
pub const EPOCH_LENGTH: u64 = 1000;

// pre-maxwell
// pub const TURN_LENGTH: u64 = 8;
// pub const EPOCH_LENGTH: u64 = 500;

pub const EXTRA_SEAL_LEN: usize = 65;
pub const EXTRA_VANITY_LEN: usize = 32;
// const NEXT_FORK_HASH_SIZE: usize = 4;
pub const NEXT_TURN_LENGTH_SIZE: usize = 1;
pub const VAL_COUNT_SIZE: usize = 1;

pub const VAL_ENTRY_LEN: usize = <H160>::BYTES_LEN + <H384>::BYTES_LEN;

pub const EXTRA_DATA_MIN_LEN: usize = EXTRA_VANITY_LEN + EXTRA_SEAL_LEN;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ExtraDataDecodeError {
    #[error("invalid extra data len")]
    InvalidExtraDataLen,
    #[error("invalid turn length (found {0}, expected {TURN_LENGTH})")]
    InvalidTurnLength(u64),
    #[error("not enough validators present in extra data")]
    NotEnoughVals,
    #[error(transparent)]
    Rlp(#[from] rlp::DecoderError),
}

pub fn parse_epoch_rotation_header_extra_data(
    data: &[u8],
) -> Result<(VoteAttestation, Valset), ExtraDataDecodeError> {
    if data.len() <= EXTRA_DATA_MIN_LEN {
        return Err(ExtraDataDecodeError::InvalidExtraDataLen);
    }

    let data = &data[EXTRA_VANITY_LEN..(data.len() - EXTRA_SEAL_LEN)];

    let num = data[0];
    let vals = data[1..]
        .array_chunks::<VAL_ENTRY_LEN>()
        .map(|x| {
            // (address, pubkey)
            (
                x.array_slice::<0, 20>().into(),
                x.array_slice::<20, 48>().into(),
            )
        })
        .take(num.into())
        .collect::<Vec<_>>();

    if vals.len() != num as usize {
        return Err(ExtraDataDecodeError::NotEnoughVals);
    }

    let turn_length = data[VAL_COUNT_SIZE + (VAL_ENTRY_LEN * num as usize)];
    if turn_length as u64 != TURN_LENGTH {
        return Err(ExtraDataDecodeError::InvalidTurnLength(turn_length as u64));
    }

    let va = rlp::decode::<VoteAttestation>(
        &data[(VAL_COUNT_SIZE + (VAL_ENTRY_LEN * num as usize) + NEXT_TURN_LENGTH_SIZE)..],
    )?;

    Ok((va, Valset::new(vals)))
}

pub fn parse_header_extra_data(data: &[u8]) -> Result<VoteAttestation, ExtraDataDecodeError> {
    if data.len() <= EXTRA_DATA_MIN_LEN {
        return Err(ExtraDataDecodeError::InvalidExtraDataLen);
    }

    let data = &data[EXTRA_VANITY_LEN..(data.len() - EXTRA_SEAL_LEN)];

    let va = rlp::decode::<VoteAttestation>(data)?;

    Ok(va)
}

pub fn get_vote_attestation_from_header_extra_data(
    header: &ParliaHeader,
) -> Result<VoteAttestation, ExtraDataDecodeError> {
    if is_epoch_rotation_header(header) {
        parse_epoch_rotation_header_extra_data(&header.extra_data).map(|x| x.0)
    } else {
        parse_header_extra_data(&header.extra_data)
    }
}

pub fn is_epoch_rotation_header(header: &ParliaHeader) -> bool {
    header.number % U256::from(EPOCH_LENGTH) == U256::ZERO
}

pub fn calculate_signing_valset_epoch_block_number(h: u64, valset_size: u64) -> u64 {
    h.sub(TURN_LENGTH.mul(valset_size.div_ceil(2)))
        .sub(EPOCH_LENGTH)
        .div_ceil(EPOCH_LENGTH)
        .mul(EPOCH_LENGTH)
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error<E> {
    #[error("invalid attestation chain")]
    InvalidAttestationChain,
    #[error("invalid attestation")]
    InvalidAttestation,
    #[error(transparent)]
    ExtraDataDecode(#[from] ExtraDataDecodeError),
    #[error(transparent)]
    ContextError(E),
    #[error("trusted valset not found for block {0}")]
    TrustedValsetNotFound(u64),
    #[error("less than 2/3+1 of the valset signed the attestation")]
    InsufficientParticipation,
    #[error("block number > u64::MAX")]
    BlockNumberTooLarge,
    #[error(
        "provided {expected} as the expected trusted valset block number, \
        but the attestation was signed by the valset at block {found}"
    )]
    InvalidTrustedValsetEpochBlockNumber { expected: u64, found: u64 },
    #[error("the valset is not sorted")]
    ValsetNotSorted,
    #[error("the header is expired")]
    HeaderExpired,
}

pub trait VerificationContext {
    type Error: std::error::Error;

    fn current_timestamp(&self) -> Timestamp;

    fn get_valset(&self, epoch_block_number: u64) -> Result<Valset, Self::Error>;

    fn verify<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk H384>,
        msg: &[u8],
        signature: H768,
    ) -> Result<(), Self::Error>;
}

/// Given 3 headers: source `S`, target `T`, and attestation `A`, where `A` contains the vote data for `S` and `T`:
/// 1. ensure that `A` is not expired
/// 2. verify that `S ∈ T ∈ A`
/// 3. validate the signature contained in `A` with the valset that signed it
/// 4. if `S` is an epoch change block, return the epoch change block number and the new valset
pub fn verify_header<C: VerificationContext>(
    source: &ParliaHeader,
    target: &ParliaHeader,
    attestation: &ParliaHeader,
    unbond_period: Duration,
    trusted_valset_epoch_block_number: u64,
    ctx: C,
) -> Result<Option<(u64, Valset)>, Error<C::Error>> {
    // 1.
    if attestation
        .full_timestamp()
        .plus_duration(unbond_period)
        .is_none_or(|header_timestamp_plus_unbonding_period| {
            header_timestamp_plus_unbonding_period < ctx.current_timestamp()
        })
    {
        return Err(Error::HeaderExpired);
    }

    // 2.
    if source.number + U256::ONE != target.number || target.number + U256::ONE != attestation.number
    {
        return Err(Error::InvalidAttestationChain);
    }

    if target.parent_hash != source.hash() || attestation.parent_hash != target.hash() {
        return Err(Error::InvalidAttestationChain);
    }

    let vote_attestation = get_vote_attestation_from_header_extra_data(attestation)?;

    if (vote_attestation.data
        != VoteData {
            source_number: source
                .number
                .try_into()
                .map_err(|()| Error::BlockNumberTooLarge)?,
            source_hash: source.hash(),
            target_number: target
                .number
                .try_into()
                .map_err(|()| Error::BlockNumberTooLarge)?,
            target_hash: target.hash(),
        })
    {
        return Err(Error::InvalidAttestation);
    }

    let trusted_valset = ctx
        .get_valset(trusted_valset_epoch_block_number)
        .map_err(Error::ContextError)?;

    let epoch_block_number = calculate_signing_valset_epoch_block_number(
        attestation
            .number
            .try_into()
            .map_err(|()| Error::BlockNumberTooLarge)?,
        trusted_valset.len().try_into().unwrap(),
    );

    if trusted_valset_epoch_block_number != epoch_block_number {
        return Err(Error::InvalidTrustedValsetEpochBlockNumber {
            expected: trusted_valset_epoch_block_number,
            found: epoch_block_number,
        });
    }

    if vote_attestation.vote_address_set.count() as usize <= (trusted_valset.len() * 2) / 3 {
        return Err(Error::InsufficientParticipation);
    }

    let signing_valset = trusted_valset
        .iter()
        .enumerate()
        .filter(|(idx, _)| vote_attestation.vote_address_set.is_set(*idx));

    if !signing_valset.clone().is_sorted_by(|a, b| a.1 < b.1) {
        return Err(Error::ValsetNotSorted);
    }

    // 3.
    ctx.verify(
        signing_valset.map(|x| &x.1 .1),
        vote_attestation.data.hash().get(),
        vote_attestation.agg_signature,
    )
    .map_err(Error::ContextError)?;

    // 4.
    if is_epoch_rotation_header(source) {
        let (_, new_valset) = parse_epoch_rotation_header_extra_data(&source.extra_data)?;
        Ok(Some((
            source
                .number
                .try_into()
                .map_err(|()| Error::BlockNumberTooLarge)?,
            new_valset,
        )))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use hex_literal::hex;

    use super::*;

    struct BlstContext {
        current_timestamp: Timestamp,
        epoch_valsets: HashMap<u64, Valset>,
    }

    // will never be BLST_SUCCESS
    #[derive(Debug, Clone, thiserror::Error)]
    #[error("{0:?}")]
    pub struct BlstError(blst::BLST_ERROR);

    impl BlstError {
        fn new_result(e: blst::BLST_ERROR) -> Result<(), Self> {
            if e == blst::BLST_ERROR::BLST_SUCCESS {
                Ok(())
            } else {
                Err(Self(e))
            }
        }

        fn new(e: blst::BLST_ERROR) -> Self {
            Self::new_result(e).unwrap_err()
        }
    }

    impl VerificationContext for BlstContext {
        type Error = BlstError;

        fn current_timestamp(&self) -> Timestamp {
            self.current_timestamp
        }

        fn get_valset(&self, epoch_block_number: u64) -> Result<Valset, Self::Error> {
            Ok(self.epoch_valsets.get(&epoch_block_number).unwrap().clone())
        }

        fn verify<'pk>(
            &self,
            public_keys: impl IntoIterator<Item = &'pk H384>,
            msg: &[u8],
            signature: H768,
        ) -> Result<(), Self::Error> {
            let agg_sig =
                blst::min_pk::Signature::uncompress(signature.get()).map_err(BlstError::new)?;

            BlstError::new_result(
                agg_sig.fast_aggregate_verify(
                    true,
                    msg,
                    b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_",
                    &public_keys
                        .into_iter()
                        .map(|pkey| {
                            blst::min_pk::PublicKey::uncompress(pkey.get()).map_err(BlstError::new)
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .iter()
                        .collect::<Vec<_>>(),
                ),
            )
        }
    }

    #[track_caller]
    fn deser_header(json: &str) -> ParliaHeader {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn verify_header_works_testnet() {
        // 55475596
        let source = deser_header(
            r#"{"hash":"0x053e82c0a815b741826b338cb3fd1346a098ea0d69dcca5f56673731279be1db","parentHash":"0xdd0eaf7f4e73b61b6b232358e235ee879b8e321d7c8f976c4242db55275c80a2","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x90409f56966b5da954166eb005cb1a8790430ba1","stateRoot":"0x8cd0286815e0e5afc095bf6a1aed01e82e906f23b8f8536149a31e0d42c0b01c","transactionsRoot":"0x406e687cb846a88d971394b5f1927ef01287cd8c0a2cfe20817d345517f78d38","receiptsRoot":"0x90bb10b203eb8e2462bff1cc0a0055b20d74f7d31b354243dd1950b06cf30c80","logsBloom":"0x000008004000000000010040000020000020400000000000000002000000000000001080000000000000000000000000000800000000000000000000002000000040000400800000000000081000000022100002000000000000000000100002000810200002000000000000010000000a0000000000000000000010000000000000002000000000000000000002000001000400004000008400000000000020020000000000002008000000028000000000000000000000000000000000000000000002000000000000140000000000000000000000001000104002000000000010800000000000010000040000010000008000000000004100000000001000","difficulty":"0x2","number":"0x34e7d8c","gasLimit":"0x5e29eed","gasUsed":"0x83351","timestamp":"0x6855884c","extraData":"0xd88301050d846765746888676f312e32332e39856c696e757800000038469028f8b27fb860a721d42755113b5cbdac611e235d97b9119eb22a44b9e14c0be9db09726ca0e4daad7b0897608eba03e70d8bb516ed740257857fbaa2f7d2213eeba4f4481b62b8fe89de0ef1a562abc3ff84810b38647420407ebf0847d048148afbf523c051f84c84034e7d8aa06e1316485b6da89f3d4c0e434d9206488843a4a9e54462ab8746976e57822dfd84034e7d8ba0dd0eaf7f4e73b61b6b232358e235ee879b8e321d7c8f976c4242db55275c80a280f33ec24f7c83f578fcd505ae3a1b83c424f04098df9382b97cc81d5f28221d6b67c789880790fc9427d2e156bb9ad51eeb4efa479fa429cf4f1f11263a3cbdd700","mixHash":"0x00000000000000000000000000000000000000000000000000000000000000fa","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x0","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6993713","size":"0x681","uncles":[],"transactions":[{"type":"0x0","chainId":"0x61","nonce":"0xd1","gasPrice":"0x66758af","gas":"0x6b155","to":"0x58cfe0e9e376981c1b7ba7cd2673a08f60690625","value":"0x0","input":"0x599682880000000000000000000000000000000000000000000000000e8b3e728c9c000000000000000000000000000000000000000000000000000014d1120d7b16000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000005d868f081b9da000000000","r":"0x5e6b0334111df188c74ef165b6f44a640e250f6ff1a14783c02f3ca9d98bf53d","s":"0x38f233b657879682c0a782e1c50c36b0f282e4475e31fc655218553bdbda70bf","v":"0xe6","hash":"0xca6df32828acd652070f3751a83af313c57bc8095115d6631f93948d200371a8","blockHash":"0x053e82c0a815b741826b338cb3fd1346a098ea0d69dcca5f56673731279be1db","blockNumber":"0x34e7d8c","transactionIndex":"0x0","from":"0x2dfd57da36f1ff5c11fa5047c7843ad5ddc28e35"},{"type":"0x0","chainId":"0x61","nonce":"0x35c","gasPrice":"0x64babf8","gas":"0x73c04","to":"0x64e6416d5ef820f23e09c53adf6ac4ab061a305d","value":"0x0","input":"0x46322c3700000000000000000000000000000000000000000000000000000000000006ef0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000","r":"0x996409d184b4c8126f89203f52d660c629ad201edfee40be1257bbb60d5448a1","s":"0x11a3f508ccf29805e30a9257d2c6abd8123cc2cea720023a2276728a60017d34","v":"0xe6","hash":"0x0432ec67be63215640d00b338ddd5b58a78b5c65bd069d2434737f39030ed882","blockHash":"0x053e82c0a815b741826b338cb3fd1346a098ea0d69dcca5f56673731279be1db","blockNumber":"0x34e7d8c","transactionIndex":"0x1","from":"0xdfd0b7f8ad5ceb600acac6524395c2886f4a11a8"},{"type":"0x0","chainId":"0x61","nonce":"0x950","gasPrice":"0x5f5e100","gas":"0x67ae","to":"0x14fb578f334bd6cfa80dedcabcc173620d56801c","value":"0x0","input":"0x095ea7b3000000000000000000000000d99d1c33f9fc3444f8101754abc46c52416550d1000000000000000000000000000000000000000000084595161401484a000000","r":"0x903d63ec55f53957346833f32b63ee4015021e442b0df9a3295f74883cd27e0a","s":"0x15ba87bb025c4db7aba180aad2bfd55652c3dfbc152f22020896c3b60d7d9e64","v":"0xe5","hash":"0x8cba86a8ace29d59fbe0058a2b6993f09408d8ebfc3e4931ea7f28ab2f5f21d6","blockHash":"0x053e82c0a815b741826b338cb3fd1346a098ea0d69dcca5f56673731279be1db","blockNumber":"0x34e7d8c","transactionIndex":"0x2","from":"0x2256be9c1832750fce12f309638e55cb26714607"},{"type":"0x0","chainId":"0x61","nonce":"0x5851f","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x2e9cc0fb2d84","input":"0xf340fa0100000000000000000000000090409f56966b5da954166eb005cb1a8790430ba1","r":"0x360f39b1f54a9187df0212129971ee10dfd9be23c639882dba33c8df4c87273d","s":"0x50d56bd784de7546e3808f9c0bcb424de42986dde80aa1f1d746940c62349395","v":"0xe6","hash":"0x5a5581d8d9dbd5ad3ac6f8ad1674140ef9bb77e47d77690d974049ac9dc74b92","blockHash":"0x053e82c0a815b741826b338cb3fd1346a098ea0d69dcca5f56673731279be1db","blockNumber":"0x34e7d8c","transactionIndex":"0x3","from":"0x90409f56966b5da954166eb005cb1a8790430ba1"}],"withdrawals":[],"milliTimestamp":"0x1978e1c69da"}"#,
        );
        let target = deser_header(
            r#"{"hash":"0xc28a31508a6f9c7376bc49e9b28529e3316dbc4e23205eacfb552ca4c7aecc97","parentHash":"0x053e82c0a815b741826b338cb3fd1346a098ea0d69dcca5f56673731279be1db","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x90409f56966b5da954166eb005cb1a8790430ba1","stateRoot":"0x1677499391845cd0e4913642a3f0353063771425a85ce440b3682e12b2c06757","transactionsRoot":"0xd0dd7a6c34eed0e99104d34f21a11aa2cbe1cad2d7c2677db297fe921b870d6f","receiptsRoot":"0x0dde1b798eff095db7ca4693cb1203d74da3430698e9acdc914fa7513e4d2127","logsBloom":"0x00000000400000000000004000002000000000000800000000000200000000000000108000000000000000000000000010000000000000000000000000200000000000000080100000000008100000002010000000000000000000000000000200080020000200000000000001000000080400000010000000000010000000000000000000000000000000000002000000000400000000008000000000000020020000002000002008000000020000000000000000000000008000000000000000000002000000000004140000000000000000000000001000104002000000000010000000000000010000040000010000008000000000004101000000001000","difficulty":"0x2","number":"0x34e7d8d","gasLimit":"0x5e12647","gasUsed":"0x5c027","timestamp":"0x6855884d","extraData":"0xd88301050d846765746888676f312e32332e39856c696e757800000038469028f8b27fb86097ecebb7fe6367c2f85d8fae3eeef6b0f2d1fe572a8671e078259179fe7d72e6b216eb8ce017b281bcb5e7c837f4fa41095b7c7f8089be29478774536b76f5dc63f404b943f184f8c58fc326979fc8dfebc3802cd8f762e1c2a4b169bbcdacaef84c84034e7d8ba0dd0eaf7f4e73b61b6b232358e235ee879b8e321d7c8f976c4242db55275c80a284034e7d8ca0053e82c0a815b741826b338cb3fd1346a098ea0d69dcca5f56673731279be1db806e89d76116b905e9585ed4a1870491457cd08f1cfad09fcb89c0557bf5e7d7f420b8c480a6dfac71c9ac4454b459041626e4f539e912ccf5bf6392733307dcbc00","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x0","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6993715","size":"0x5db","uncles":[],"transactions":[{"type":"0x0","chainId":"0x61","nonce":"0x412","gasPrice":"0x12a05f200","gas":"0x5208","to":"0x6d31393d6d39bb7324d008053222c4b57229942d","value":"0x5af3107a4000","input":"0x","r":"0xe7f6d307d90fc8c745d8c3dbf2daddc96c6d11530f403e7662f5545f7b1a4b61","s":"0x1e169e4d746f6e2c2c1d4e80a7d5f8cd42d8411b78a98b6305318a10051d0e6f","v":"0xe5","hash":"0x45979f1f5e0c5aff34e4d2d2e3725c8d1b98b3a61b8279ba78ff1d3b72d71a93","blockHash":"0xc28a31508a6f9c7376bc49e9b28529e3316dbc4e23205eacfb552ca4c7aecc97","blockNumber":"0x34e7d8d","transactionIndex":"0x0","from":"0xcdd8a8ee2122310c6434d4cfb3933918b6dcb0bc"},{"type":"0x0","chainId":"0x61","nonce":"0x346","gasPrice":"0x65b747e","gas":"0x78595","to":"0x29f0852abbe23137e7d15dd64893c5ae4b05bce9","value":"0x0","input":"0x599682880000000000000000000000000000000000000000000000000f43fc2c04ee00800000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000a816a097cd0180","r":"0x6a60afe2f30489cd1d1ed1d4d48ac38f195c91dc92f272a571c925eaf5845e4a","s":"0x145d4be0481385e300ee3df3c89ba1bc93d2537e0f3996db4b8b637e665d2ada","v":"0xe5","hash":"0x610ef752f97a5f820ad0d3eb16f368e7cd3eff346cdd203051d78e340c6753a1","blockHash":"0xc28a31508a6f9c7376bc49e9b28529e3316dbc4e23205eacfb552ca4c7aecc97","blockNumber":"0x34e7d8d","transactionIndex":"0x1","from":"0xc9c9496193ad1607a8371f4b4ce365dd3056150e"},{"type":"0x0","chainId":"0x61","nonce":"0x1aa02","gasPrice":"0x0","gas":"0x61a8","to":"0x0eabbde133fbf3c5eb2bee6f7c8210deeaa0f7db","value":"0x0","input":"0x","r":"0x69debde7d8b1f852aba327db6888fd31acd2b88c2386e2afa84df79a15f0efea","s":"0x7ac09956ee3ed8785d88d6466cf4ddba90acbf02b225f1fb67c6163d66a21223","v":"0xe6","hash":"0x1fd653ae7db969984d06301553b528211f54851532794e3696e6a2ba703b843d","blockHash":"0xc28a31508a6f9c7376bc49e9b28529e3316dbc4e23205eacfb552ca4c7aecc97","blockNumber":"0x34e7d8d","transactionIndex":"0x2","from":"0xa6cbb1e75d42b3abae7c419e5ce5ceb2b0b2cee7"},{"type":"0x0","chainId":"0x61","nonce":"0x58520","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x7a9385804996","input":"0xf340fa0100000000000000000000000090409f56966b5da954166eb005cb1a8790430ba1","r":"0x7942e70021764570ec282d57d564c06f2246c9922200867e819440a3a31000c5","s":"0x4fbfd8a45e399c0d14d041a79f35a08a45922ea87834cbefbbf77e8f5f6164a5","v":"0xe5","hash":"0x7691e72945986ea463becc553cda5dccc32dabbf2154ee103b2cfccae3b2e598","blockHash":"0xc28a31508a6f9c7376bc49e9b28529e3316dbc4e23205eacfb552ca4c7aecc97","blockNumber":"0x34e7d8d","transactionIndex":"0x3","from":"0x90409f56966b5da954166eb005cb1a8790430ba1"}],"withdrawals":[],"milliTimestamp":"0x1978e1c6cc8"}"#,
        );
        let attestation = deser_header(
            r#"{"hash":"0x03b1077518e32f66c8f5723660f2fae0fe35995b85cccaa3f74192a00ce5714c","parentHash":"0xc28a31508a6f9c7376bc49e9b28529e3316dbc4e23205eacfb552ca4c7aecc97","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x90409f56966b5da954166eb005cb1a8790430ba1","stateRoot":"0x7810faeda27cf9c2b482a323b6b0daf1b091da489dfea387299291097ec63841","transactionsRoot":"0x558ca0974f34670318eebfee8d1b4297c5833b3b46a420fb5e7b1b8368061c20","receiptsRoot":"0xf2a9868aa55b9876f43acfe14f47615f0ac8326a073cc51d509e39460da2dea0","logsBloom":"0x00000000400000000000004000002800000000000000000000000280000000200000108000000000000000000010000000000000000000000000000000200000004000000080008000000008100000002010000000000000000000000000000200080020000200000000000001000000080000000000000000000010000000000000000000000000000000002002200008000400000000008000000000000020020000000000002008200040020000000000000000000000000000000000000000000002000000000000140000000000000000000000003100104002000000000010000000200000010000040000010010008000000000004100000000001000","difficulty":"0x2","number":"0x34e7d8e","gasLimit":"0x5dfadff","gasUsed":"0x62181","timestamp":"0x6855884d","extraData":"0xd88301050d846765746888676f312e32332e39856c696e757800000038469028f8b27fb86096617b60a045db74aa4db769a554c4e5c7629a78b8f8b93534fca11d1cc6602c51c3686dd8c07810d28c4b84e894598d02063576968638fdd715e3c5bb6a6236df3cd8fb66a48ee7994a2ad746f10f77f1cb031976d1258ab199aeb60d7694f3f84c84034e7d8ca0053e82c0a815b741826b338cb3fd1346a098ea0d69dcca5f56673731279be1db84034e7d8da0c28a31508a6f9c7376bc49e9b28529e3316dbc4e23205eacfb552ca4c7aecc9780795dadc6947ae45cd4cde727de61d5755ba6da14d26d4192a2709fc333f94d5640780856238690e3b2aea2c97335495713a26878f6839b6fdb6bf8a154b67a5700","mixHash":"0x00000000000000000000000000000000000000000000000000000000000002ee","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x20000","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6993717","size":"0x64b","uncles":[],"transactions":[{"type":"0x0","chainId":"0x61","nonce":"0x2a","gasPrice":"0x7270e00","gas":"0x1134e","to":"0xb880e767739a82eb716780bdfdbc1ed7b23bdb38","value":"0x0","input":"0x095ea7b3000000000000000000000000b4f72ffd54b7090b6da8ead9dd689b5ba93ccdf50000000000000000000000000000000000000000000000007394bd33d9bf56af","r":"0xa976cfa61d4244d9c5c81e240c4aded3f8556530d4b235db2749a937a863ecc5","s":"0x41c7084fb97387ea9144b6e2f1d2d50ef31786cb85c45fb1b6dc1c60a3f3c651","v":"0xe6","hash":"0x48b581d213ca82678a4608fe24daa8e6859a8c62626c23b6ab857430a95a2971","blockHash":"0x03b1077518e32f66c8f5723660f2fae0fe35995b85cccaa3f74192a00ce5714c","blockNumber":"0x34e7d8e","transactionIndex":"0x0","from":"0xddb54a0803d573ffd205d6150db1cc5947868d5e"},{"type":"0x3","chainId":"0x61","nonce":"0x26e3e0","gas":"0xcf08","maxFeePerGas":"0x7270e00","maxPriorityFeePerGas":"0x7270e00","to":"0xff00000000000000000000000000000000005611","value":"0x0","accessList":[],"blobVersionedHashes":["0x0152d68a1558fe4ac4942bd3533e3a4efb59f9b62004088ca8cec68e3ae288e3"],"maxFeePerBlobGas":"0x1","input":"0x","r":"0x38985ff35396e4341dc285c94004c8571c8a9765aaa8b43770c0d72101ad4f42","s":"0x53c1d91b52664d753f6078c990545671b765f9ecd3d261e6e0795cee03a388b8","yParity":"0x0","v":"0x0","hash":"0xe869eb7c73ec9660d4941c6c3ceb69b0ed3c5b76aea46e13ea82929edf534777","blockHash":"0x03b1077518e32f66c8f5723660f2fae0fe35995b85cccaa3f74192a00ce5714c","blockNumber":"0x34e7d8e","transactionIndex":"0x1","from":"0x1fd6a75cc72f39147756a663f3ef1fc95ef89495","gasPrice":"0x7270e00"},{"type":"0x0","chainId":"0x61","nonce":"0x770","gasPrice":"0x6616cd5","gas":"0x6c1a0","to":"0xe886b759c7811052ef54ccbc7359766a134211fb","value":"0x0","input":"0x599682880000000000000000000000000000000000000000000000000e3d1590638d0000000000000000000000000000000000000000000000000000120a871cc002000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000005809ffdc73c0d000000000","r":"0xb163e64dd4f69352c34db480c8a8ed5776939bb8b398d9bcebb9b465fe669af0","s":"0x377182cf15c8ad4003e4c5ed2a44781286e4944e6d5833c071f8db6a232bdcf4","v":"0xe5","hash":"0x6684bd2c6afb178ae9483bcd0f07ea37d6c9dcf5c787e6eb411e77a867877a40","blockHash":"0x03b1077518e32f66c8f5723660f2fae0fe35995b85cccaa3f74192a00ce5714c","blockNumber":"0x34e7d8e","transactionIndex":"0x2","from":"0xcd306fc33bad86f292c21839dde392abf2931a23"},{"type":"0x0","chainId":"0x61","nonce":"0x58521","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x227cb1fc3d31","input":"0xf340fa0100000000000000000000000090409f56966b5da954166eb005cb1a8790430ba1","r":"0x250bf937857ea24636025eb6aa08a0ec404e6fcc3fb2b507303661fff9b29761","s":"0x3e1ceaf0672a04d6b246d8f14a6d788ebd403b43cf8261ce685954d0dcc7245c","v":"0xe6","hash":"0xcaa683634e75bf06c8925412b890cd0e4ad52227b08d51b1863f50b2f4c836b4","blockHash":"0x03b1077518e32f66c8f5723660f2fae0fe35995b85cccaa3f74192a00ce5714c","blockNumber":"0x34e7d8e","transactionIndex":"0x3","from":"0x90409f56966b5da954166eb005cb1a8790430ba1"}],"withdrawals":[],"milliTimestamp":"0x1978e1c6fb6"}"#,
        );

        let (_, valset) = parse_epoch_rotation_header_extra_data(&hex!("d883010510846765746888676f312e32332e39856c696e7578000000384690280708265da01e1a65d62b903c7b34c08cb389bf3d9996f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c21a3d9d7a717d64e6088ac937d5aacdd3e20ca963979974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd99850023053387f3321fd69d1e030bb921230dfb188826affaa39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b76d76ee8823de52a1a431884c2ca930c5e72bff3803af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea99e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf90409f56966b5da954166eb005cb1a8790430ba1962a2342bac4831c6de73fcb77ad08669aaaa0a2ba6c6973a02b8928dbe573d17864e48c3521f238ace1c16e160bb7f5d447b49cd040d20bc21e49ffea6487f5638e4346ad9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb10f8b27fb860a18a293898150b2c5c7112c7cd482121e3a50f30b03a4fa9bb39cac2397b2058089eb019af04dc6c43a59189f6a438400fc612012b27a9df59a91531b3cc36b75dbdd51008f61a7bf4a0c7da5fe2b7df9eefa6e9c42b9dedca7f32bcb6edc907f84c84034e7b36a0d40e88dd0303dd4ca0772ba4e9c5d36b82f4e526bd294f2cd98b1b45886a60ea84034e7b37a0def92aee98a20bf20f2c7a6ddc5807700a95353c75c2f7b9329c9be2d1e6a6c480c692716571c53a20a0de653cefffd0da1d1c092d01e47a96daa1195431420b453cb0e01350b790d8523a7c3ef4edffeb776187c57e1c96a06ebc582baabc889301")).unwrap();

        let res = verify_header(
            &source,
            &target,
            &attestation,
            Duration::from_secs(604800),
            55475000,
            BlstContext {
                current_timestamp: Timestamp::from_nanos(1750435993000000000),
                epoch_valsets: [(55475000_u64, valset)].into_iter().collect(),
            },
        )
        .unwrap();

        assert_eq!(res, None);
    }

    #[test]
    fn verify_header_works_valset_rotation_testnet() {
        // 55475000
        let source = deser_header(
            r#"{"hash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","parentHash":"0xdef92aee98a20bf20f2c7a6ddc5807700a95353c75c2f7b9329c9be2d1e6a6c4","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x76d76ee8823de52a1a431884c2ca930c5e72bff3","stateRoot":"0x2252dded282328d3057a456ee699c2e68f4ee62dbdb0caab780a7a68946a4b9f","transactionsRoot":"0xd6248fa40389110b461656d1d2d466a15d5fde590d2da9c5a12252bf8f9c672c","receiptsRoot":"0x9933ab2eae9c3cf424fccfec480ecafed7dc7507c4d97a9a1ea526337af8506c","logsBloom":"0x040000004000040000000040000020000a0000000000001000000200020000000000108000000000008030400800000000000000000001000000000020200000000001000081000100000008100000a021100000000000000040000000000002000800208202000100000000010008000800240000000000002000100000000000000820000040800200000000060000000004000000000280000000000000200200000000000220080000000280000004000000000010100000000000000000008000020000000000001420000000004000000000000250001048020000240040100000000000000110008c0000010500008000008000004100208000081000","difficulty":"0x2","number":"0x34e7b38","gasLimit":"0x5f5e100","gasUsed":"0x14a393","timestamp":"0x6855868d","extraData":"0xd883010510846765746888676f312e32332e39856c696e7578000000384690280708265da01e1a65d62b903c7b34c08cb389bf3d9996f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c21a3d9d7a717d64e6088ac937d5aacdd3e20ca963979974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd99850023053387f3321fd69d1e030bb921230dfb188826affaa39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b76d76ee8823de52a1a431884c2ca930c5e72bff3803af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea99e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf90409f56966b5da954166eb005cb1a8790430ba1962a2342bac4831c6de73fcb77ad08669aaaa0a2ba6c6973a02b8928dbe573d17864e48c3521f238ace1c16e160bb7f5d447b49cd040d20bc21e49ffea6487f5638e4346ad9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb10f8b27fb860a18a293898150b2c5c7112c7cd482121e3a50f30b03a4fa9bb39cac2397b2058089eb019af04dc6c43a59189f6a438400fc612012b27a9df59a91531b3cc36b75dbdd51008f61a7bf4a0c7da5fe2b7df9eefa6e9c42b9dedca7f32bcb6edc907f84c84034e7b36a0d40e88dd0303dd4ca0772ba4e9c5d36b82f4e526bd294f2cd98b1b45886a60ea84034e7b37a0def92aee98a20bf20f2c7a6ddc5807700a95353c75c2f7b9329c9be2d1e6a6c480c692716571c53a20a0de653cefffd0da1d1c092d01e47a96daa1195431420b453cb0e01350b790d8523a7c3ef4edffeb776187c57e1c96a06ebc582baabc889301","mixHash":"0x00000000000000000000000000000000000000000000000000000000000000fa","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x0","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x699326b","size":"0xdf3","uncles":[],"transactions":[{"type":"0x0","chainId":"0x61","nonce":"0x35a","gasPrice":"0x12a05f200","gas":"0x5208","to":"0x78ad73abe074d4d5ceb6f0acf67335dfae23a31f","value":"0x5af3107a4000","input":"0x","r":"0x879d83b323e5cada68494676c4f7f80c784144dce71b014646f3d54d705ff9f2","s":"0x60ddf6ae632ca3995b36cfff3020d1ac47333664ff82e8307dc0ffeec428b581","v":"0xe6","hash":"0xa83e1cf7a118a29fc61a94362f9409de537b5414863770dbbd3b2462b9d27758","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x0","from":"0xcdd8a8ee2122310c6434d4cfb3933918b6dcb0bc"},{"type":"0x0","chainId":"0x61","nonce":"0x35b","gasPrice":"0x12a05f200","gas":"0x61a8","to":"0x94aa5b00d0ea27db154e51d4314cac346419414d","value":"0x5af3107a4000","input":"0x","r":"0xc1b9db63495e291df655dd7a5a19774b776b99b207fa34f74822df46a6712d15","s":"0x129513928070f7fac6180b555c235eb581811f871623823d5a20c815fe4231f0","v":"0xe5","hash":"0x3b8df6a0b99c1c852dbfeedfa07cacd789dec9d9e58227f1b68475a42d961d27","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x1","from":"0xcdd8a8ee2122310c6434d4cfb3933918b6dcb0bc"},{"type":"0x0","chainId":"0x61","nonce":"0x35c","gasPrice":"0x12a05f200","gas":"0x61a8","to":"0x089347f74ea5380691e1364b021501f269eeec80","value":"0x5af3107a4000","input":"0x","r":"0x8fdcbd2e339416bdfc17ff47cd10e1ca1a324d19a7e5e1d3e0aca590e0668693","s":"0x584b0af1178ded4e88ccf42695eb47d19068c487140dc8fd430abf2cc3078934","v":"0xe6","hash":"0x6631e384b052b679fdbf9e3c26df427a2faa1ef51c87d18087a5438ef6540bfb","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x2","from":"0xcdd8a8ee2122310c6434d4cfb3933918b6dcb0bc"},{"type":"0x0","chainId":"0x61","nonce":"0x773","gasPrice":"0x64c04d4","gas":"0x6c1a0","to":"0x8fc864c4238c8642cd1d9bd5f8c27f18f2248521","value":"0x0","input":"0x5996828800000000000000000000000000000000000000000000000011fc51222ce8000000000000000000000000000000000000000000000000000026db992a3b18000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","r":"0xce5a894726c09700ff16c90e881b384ec983794e55cef132fcbe458cb2d58776","s":"0x2e06420826b0044730f716bdcbebd4a0acba3a9576bbec480b69b29b28b0ef68","v":"0xe5","hash":"0x0727f28761c758f8ffc66e3eb7498491e60accac9b859590a307a690da1e2a80","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x3","from":"0x691242054044f224eb1026896041cfb448dfa61b"},{"type":"0x0","chainId":"0x61","nonce":"0x3b5","gasPrice":"0x6478d49","gas":"0x73c04","to":"0xe278196f5ba680ba4dc9f0c627cc3e589c6bd6c9","value":"0x0","input":"0x46322c37000000000000000000000000000000000000000000000000000000000000574f0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000","r":"0x7ecc0cb3a9a8a6d16afdcfa06c09b2d06243b64921641a60d68d27f0ce88c84d","s":"0x6cb8bd653a147c9a3b54b56a3da563857e8aee041efcbedafde95a7061dc5d91","v":"0xe6","hash":"0x21bb49d2c5697a4f5603efa088c36f718f57b7d424861b4d7845cb0cb1ef78d1","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x4","from":"0x5ebc0c5099719259c995e823f42e6675fa826554"},{"type":"0x0","chainId":"0x61","nonce":"0x619","gasPrice":"0x5f5e100","gas":"0x452dc","to":"0xe278196f5ba680ba4dc9f0c627cc3e589c6bd6c9","value":"0x0","input":"0x59968288000000000000000000000000000000000000000000000000016345785d8a00000000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000028834bc7e605bab","r":"0xe670f6f123599674c74921132bf9eb643b4caaa4223c896f41a1a2bc62d793e8","s":"0xaf6121a18e2c71771b5e2b8dd1715750366f3f0899fa60e95d6dc25d0d735cc","v":"0xe5","hash":"0xe1a3c9b2ba0046726f24a604092c234121a60a858dc9f9637775fdb2d27d08c9","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x5","from":"0x9854a34183a0516d47db9ba12aea570ef96b7aba"},{"type":"0x0","chainId":"0x61","nonce":"0x4b9","gasPrice":"0x65f5249","gas":"0x78595","to":"0x009966e81bc03d412052e75f3b281754e0f4c7b9","value":"0x0","input":"0x599682880000000000000000000000000000000000000000000000000ff92c66d87980000000000000000000000000000000000000000000000000000f43fc2c04ee008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000adf0e4533d5f180","r":"0x67749c2dbf7240bab74f8db19f557797ea719075ec674d6258273985f006e5f0","s":"0x43fbd74748aa064e6b352534d2e362daf136add2bbab11a492ac5e9d95f6cc4d","v":"0xe6","hash":"0x389c2e801184d8cfd1ecf155e855a7e1d5bcafe950f73c847142a25a6489d5ad","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x6","from":"0x0811aa33d5ee210008d72498f7aa3b5cf9c01d1b"},{"type":"0x0","chainId":"0x61","nonce":"0x212f7","gasPrice":"0x0","gas":"0x61a8","to":"0x48b2665e5e9a343409199d70f7495c8ab660bb48","value":"0x0","input":"0x","r":"0x607651fdf34c72d1e480d77e389092db85c03568b392faaddce2139ad8050a23","s":"0x68c68d794db6dd178b9b3b9dcbd8da88e7ece5f95424c70c55833666f0ddfb94","v":"0xe6","hash":"0x119bd8a04c799ea8df653a82758a5b8296e8c54f96450f4327bf60bf59b864f2","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x7","from":"0x130bd4c3ca083e81e9e6f8e9408da88aaf32e36f"},{"type":"0x0","chainId":"0x61","nonce":"0x1f8d81","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x186d18d1b8142","input":"0xf340fa0100000000000000000000000076d76ee8823de52a1a431884c2ca930c5e72bff3","r":"0xb195b50d5984d162ec4fcd01bacf328e9df65dfcd3a9cb75414eb3654ea00fb9","s":"0x683b6c9c34417528796ce0134c8133549c80a5820a3595c26d34350ba17507fb","v":"0xe5","hash":"0x6ece9a48c24480dc36b9cd7a84fc108dfa5264df68df294ab1f395855e2cb9a4","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x8","from":"0x76d76ee8823de52a1a431884c2ca930c5e72bff3"},{"type":"0x0","chainId":"0x61","nonce":"0x1f8d82","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x0","input":"0x300c356700000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000000700000000000000000000000008265da01e1a65d62b903c7b34c08cb389bf3d990000000000000000000000001a3d9d7a717d64e6088ac937d5aacdd3e20ca96300000000000000000000000053387f3321fd69d1e030bb921230dfb188826aff00000000000000000000000076d76ee8823de52a1a431884c2ca930c5e72bff30000000000000000000000007f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea00000000000000000000000090409f56966b5da954166eb005cb1a8790430ba1000000000000000000000000d447b49cd040d20bc21e49ffea6487f5638e4346000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000001020000000000000000000000000000000000000000000000000000000000000103000000000000000000000000000000000000000000000000000000000000010600000000000000000000000000000000000000000000000000000000000000f700000000000000000000000000000000000000000000000000000000000000e500000000000000000000000000000000000000000000000000000000000001040000000000000000000000000000000000000000000000000000000000000103","r":"0xef3b3385cbf542fc07a57218890f362704fa736b2d0c1a09f3b06e3882111d1b","s":"0x15fd16e1b179e1989c08a605dd6221b340ba7ce6ba0ed24cac3ac6095f0f4886","v":"0xe6","hash":"0x791c62ca692201dd12fd76e61ec99c8cd6a74bf47ba44b172ccc39943f8b4716","blockHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","blockNumber":"0x34e7b38","transactionIndex":"0x9","from":"0x76d76ee8823de52a1a431884c2ca930c5e72bff3"}],"withdrawals":[],"milliTimestamp":"0x1978e1597c2"}"#,
        );
        let target = deser_header(
            r#"{"hash":"0xe01078f0512466d0eb277e99de0c78756546768e2ac904ba3f218c726e013c9c","parentHash":"0xaeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f19225","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x76d76ee8823de52a1a431884c2ca930c5e72bff3","stateRoot":"0xe22e98d8250109d6b6583387d9f9dfccf62f099be6af84849da73287a9afca2b","transactionsRoot":"0x7776cbefdcb7ccd45ba8008813eda868b404cdc044eaa9d50d5d44f13dd1a809","receiptsRoot":"0xc1bdcab4a5b74f334b3200241b17ede1ed068933187081946abae4f3e83ecd00","logsBloom":"0x00040000000024000000004000002200080000000000000000000200000000000000108000000000000000000000000000000000000000000000000000000000000000000000000000000008000000002010000000000000000000000000100200080020120200010000000001000800080004000000000000000010000000000020002000000000040000000002000000000400000000028000000000000020000000000000002008000000020000000000000020000000000000000000000000000002000000000020100000000000000080000000001100384002000020000000000000000000010000040000010000008000002000004000000800001000","difficulty":"0x2","number":"0x34e7b39","gasLimit":"0x5f5e100","gasUsed":"0x910b6","timestamp":"0x6855868e","extraData":"0xd883010510846765746888676f312e32332e39856c696e757800000038469028f8b27fb860ab01f151428ea696802a2b626fb0e9598c10ccc06bb6e8c4aed9bf6f789c35d300903c2b5db7fe71c0529474ed39cbbb09b938a80eb26801ca8a1db7197294b2f839cbd9aa033cac19adef4f0b2b02e463431b0418b417b10acdd0797ca66d3df84c84034e7b37a0def92aee98a20bf20f2c7a6ddc5807700a95353c75c2f7b9329c9be2d1e6a6c484034e7b38a0aeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f1922580b8f5f7b3eb35fe7c81590b2913b453a1f49df5eea0fa8b31b9ee64844659a474629acb0618e1abc9c643669630b542ee94274601f0976eb2cab1aa8886fb1a2d00","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x0","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x699326d","size":"0x5b2","uncles":[],"transactions":[{"type":"0x0","chainId":"0x61","nonce":"0x77","gasPrice":"0x7270e00","gas":"0x62feb","to":"0x4d3aeb975d0178acf62cbb0d11b8ac30671ac003","value":"0x0","input":"0x46322c3700000000000000000000000000000000000000000000000000000000000055250000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000bccf187ce8ca709a","r":"0x2aa92f6e8dded3c34858ae948633f8eade40508f8f097f51ceedbd92dbe60368","s":"0x41c8a2386aaef2ec81a4130994bd2f1ac0a08f58734983366552d87a87902cd1","v":"0xe6","hash":"0x19f1d85ea608482e7b3a8fbdbaadbe51762030517c00329c94931c6e12ce5583","blockHash":"0xe01078f0512466d0eb277e99de0c78756546768e2ac904ba3f218c726e013c9c","blockNumber":"0x34e7b39","transactionIndex":"0x0","from":"0x79d5a1205098637d4dff32f4fc374572de5cf27d"},{"type":"0x0","chainId":"0x61","nonce":"0x3b4","gasPrice":"0x6502cc7","gas":"0x6f4c1","to":"0xc89c7066b6f1ff3a261738b95b1e419fe04c10a9","value":"0x0","input":"0x46322c3700000000000000000000000000000000000000000000000000000000000062570000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000","r":"0x297703f888ad5ed7d978e5a764ab928dc379ef39f619f99c5c073e8cb5f33743","s":"0x190e1bdc2d239d5beb6feb0e184051832fd42745d960a8be07a5320df9bdf5fc","v":"0xe5","hash":"0x71a76700da0a86eeb3775747fabc590cb7ec803ed58336acc0e0425f18d8910d","blockHash":"0xe01078f0512466d0eb277e99de0c78756546768e2ac904ba3f218c726e013c9c","blockNumber":"0x34e7b39","transactionIndex":"0x1","from":"0x3cfbdc54c2804c82fb2d219b68a2ae040572f1e6"},{"type":"0x0","chainId":"0x61","nonce":"0x1f8d83","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x3740bec3f02d","input":"0xf340fa0100000000000000000000000076d76ee8823de52a1a431884c2ca930c5e72bff3","r":"0x9f2118f8d90a6ffad3a60351c8905383248501fd580b08fc9f435216cd17c56e","s":"0x67d2e9594f4a7243c5ab2f8414e2394f6b5f400e5969e5d7de7472abd344bfa1","v":"0xe6","hash":"0x8dc1fa2b39f9f53d93855d29e3d35040058110628122ea8218a020bbb13c7ca3","blockHash":"0xe01078f0512466d0eb277e99de0c78756546768e2ac904ba3f218c726e013c9c","blockNumber":"0x34e7b39","transactionIndex":"0x2","from":"0x76d76ee8823de52a1a431884c2ca930c5e72bff3"}],"withdrawals":[],"milliTimestamp":"0x1978e159ab0"}"#,
        );
        let attestation = deser_header(
            r#"{"hash":"0x5375cc683a0022546506c05e6ffdd795b25ee2bed1867688474ec86ec58dcc01","parentHash":"0xe01078f0512466d0eb277e99de0c78756546768e2ac904ba3f218c726e013c9c","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x76d76ee8823de52a1a431884c2ca930c5e72bff3","stateRoot":"0x09d6c0147c074f080ed141cbf2caf5fdba7310fbd53fe3e4895b3d6f49b54e33","transactionsRoot":"0xf458e35c177806418bcc6f8ee67eeac09472e82c60e57b9e5ce673cd8d66d59f","receiptsRoot":"0x8eb671ae0f8cf30892abf6d2c2c30766290a017d0cb4db4dbab534d0c779ec43","logsBloom":"0x00000000000004000000004000002000080000000000000000000000000000000000108000000000000000000000000000000000000000000000000000000000000000000000000000000008000000082010000000000000000000000000000200080020020200010000000001000800080004000000000002000010000000000000002000000000000002000002000000000400000000028000000000000020000000000000002008000000020000000000000000000000000000000000000000000802000000000080100000000100000000000000001000104002000020000004000000000000010000040000010000088000000000004000000000001000","difficulty":"0x2","number":"0x34e7b3a","gasLimit":"0x5f5e100","gasUsed":"0x571ae","timestamp":"0x6855868e","extraData":"0xd883010510846765746888676f312e32332e39856c696e757800000038469028f8b27fb860a994dcfd43a2e7793332ad42a17d1f1caa916cf900b3ed13f0e757c908e389f3fae08b7e5f0c95f919247491ad7f86050a2d1b5dc199092ffc7aa49227223eb346d1c1e3d7f6a471d03aebb1d678ebd9f9b3bc21aff102e0beadcdeeb1fd0dfff84c84034e7b38a0aeab92600438b508c9ec3528b0952ceffe95a68e7a13dc8b20549c4044f1922584034e7b39a0e01078f0512466d0eb277e99de0c78756546768e2ac904ba3f218c726e013c9c807603ff1db913fab4efdad0969896903683bd92483ae75ecc204a2effce6d4a011c19783f7dbddc5b50abfac6bd1b25bd6c25af6f8853bf834f45abc5fc5d8e8201","mixHash":"0x00000000000000000000000000000000000000000000000000000000000002ee","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x40000","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x699326f","size":"0x613","uncles":[],"transactions":[{"type":"0x0","chainId":"0x61","nonce":"0x2c7","gasPrice":"0x6687cf1","gas":"0x6db0d","to":"0x64e5b483c669e5ed772928b66ded4b4a27c6e563","value":"0x0","input":"0x46322c370000000000000000000000000000000000000000000000000000000000005ffb0000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000089506bf78819680","r":"0xa776179c2f22ad71e59ed20337eeeced2423e31276ec3a0a028ca54dca22c2b2","s":"0x37811d002b1bfb83b38370c5b0d226e3c0406482d2d19cffedeab0165b1f3a5e","v":"0xe6","hash":"0x2e12490d48e9c15eba3f6dc7a28621a1a7fb7be2e7e81ee650f932a9045cf857","blockHash":"0x5375cc683a0022546506c05e6ffdd795b25ee2bed1867688474ec86ec58dcc01","blockNumber":"0x34e7b3a","transactionIndex":"0x0","from":"0x1668d9eb11fd860690bf6f93bf1c34975cde7a7e"},{"type":"0x3","chainId":"0x61","nonce":"0x26e34b","gas":"0xcf08","maxFeePerGas":"0x65d1252","maxPriorityFeePerGas":"0x65d1252","to":"0xff00000000000000000000000000000000005611","value":"0x0","accessList":[],"blobVersionedHashes":["0x01e63988e6561e395ad3f6ba70c607f244cfc3f8883f3b9c4c2d915f21ba3787"],"maxFeePerBlobGas":"0x1","input":"0x","r":"0xc5fde710aa80540c2823ede8663a4c8e2a445f40bf6e32f1ba6b5386aaa2682f","s":"0x3d41032f6da0d5ada868fd09fabbc1377f5e80921746849bb668ee7f7232f0e6","yParity":"0x0","v":"0x0","hash":"0xc1247e8b696b7b3d4424bbb7374465264d5b0f2821591bbfa7bc13b8d36021b5","blockHash":"0x5375cc683a0022546506c05e6ffdd795b25ee2bed1867688474ec86ec58dcc01","blockNumber":"0x34e7b3a","transactionIndex":"0x1","from":"0x1fd6a75cc72f39147756a663f3ef1fc95ef89495","gasPrice":"0x65d1252"},{"type":"0x3","chainId":"0x61","nonce":"0x8b205","gas":"0xcf08","maxFeePerGas":"0x5f5e100","maxPriorityFeePerGas":"0x5f5e100","to":"0xff00000000000000000000000000000004702b62","value":"0x0","accessList":[],"blobVersionedHashes":["0x0132efa9e7060dd1497ae40d513e9f004c92b4a25c8ec22a99018be3e96df314"],"maxFeePerBlobGas":"0x1","input":"0x","r":"0xc51784ef5f371f687827f8e489f773a559f9afacd8774f130c881dc552257696","s":"0xd2e511e5aef8901eb64f8e5e662d23e3f48da2f5b8dc12dbdd457386b648d10","yParity":"0x0","v":"0x0","hash":"0xd08d4ccb7c110e06929d4d2b70cb9a05f9e95764368e014b9ea354ba8ae3e681","blockHash":"0x5375cc683a0022546506c05e6ffdd795b25ee2bed1867688474ec86ec58dcc01","blockNumber":"0x34e7b3a","transactionIndex":"0x2","from":"0x4018433590eaeb23925dd6322dbe5df556050025","gasPrice":"0x5f5e100"},{"type":"0x0","chainId":"0x61","nonce":"0x1f8d84","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x1d47e8d29ecc","input":"0xf340fa0100000000000000000000000076d76ee8823de52a1a431884c2ca930c5e72bff3","r":"0xec46e3567e8476ddb1466f33c556e669ef00f813d3866c2dce4f3ee730a22ef6","s":"0x4bf9227465fc67a4c003c7099624ddeb2c34e9170988483b6cd65b2fbb44ecce","v":"0xe6","hash":"0x777af38983ed905d0be218d2dac5eab45fa7d34e1da531edd6a4e03caabd26b6","blockHash":"0x5375cc683a0022546506c05e6ffdd795b25ee2bed1867688474ec86ec58dcc01","blockNumber":"0x34e7b3a","transactionIndex":"0x3","from":"0x76d76ee8823de52a1a431884c2ca930c5e72bff3"}],"withdrawals":[],"milliTimestamp":"0x1978e159d9e"}"#,
        );

        let (_, valset) = parse_epoch_rotation_header_extra_data(&hex!("d883010510846765746888676f312e32332e39856c696e7578000000384690280708265da01e1a65d62b903c7b34c08cb389bf3d9996f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c21a3d9d7a717d64e6088ac937d5aacdd3e20ca963979974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd99850023053387f3321fd69d1e030bb921230dfb188826affaa39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b76d76ee8823de52a1a431884c2ca930c5e72bff3803af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea99e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf90409f56966b5da954166eb005cb1a8790430ba1962a2342bac4831c6de73fcb77ad08669aaaa0a2ba6c6973a02b8928dbe573d17864e48c3521f238ace1c16e160bb7f5d447b49cd040d20bc21e49ffea6487f5638e4346ad9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb10f8b27fb860939558d8be3f3db29729a2ec0ba2d48ef9be23254980cb49c561d42ba58b4d727a3b4dd69a8df174a876f3ca799f61b60cd66d5dbec0b0c0be4ca3ff0d6e260234fec1b9f6c81f60a2da93831ef39f17230c017daa2cb815bbc2ba5fa84efe2cf84c84034e774ea064fb026e8f5c04e2a642ea5af194bc76961f0c26100fc04e6fe4216e02c867df84034e774fa07ed29e8be587d0d8de91070d0d9d22be22d6b385aa8e5c723e1510526441a5c880dffc3da994d38fe40d51cc4322cd0afe05ae676299b44af2fb2836c41e983baf3cda8ccd4e49392772dfe698f3aaf9a84e2ddd62a81d251784384770feaaa6b501")).unwrap();

        let res = verify_header(
            &source,
            &target,
            &attestation,
            Duration::from_secs(604800),
            55474000,
            BlstContext {
                current_timestamp: Timestamp::from_nanos(1750435993000000000),
                epoch_valsets: [(55474000, valset)].into_iter().collect(),
            },
        )
        .unwrap();

        assert_eq!(source.number, U256::from(55475000_u64));

        assert_eq!(
            res.unwrap(),
            (
                source.number.try_into().unwrap(),
                parse_epoch_rotation_header_extra_data(&source.extra_data)
                    .unwrap()
                    .1
            )
        );
    }

    #[test]
    fn verify_header_works_valset_rotation_before_delay_period_testnet() {
        // 55474112
        let source = deser_header(
            r#"{"hash":"0xb6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e8","parentHash":"0x4ac159608683e1d62cd3c46e54dfcb058e6caf1a3eb1ca33d6b5737da765198e","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","stateRoot":"0x8b1b1774811feccce4e55db210aac5bdcc1c0afc565db95a5bba7da1c5aa62c2","transactionsRoot":"0xffac28f838083beaf336f894fdb4292601eba3544c5d21cc5a5eb59bb2692628","receiptsRoot":"0x9ff97855e051eeee9ee1eb49efd5ab2ba52bf46d48500a57fff9fc6224bc6f75","logsBloom":"0x00000000000000000000104000002000000000000000000000000200000000000000108000000000000000000000000000000000000000000000000000200000000000000080000500000008100000002010000000000000000000000000000200080030000200000000000001000400080000008000000000000010000000000000080000000000000000000002000000000400000000008000001000000020020000000000002008000000020000000000000000000000000000000000000000000002000000000000100000000000000000000100001000104002000000008010000000000000010000040000010000008000008200004000000000001000","difficulty":"0x2","number":"0x34e77c0","gasLimit":"0x5f5e100","gasUsed":"0x51b50","timestamp":"0x685583f3","extraData":"0xd883010510846765746888676f312e32332e39856c696e757800000038469028f8b277b860a965d83592e741017209605a62eeb83f6ddec5ab918145d730023d7bd94ff5569c13965f9295b3a88d1be544e627c4030f67d6d3b6ca5b44c933d5fe67ef814f94d7f1671f30e64b3362e70d7014a6a2af6bddc233770ed91870fbe8091c7eeef84c84034e77bea0cd0ba3b6896eac74af8b382680a3f8513d8e0395f3cf5c8d5cf172c91a367eaa84034e77bfa04ac159608683e1d62cd3c46e54dfcb058e6caf1a3eb1ca33d6b5737da765198e8005aadb5aa8e1d22c5e2279f90347b7eb57e993ffeca1213edfa29bea4626586b041b35319b05e1e454beff424afe0c774f0633a6d11d879dc2494b0c7224162500","mixHash":"0x00000000000000000000000000000000000000000000000000000000000000fa","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x0","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6992b7b","size":"0x506","uncles":[],"transactions":[{"type":"0x0","chainId":"0x61","nonce":"0x75e","gasPrice":"0x64645cf","gas":"0x6c1a0","to":"0x5b16e45ab066bfc8882409b28fe768a2abde0d56","value":"0x0","input":"0x599682880000000000000000000000000000000000000000000000001306c53c730f80000000000000000000000000000000000000000000000000001e87f85809dc010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","r":"0xd0e5053300e407f28261932a62f4dcca6262fea6962e5cf6a248758ca2d21c65","s":"0x103a0170f58b3b6ffd500bc4b07d12fe63baa751048621743fdc72b3a6060446","v":"0xe5","hash":"0x83aec9f601af93de0fe4855a583d704973e1b9ea87de8f0bd2d5941879027a91","blockHash":"0xb6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e8","blockNumber":"0x34e77c0","transactionIndex":"0x0","from":"0x4e65e7b0b09e8f8b73af48ab4023024449c76a2b"},{"type":"0x0","chainId":"0x61","nonce":"0x1f9d76","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x1ab532b901b2","input":"0xf340fa010000000000000000000000007f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","r":"0xffa5acb9d5f44ca017ca17bf1867054f0ec9e47434bbb432659070bc5a31c579","s":"0xba421766635927b88779dc82805b20e9a35c64e4cfbef57a8fafe0bd851861c","v":"0xe6","hash":"0x9f6fbbaadccb38a5ffd30e3aa1c6b4887d4d4a1b04dd9bab3385adf426bdf2cd","blockHash":"0xb6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e8","blockNumber":"0x34e77c0","transactionIndex":"0x1","from":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea"}],"withdrawals":[],"milliTimestamp":"0x1978e0b6e32"}"#,
        );
        let target = deser_header(
            r#"{"hash":"0x6ec704e1a87d58d218103f31256e13016a513fc5ff844163d9477ed5829958b6","parentHash":"0xb6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e8","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","stateRoot":"0x26c683b0bfc542c844bd7646f4f30495fd1be51eaf5ca9f94f4f58c63401de23","transactionsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","receiptsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","difficulty":"0x2","number":"0x34e77c1","gasLimit":"0x5f5e100","gasUsed":"0x0","timestamp":"0x685583f4","extraData":"0xd883010510846765746888676f312e32332e39856c696e757800000038469028f8b27fb8608c2bd393036be207a5c88df82bcbd5364a5cd024ca42a42e0b16a59c43b3c4834eef510e9fc2f2c84eab6ef0f0cfabf018d26e9d07692c6a63388a3e9d60d80207d79283583d75b718085a8ed3b726165eaef42fe6d324c75cd4ffd82914479af84c84034e77bfa04ac159608683e1d62cd3c46e54dfcb058e6caf1a3eb1ca33d6b5737da765198e84034e77c0a0b6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e880f6d9135f0e16435ab7abad6e3cd91c1e24faa480ba3f865b2430eb9134f9b911735880a79968d5d767d461220f36353945056384cdd430ff75bf3620467ae35000","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x0","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6992b7d","size":"0x37e","uncles":[],"transactions":[],"withdrawals":[],"milliTimestamp":"0x1978e0b7120"}"#,
        );
        let attestation = deser_header(
            r#"{"hash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","parentHash":"0x6ec704e1a87d58d218103f31256e13016a513fc5ff844163d9477ed5829958b6","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","stateRoot":"0xdde121b931a529aa48595ca75ed3ca17f34a2eb7a18596ca2fd61980277b6da5","transactionsRoot":"0xaac61e88bd2f9108b22265ff0d314568a50b7cca9ff9eaed252a61a92414cad3","receiptsRoot":"0x700cc891041dc16782667ad7efae9adbc244f59a4dc21141fefc05d8a8b94eca","logsBloom":"0x0004000000000400000000400000320008000000000000000000020000000000000010800000000000000000000000000000004000000000000000000020000000000000018000010000000810000000201000000000000000000000000000020008002002020000000000000100080008000000000000000000001000000000000008200000000004000000200200000000040000000008800000002000002002000000000000200800000002000000000004000200000000000000000000000000000200000000000010000000000000000000000000100010400200002000001000000000000001004004000001000000800000800000c000000800001000","difficulty":"0x2","number":"0x34e77c2","gasLimit":"0x5f5e100","gasUsed":"0xa0520","timestamp":"0x685583f4","extraData":"0xd883010510846765746888676f312e32332e39856c696e757800000038469028f8b27fb860ac267d3f027905b02d1c0a6b28861c1f3ebce3229210e85dc7589f52891d44130f6530271d38aebe921ccc8daf46da400f0174c20ac07c6b00f734708a4ba1176754104a96d83228c766175e672717dca352aa4e6e79bed137389fd150f88daaf84c84034e77c0a0b6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e884034e77c1a06ec704e1a87d58d218103f31256e13016a513fc5ff844163d9477ed5829958b6806bf889d7fd07d18228692ba65cab19ce7fc112aa8d8d601768aca194b812f3c509f455e7817fa5cb4debc56deb1c06ccc59b644feeb3e69fa4813235f7fe420c01","mixHash":"0x00000000000000000000000000000000000000000000000000000000000002ee","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x40000","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6992b7f","size":"0x701","uncles":[],"transactions":[{"type":"0x3","chainId":"0x61","nonce":"0x26e26d","gas":"0xcf08","maxFeePerGas":"0x664ce58","maxPriorityFeePerGas":"0x664ce58","to":"0xff00000000000000000000000000000000005611","value":"0x0","accessList":[],"blobVersionedHashes":["0x01cd7201be3475313c5a8a33446a088dd3fe16c404f7b6235a308cc9280d5c58"],"maxFeePerBlobGas":"0x1","input":"0x","r":"0xfb09c05f7057c9869a213aa892a568becc35ecacb2c8415a477f7bc3c0dedc7f","s":"0x35a9db427770408e385198f85ef48a851ebba0430655a267d0d2c97d81f7c75c","yParity":"0x1","v":"0x1","hash":"0x179f008a08428fae128f7612510fd6441f9d5443fb5fac4bc1a636b7e0512933","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x0","from":"0x1fd6a75cc72f39147756a663f3ef1fc95ef89495","gasPrice":"0x664ce58"},{"type":"0x0","chainId":"0x61","nonce":"0x3d1","gasPrice":"0x64bfb21","gas":"0x6f4c1","to":"0x461e07c991aaef9007577ce280cd466a385b16b2","value":"0x0","input":"0x46322c370000000000000000000000000000000000000000000000000000000000005e3c0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000","r":"0x44fb9b28fb8e32a12546c30709bb088d13f8773a269c4dfe83914326dbbc0ce2","s":"0x567a0e6d24d3e6039cd53a7f7bc2be064c439b9641214606ead2147cf92b660c","v":"0xe6","hash":"0x0face212ceec5646833dd650654de4ae7e693f80cd720dc5092a3237c5de6767","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x1","from":"0x48243068aec52abc53398692272e45e558a2cf29"},{"type":"0x0","chainId":"0x61","nonce":"0x350","gasPrice":"0x646a6ea","gas":"0x78595","to":"0xc89c7066b6f1ff3a261738b95b1e419fe04c10a9","value":"0x0","input":"0x599682880000000000000000000000000000000000000000000000000f4ea4a7f341800000000000000000000000000000000000000000000000000017979cfe362a00000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000439fba6910edf800","r":"0x17b3ad7fc7df657a27820c82e4f643a768e86c4781a3536c29725c0f87a5a7ff","s":"0x7413e17dbc97cadca0f2ff73d09a941009ad9891acad91e96a6cfead62a40d43","v":"0xe6","hash":"0x3afd908ac648076157a964327903ca75650086366885d5ec0ee3b4fa58749724","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x2","from":"0xc52f6c876a926ec0d866e18a14df74873e18d706"},{"type":"0x3","chainId":"0x61","nonce":"0x8b128","gas":"0xcf08","maxFeePerGas":"0x5f5e100","maxPriorityFeePerGas":"0x5f5e100","to":"0xff00000000000000000000000000000004702b62","value":"0x0","accessList":[],"blobVersionedHashes":["0x01f40b458f9f00647055ca0d6822a4c399f8ef2fb5cc73bd1ea8f1a9fee6c8e1"],"maxFeePerBlobGas":"0x1","input":"0x","r":"0x86208af88d8caeec37da8d56c1b01abab40bacf20fba599744c5d5b844e7f731","s":"0x6cb2cfd6862446a5bff913df5252a42e4621578377012bd01e844d45dcf7a187","yParity":"0x1","v":"0x1","hash":"0x0c6487cfe64b731f929e5a3f639bf1c7ac49f2c34a71e0bb0d0bf403f5ea9eb3","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x3","from":"0x4018433590eaeb23925dd6322dbe5df556050025","gasPrice":"0x5f5e100"},{"type":"0x0","chainId":"0x61","nonce":"0x1f9d77","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x39924fcbbfc5","input":"0xf340fa010000000000000000000000007f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","r":"0x5fc6a6c5d98b2271488d383d7d9a42d6330398bd4da8e854031e7dd9df5dc1f3","s":"0x75923f47c94e8ff2f337eae0411da3bb5f934b946df1ac8384b75484aa44cd3","v":"0xe5","hash":"0x9f93db3a6afd0d1a41fdf85d77d618a8aee001accfc06a52f2606588a281eb8a","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x4","from":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea"}],"withdrawals":[],"milliTimestamp":"0x1978e0b740e"}"#,
        );

        let (_, valset) = parse_epoch_rotation_header_extra_data(&hex!("d883010510846765746888676f312e32332e39856c696e7578000000384690280708265da01e1a65d62b903c7b34c08cb389bf3d9996f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c21a3d9d7a717d64e6088ac937d5aacdd3e20ca963979974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd99850023053387f3321fd69d1e030bb921230dfb188826affaa39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b76d76ee8823de52a1a431884c2ca930c5e72bff3803af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea99e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf90409f56966b5da954166eb005cb1a8790430ba1962a2342bac4831c6de73fcb77ad08669aaaa0a2ba6c6973a02b8928dbe573d17864e48c3521f238ace1c16e160bb7f5d447b49cd040d20bc21e49ffea6487f5638e4346ad9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb10f8b27fb860939558d8be3f3db29729a2ec0ba2d48ef9be23254980cb49c561d42ba58b4d727a3b4dd69a8df174a876f3ca799f61b60cd66d5dbec0b0c0be4ca3ff0d6e260234fec1b9f6c81f60a2da93831ef39f17230c017daa2cb815bbc2ba5fa84efe2cf84c84034e774ea064fb026e8f5c04e2a642ea5af194bc76961f0c26100fc04e6fe4216e02c867df84034e774fa07ed29e8be587d0d8de91070d0d9d22be22d6b385aa8e5c723e1510526441a5c880dffc3da994d38fe40d51cc4322cd0afe05ae676299b44af2fb2836c41e983baf3cda8ccd4e49392772dfe698f3aaf9a84e2ddd62a81d251784384770feaaa6b501")).unwrap();

        let res = verify_header(
            &source,
            &target,
            &attestation,
            Duration::from_secs(604800),
            55474000,
            BlstContext {
                current_timestamp: Timestamp::from_nanos(1750435993000000000),
                epoch_valsets: [(55474000, valset)].into_iter().collect(),
            },
        )
        .unwrap();

        assert_eq!(res, None);
    }

    #[test]
    fn verify_header_works_valset_rotation_after_delay_period_testnet() {
        // 55474112
        let source = deser_header(
            r#"{"hash":"0x6ec704e1a87d58d218103f31256e13016a513fc5ff844163d9477ed5829958b6","parentHash":"0xb6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e8","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","stateRoot":"0x26c683b0bfc542c844bd7646f4f30495fd1be51eaf5ca9f94f4f58c63401de23","transactionsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","receiptsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","difficulty":"0x2","number":"0x34e77c1","gasLimit":"0x5f5e100","gasUsed":"0x0","timestamp":"0x685583f4","extraData":"0xd883010510846765746888676f312e32332e39856c696e757800000038469028f8b27fb8608c2bd393036be207a5c88df82bcbd5364a5cd024ca42a42e0b16a59c43b3c4834eef510e9fc2f2c84eab6ef0f0cfabf018d26e9d07692c6a63388a3e9d60d80207d79283583d75b718085a8ed3b726165eaef42fe6d324c75cd4ffd82914479af84c84034e77bfa04ac159608683e1d62cd3c46e54dfcb058e6caf1a3eb1ca33d6b5737da765198e84034e77c0a0b6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e880f6d9135f0e16435ab7abad6e3cd91c1e24faa480ba3f865b2430eb9134f9b911735880a79968d5d767d461220f36353945056384cdd430ff75bf3620467ae35000","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x0","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6992b7d","size":"0x37e","uncles":[],"transactions":[],"withdrawals":[],"milliTimestamp":"0x1978e0b7120"}"#,
        );
        let target = deser_header(
            r#"{"hash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","parentHash":"0x6ec704e1a87d58d218103f31256e13016a513fc5ff844163d9477ed5829958b6","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","stateRoot":"0xdde121b931a529aa48595ca75ed3ca17f34a2eb7a18596ca2fd61980277b6da5","transactionsRoot":"0xaac61e88bd2f9108b22265ff0d314568a50b7cca9ff9eaed252a61a92414cad3","receiptsRoot":"0x700cc891041dc16782667ad7efae9adbc244f59a4dc21141fefc05d8a8b94eca","logsBloom":"0x0004000000000400000000400000320008000000000000000000020000000000000010800000000000000000000000000000004000000000000000000020000000000000018000010000000810000000201000000000000000000000000000020008002002020000000000000100080008000000000000000000001000000000000008200000000004000000200200000000040000000008800000002000002002000000000000200800000002000000000004000200000000000000000000000000000200000000000010000000000000000000000000100010400200002000001000000000000001004004000001000000800000800000c000000800001000","difficulty":"0x2","number":"0x34e77c2","gasLimit":"0x5f5e100","gasUsed":"0xa0520","timestamp":"0x685583f4","extraData":"0xd883010510846765746888676f312e32332e39856c696e757800000038469028f8b27fb860ac267d3f027905b02d1c0a6b28861c1f3ebce3229210e85dc7589f52891d44130f6530271d38aebe921ccc8daf46da400f0174c20ac07c6b00f734708a4ba1176754104a96d83228c766175e672717dca352aa4e6e79bed137389fd150f88daaf84c84034e77c0a0b6c3fb5db2749184b29c250dcde3604422b584603a9d8462d051c2e2fc2698e884034e77c1a06ec704e1a87d58d218103f31256e13016a513fc5ff844163d9477ed5829958b6806bf889d7fd07d18228692ba65cab19ce7fc112aa8d8d601768aca194b812f3c509f455e7817fa5cb4debc56deb1c06ccc59b644feeb3e69fa4813235f7fe420c01","mixHash":"0x00000000000000000000000000000000000000000000000000000000000002ee","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x40000","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6992b7f","size":"0x701","uncles":[],"transactions":[{"type":"0x3","chainId":"0x61","nonce":"0x26e26d","gas":"0xcf08","maxFeePerGas":"0x664ce58","maxPriorityFeePerGas":"0x664ce58","to":"0xff00000000000000000000000000000000005611","value":"0x0","accessList":[],"blobVersionedHashes":["0x01cd7201be3475313c5a8a33446a088dd3fe16c404f7b6235a308cc9280d5c58"],"maxFeePerBlobGas":"0x1","input":"0x","r":"0xfb09c05f7057c9869a213aa892a568becc35ecacb2c8415a477f7bc3c0dedc7f","s":"0x35a9db427770408e385198f85ef48a851ebba0430655a267d0d2c97d81f7c75c","yParity":"0x1","v":"0x1","hash":"0x179f008a08428fae128f7612510fd6441f9d5443fb5fac4bc1a636b7e0512933","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x0","from":"0x1fd6a75cc72f39147756a663f3ef1fc95ef89495","gasPrice":"0x664ce58"},{"type":"0x0","chainId":"0x61","nonce":"0x3d1","gasPrice":"0x64bfb21","gas":"0x6f4c1","to":"0x461e07c991aaef9007577ce280cd466a385b16b2","value":"0x0","input":"0x46322c370000000000000000000000000000000000000000000000000000000000005e3c0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000","r":"0x44fb9b28fb8e32a12546c30709bb088d13f8773a269c4dfe83914326dbbc0ce2","s":"0x567a0e6d24d3e6039cd53a7f7bc2be064c439b9641214606ead2147cf92b660c","v":"0xe6","hash":"0x0face212ceec5646833dd650654de4ae7e693f80cd720dc5092a3237c5de6767","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x1","from":"0x48243068aec52abc53398692272e45e558a2cf29"},{"type":"0x0","chainId":"0x61","nonce":"0x350","gasPrice":"0x646a6ea","gas":"0x78595","to":"0xc89c7066b6f1ff3a261738b95b1e419fe04c10a9","value":"0x0","input":"0x599682880000000000000000000000000000000000000000000000000f4ea4a7f341800000000000000000000000000000000000000000000000000017979cfe362a00000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000439fba6910edf800","r":"0x17b3ad7fc7df657a27820c82e4f643a768e86c4781a3536c29725c0f87a5a7ff","s":"0x7413e17dbc97cadca0f2ff73d09a941009ad9891acad91e96a6cfead62a40d43","v":"0xe6","hash":"0x3afd908ac648076157a964327903ca75650086366885d5ec0ee3b4fa58749724","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x2","from":"0xc52f6c876a926ec0d866e18a14df74873e18d706"},{"type":"0x3","chainId":"0x61","nonce":"0x8b128","gas":"0xcf08","maxFeePerGas":"0x5f5e100","maxPriorityFeePerGas":"0x5f5e100","to":"0xff00000000000000000000000000000004702b62","value":"0x0","accessList":[],"blobVersionedHashes":["0x01f40b458f9f00647055ca0d6822a4c399f8ef2fb5cc73bd1ea8f1a9fee6c8e1"],"maxFeePerBlobGas":"0x1","input":"0x","r":"0x86208af88d8caeec37da8d56c1b01abab40bacf20fba599744c5d5b844e7f731","s":"0x6cb2cfd6862446a5bff913df5252a42e4621578377012bd01e844d45dcf7a187","yParity":"0x1","v":"0x1","hash":"0x0c6487cfe64b731f929e5a3f639bf1c7ac49f2c34a71e0bb0d0bf403f5ea9eb3","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x3","from":"0x4018433590eaeb23925dd6322dbe5df556050025","gasPrice":"0x5f5e100"},{"type":"0x0","chainId":"0x61","nonce":"0x1f9d77","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x39924fcbbfc5","input":"0xf340fa010000000000000000000000007f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","r":"0x5fc6a6c5d98b2271488d383d7d9a42d6330398bd4da8e854031e7dd9df5dc1f3","s":"0x75923f47c94e8ff2f337eae0411da3bb5f934b946df1ac8384b75484aa44cd3","v":"0xe5","hash":"0x9f93db3a6afd0d1a41fdf85d77d618a8aee001accfc06a52f2606588a281eb8a","blockHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","blockNumber":"0x34e77c2","transactionIndex":"0x4","from":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea"}],"withdrawals":[],"milliTimestamp":"0x1978e0b740e"}"#,
        );
        let attestation = deser_header(
            r#"{"hash":"0x4339b6374c679fa16ca476b318030bc0119f5853274f706d1204e31f676745f5","parentHash":"0x3fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","stateRoot":"0x166fbf66c503377536a8d56826bee5f27c1a7f12901518c614807155fdb72933","transactionsRoot":"0xf32c0923c016612b60094495958bc70d4800b32c9c88c43d85b62c1ac3548e90","receiptsRoot":"0x895100f4586b6e3eb4776d3ea3cd6dd2ae594ecd6eee63b29e652d031ca0ea37","logsBloom":"0x0000000000008400000004400000200008000000020000000000000000000000040014800000000200000000000200000000000000000010221000000000000004000000004000010000000c000000002010000000000000000000800000600200080020020200000000000101000800080000000000000000200010000000000004482020000000400000000002000000000400080001008000004000000020080000000000012008000000020000000000100000000000000000002000000000000002001000000000100000000000000000000000041000104003000030000000000000001000010000040000010000008000008080004000000004009000","difficulty":"0x2","number":"0x34e77c3","gasLimit":"0x5f5e100","gasUsed":"0xfd28c","timestamp":"0x685583f5","extraData":"0xd883010510846765746888676f312e32332e39856c696e757800000038469028f8b27fb860a8f61a3fccdd005d088675f23ce095e7491244f0cbd53b28a60ea1533a94dc03c22aa36106bfc4f7555a392e634c6e9d0f0aa5e573e2c6c4ce7a9b66662fbc5b909666d745f554a906af696888f990d92fa6dbf56ba4f9f62b71eef5053660def84c84034e77c1a06ec704e1a87d58d218103f31256e13016a513fc5ff844163d9477ed5829958b684034e77c2a03fdd6eb1e12e042bb06250423c472e8edd996c830b8e91a5e53f7eda03aa9405804a3580d2f84d406cf66d597425e94e93f8cb331947cfacfe3a7d95aa60cc8e437f512a2e95f212e6b19f7b717f58c753984fc5c21facd98fc2f7730406d02f6201","mixHash":"0x00000000000000000000000000000000000000000000000000000000000001f4","nonce":"0x0000000000000000","baseFeePerGas":"0x0","withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","blobGasUsed":"0x0","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","totalDifficulty":"0x6992b81","size":"0x8eb","uncles":[],"transactions":[{"type":"0x0","chainId":"0x61","nonce":"0x3226","gasPrice":"0x77359400","gas":"0x7a1200","to":"0xff894ac1c2e23469613628e55d662dc541aa176a","value":"0x0","input":"0x62b980320000000000000000000000005437e6696001f582a34f6552ae61e31ef3180f1a","r":"0x78074443da16c7ef6d4e86105e9302450f9d33d06fcecbce65ce53b7fed46f11","s":"0x9d8ed27bfd0595700dfc908800805cde12a4cb9ef107810527cff2bfb0c8341","v":"0xe5","hash":"0x4bc91509fcc45037079c73ed646cd5ace60050fd8b08add6e344660e1064fdd1","blockHash":"0x4339b6374c679fa16ca476b318030bc0119f5853274f706d1204e31f676745f5","blockNumber":"0x34e77c3","transactionIndex":"0x0","from":"0x5437e6696001f582a34f6552ae61e31ef3180f1a"},{"type":"0x0","chainId":"0x61","nonce":"0x44b","gasPrice":"0x3b9aca00","gas":"0x989680","to":"0x1b81d678ffb9c0263b24a97847620c99d213eb14","value":"0x0","input":"0x414bf389000000000000000000000000e588807c0581d11ee2d7665eb67d8143ea5717bd000000000000000000000000f972f0477c4a5b054e00a19bbdf71f7941600bfc000000000000000000000000000000000000000000000000000000000000006400000000000000000000000039afffca3826728187926dd1f29562434971bdcf00000000000000000000000000000000000000000000000000000000685588a0000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","r":"0xcf36fe13f2479df2b9dbcf01aef3d549e303bf1f262712e7f45416e2bac63d26","s":"0x7efc79a3315752a5c35670a543675ac04b927d10e22a07660a33aba3f20f5baf","v":"0xe6","hash":"0xa75d0a8d37a718181487ed82dd76325ef0dfc1cb3b21cf3de0016360f8f04cf5","blockHash":"0x4339b6374c679fa16ca476b318030bc0119f5853274f706d1204e31f676745f5","blockNumber":"0x34e77c3","transactionIndex":"0x1","from":"0x39afffca3826728187926dd1f29562434971bdcf"},{"type":"0x2","chainId":"0x61","nonce":"0xe6","gas":"0x40714","maxFeePerGas":"0x3b9aca00","maxPriorityFeePerGas":"0x3b9aca00","to":"0xd9d2948f867cf04e06db278aad2dd0fe75fbe9f2","value":"0x0","accessList":[],"input":"0x46322c370000000000000000000000000000000000000000000000000000000000017c6a0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000235ccaf2d57c904ad0","r":"0x800dc940824bd516ccaccacdfbc0bd22be120bc9ea99886144b2e43885c698bf","s":"0x480c35bace84ef3fc9b715b1c52350a5c8873f7a8e64d55a1de809eab66a146f","yParity":"0x0","v":"0x0","hash":"0x0db554e91fbd1d1e0fe2c7c4ee87551284eebc72cb4d48fba3906fea21a6d082","blockHash":"0x4339b6374c679fa16ca476b318030bc0119f5853274f706d1204e31f676745f5","blockNumber":"0x34e77c3","transactionIndex":"0x2","from":"0x2487064ef99208791f0a409a4d5350c2310346b4","gasPrice":"0x3b9aca00"},{"type":"0x0","chainId":"0x61","nonce":"0x59","gasPrice":"0x7270e00","gas":"0x6b49c","to":"0x757c732844ef5953b163fa2fd4f7dc0093ed3f44","value":"0x0","input":"0x46322c3700000000000000000000000000000000000000000000000000000000000060410000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000001f1a23384c8335c5","r":"0x85b7bf8b08ee904c51ee61e315c3e0257fc6feb418549144e5cb281f785fdfe8","s":"0x5c675a28445f41395c4e21ddca01160056657cf7f8b838fd4f94ce8067fe9bb9","v":"0xe5","hash":"0xb96b2d7cf25dae4282445d1b74aeaa3ee91abc9b9a33627260820c771271ea45","blockHash":"0x4339b6374c679fa16ca476b318030bc0119f5853274f706d1204e31f676745f5","blockNumber":"0x34e77c3","transactionIndex":"0x3","from":"0xa86969b73bac8c3650462c8ed665aff143c626c7"},{"type":"0x0","chainId":"0x61","nonce":"0x3ed","gasPrice":"0x647bbed","gas":"0x73c04","to":"0x5a42905192d4a2d27f0664e912b7f3ffd8254591","value":"0x0","input":"0x46322c3700000000000000000000000000000000000000000000000000000000000017d00000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000000","r":"0x503ff7834a4df61446256650720af38e8e52d6f4248d86333e17ba0e7c5b30a7","s":"0x99b103c9883fe35eaef7c37901fa7dc7acf94985809d60e3e8ea56e334444d9","v":"0xe6","hash":"0x1bc774548feccddfe90c44dcb90ff49a6f9831b3aaf80ac784bfb2fd2aa2c507","blockHash":"0x4339b6374c679fa16ca476b318030bc0119f5853274f706d1204e31f676745f5","blockNumber":"0x34e77c3","transactionIndex":"0x4","from":"0x916796bf803205abd5e9202d7563d41901ec28dc"},{"type":"0x0","chainId":"0x61","nonce":"0x2138e","gasPrice":"0x0","gas":"0x61a8","to":"0x0eabbde133fbf3c5eb2bee6f7c8210deeaa0f7db","value":"0x0","input":"0x","r":"0x911041e4adbf1edf7488a51df24f24b9971d2152d9a7d93d17763ec15186db47","s":"0x637daddf39a913f08576d464cfc1325981645f83bd3f55747eeff59ed2b1fa9d","v":"0xe6","hash":"0x29e3d9c6d5055d0331f308fa67ac9e45df95f9e429488e2e21d4d2df82f8d4a1","blockHash":"0x4339b6374c679fa16ca476b318030bc0119f5853274f706d1204e31f676745f5","blockNumber":"0x34e77c3","transactionIndex":"0x5","from":"0xa2f5df6f9055d991033813408bfd9fb8f3cf0fe9"},{"type":"0x0","chainId":"0x61","nonce":"0x1f9d78","gasPrice":"0x0","gas":"0x7fffffffffffffff","to":"0x0000000000000000000000000000000000001000","value":"0x258b628443f32","input":"0xf340fa010000000000000000000000007f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea","r":"0x2f58b47114cf359cc3986c64b89bd73ef014abd89bb901509bcf1734c5b60285","s":"0x290b409eb99510d251be9988216cb6dcf72624ea608ff7d397bba538804da9c3","v":"0xe6","hash":"0x6e74231cfc482c398df081bca3f58b0aade00544b20741d9c639876a599775f7","blockHash":"0x4339b6374c679fa16ca476b318030bc0119f5853274f706d1204e31f676745f5","blockNumber":"0x34e77c3","transactionIndex":"0x6","from":"0x7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea"}],"withdrawals":[],"milliTimestamp":"0x1978e0b76fc"}"#,
        );

        let (_, valset) = parse_epoch_rotation_header_extra_data(&hex!("d883010510846765746888676f312e32332e39856c696e7578000000384690280708265da01e1a65d62b903c7b34c08cb389bf3d9996f763f030b1adcfb369c5a5df4a18e1529baffe7feaec66db3dbd1bc06810f7f6f88b7be6645418a7e2a2a3f40514c21a3d9d7a717d64e6088ac937d5aacdd3e20ca963979974cd8ff90cbf097023dc8c448245ceff671e965d57d82eaf9be91478cfa0f24d2993e0c5f43a6c5a4cd99850023053387f3321fd69d1e030bb921230dfb188826affaa39ebf1c38b190851e4db0588a3e90142c5299041fb8a0db3bb9a1fa4bdf0dae84ca37ee12a6b8c26caab775f0e007b76d76ee8823de52a1a431884c2ca930c5e72bff3803af79641cf964cc001671017f0b680f93b7dde085b24bbc67b2a562a216f903ac878c5477641328172a353f1e493cf7f5f2cf1aec83bf0c74df566a41aa7ed65ea84ea99e3849ef31887c0f880a0feb92f356f58fbd023a82f5311fc87a5883a662e9ebbbefc90bf13aa533c2438a4113804bf90409f56966b5da954166eb005cb1a8790430ba1962a2342bac4831c6de73fcb77ad08669aaaa0a2ba6c6973a02b8928dbe573d17864e48c3521f238ace1c16e160bb7f5d447b49cd040d20bc21e49ffea6487f5638e4346ad9fc6d1ec30e28016d3892b51a7898bd354cfe78643453fd3868410da412de7f2883180d0a2840111ad2e043fa403eb10f8b27fb860939558d8be3f3db29729a2ec0ba2d48ef9be23254980cb49c561d42ba58b4d727a3b4dd69a8df174a876f3ca799f61b60cd66d5dbec0b0c0be4ca3ff0d6e260234fec1b9f6c81f60a2da93831ef39f17230c017daa2cb815bbc2ba5fa84efe2cf84c84034e774ea064fb026e8f5c04e2a642ea5af194bc76961f0c26100fc04e6fe4216e02c867df84034e774fa07ed29e8be587d0d8de91070d0d9d22be22d6b385aa8e5c723e1510526441a5c880dffc3da994d38fe40d51cc4322cd0afe05ae676299b44af2fb2836c41e983baf3cda8ccd4e49392772dfe698f3aaf9a84e2ddd62a81d251784384770feaaa6b501")).unwrap();

        let res = verify_header(
            &source,
            &target,
            &attestation,
            Duration::from_secs(604800),
            55474000,
            BlstContext {
                current_timestamp: Timestamp::from_nanos(1750435993000000000),
                epoch_valsets: [(55474000, valset)].into_iter().collect(),
            },
        )
        .unwrap();

        assert_eq!(res, None);
    }

    #[test]
    fn calculate_signing_valset_epoch_block_number_works() {
        let cases: &[(u64, u64)] = &[(50932088, 50931500), (50932089, 50932000)];

        for &(h, expected) in cases {
            assert_eq!(calculate_signing_valset_epoch_block_number(h, 21), expected);
        }
    }

    #[test]
    fn header_hash() {
        let header = deser_header(
            r#"{"withdrawalsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","withdrawals":[],"hash":"0x20315dee443b8910db5041505a17758cc44ac2348af003a08760f9a14995f3ae","parentHash":"0x456e27b4d951b8503078f5fa44d20f8d2ce976f4424a9c45782b73658c856d50","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","miner":"0x8a239732871adc8829ea2f47e94087c5fbad47b6","stateRoot":"0x2f3a8a8a47bc02231865163b9cc539e67a10e97bf8acc30279f2649f7c21cc85","transactionsRoot":"0x99f7054aa082decd3ac39e0a8d9189ec4d1d91fe953b2ada2cce29ec3a5715e6","receiptsRoot":"0x903944bf3a03f3933949bf9355dc73be8be02dfbf86d4066a1b6e41f0251346f","logsBloom":"0x05b1561781c6205dcea13c5b96cb1a801fd0b40a1e1b6ba42f80e21636949a9db102798b165f02212689f5202c8a83818904b22014dc6085068aa319552805d1fc0adf0c13684c3c43946d4cb63b9e24b490cabb336ac12e201500d0da2b9e68c3db07217846b842edec4337d71679a70d82841746856c0047acc8dda2a9a3db9084cd223a5c7a4889883b96bd4087f3e87eec65b19be169028280e18306572062ec04c9032e97a1ab0e3371c6644771d3aa36c0408e090d09dfb1a9e1ab9899b869e1ae4dc080a1aab47198c11acb09f1889d333ba42e983072dc73b404c8010ad30b178da4481309f5a28d926781c11491f7a0684b357b71ee177fb2c16721","difficulty":"0x2","number":"0x30a4c62","gasLimit":"0x7735940","gasUsed":"0x18fe8b7","timestamp":"0x68437ede","extraData":"0xd88301050c846765746888676f312e32332e37856c696e75780000003bfc8c16f8b5831f7fffb860906c1bdcc84b3cfbbab0f3df2939413a36c373e351fe7ec6654cf983b8d6ae80f052d2a66bddcf7bf6f4c28754c4dc770e68597fba67ee38748639c31083ed517ccaae5836e421d4b0ba0234173b456646d8146f0063846eb660e1fdcf91f3baf84c84030a4c60a0db7cefa3c1fe375262865b93e4d4223b045da94df8fd769875fb5fc7f73056da84030a4c61a0456e27b4d951b8503078f5fa44d20f8d2ce976f4424a9c45782b73658c856d508003ce6d018ef27172e9d39c346aa0dc3a939a4cc3766078b54311526b987d418332d87369564cf5421fd122417e61369bff2c85e967e2ce875936ef1cd57b321d01","mixHash":"0x00000000000000000000000000000000000000000000000000000000000001f4","nonce":"0x0000000000000000","size":"0x19fed","totalDifficulty":"0x60bbae3","baseFeePerGas":"0x0","transactions":["0xd70bcfb684574678fcf1be9241721a3115ac9bc164fdfafb8c0076a8092852f3","0xe7ed8f53738977e3417aa4330af9085b7b89107949bfa87937ad9ad3cd94c4ba","0xf4dd39006116a9dd2a9300916d91369e5371d8adbb1a95438461066ab31fb6e7","0xbbd80c91921bedd0f7bbda6ed21f31f3419685437f7f4ad21f8b875647e5750c","0x475408e05fa89cda24ecfde9fb13a174cbb0c6f0f5a69a3536e18ff74e978144","0x559da3c6e132eeb8b24fa900cb833311951290b3e9eabb114ffae6045e7e2062","0x52175770a4868c8059cda61afefa4cfdfeacbd99b2b59f61f03de7bf9f44af2c","0x17fd8df76b1b441fcfb466c9190890fe9a205c3f632f1ddd1147c98568a00bf7","0x8e2862d1e49585e7fd8b0fda46e87a7a2265f69c968dc5d8d6750c969af84310","0xe711e53d328ddc2b9a675774e946eec76154cb0645bf9f414b0b513f67f91486","0x8d5ff5d4e92ad8aeecb439116f35e3c10186adfefd1a9679d12fc904884995c0","0xa2349c374babfd104b98273e0c38878a25fa1849584550072c48965bcd8d69a0","0x7192e767bb8650d6c4fc87186d2a017d8362994dac9e335069b1c80d43bf81d2","0x474d51fb474a845144496d12b9eb283e4a5212e4cbb462c8f2c15ea58454a3e9","0x7e3a43c86631aee8980abbb97553c95ebcdf975eb9fad283616530448e230b40","0xc57c21f95b8c850be12492e2c0c9061c011e53c90b0ef40731080f50c270cea8","0xd09fab2203c10ad75e05c6addefba1661c10fb03dcf208400bc1f4947b333cef","0xd255a1214cfb4af0fde445b2e7fc23df6515d886a7d01e54539afba000e6f4fe","0xcde261170a5781e6a62345aa8fc8199e66c598bb9161c0027e47befdb6d8e4bd","0xbd5a6d62fa05ba3489fb0aaa67c975189c54e90c2d709193e49ba32c18e8f269","0x6ba819c57ba21be8aafc4f82482dfaeaf3533f122ac1ec033a0c6666b40dadb5","0x94c8759f8fdea8e493b7cdd008be56f79d0eb9badb5ff0ca659180e10e66dcbb","0xf41af02ff13ba9a1c9bde9354e4cb7c1285509f6acb7460989e0f23e3504c057","0xb7a4f11426cb86c1c8ede91b38e862bb56e0692550e83ee2581fd2a78aede761","0x6e83b72f7bb6e29e567d6741888854265a38ab213853c4c3b883614b899fdc0f","0x64be93c53d71c93d81f7fc019651f3c6e50f3ad71e5527ff82f43ce2cc525c9e","0xf9f1e95918a8c49c212c5f59eebd3683ed85b401b07ba6cdc6d1c6bf68e94892","0x427a93a52031e5cfa4d195ae83eba7d0a4a1653558ea3acdba3d36c75a6a44b9","0x8fc87c00f10c4298747a617310383d23a8fac23b6dbdeadb4516d55f9d3fb254","0x134637a3371a0dad404eae48a5d2c4b432e990ac5558dfca19e0cb2817b71eed","0x50f5d0be08c64e5b961849630b6661bd782f31f7e3bc0c59bf01cab14e4f4162","0x532e7c11ca5add13d3eff66d11a045a74b117e4340135dc7acc591268f53e195","0x18b04f30cccb5d739a4d925e1c850b18d8af6745652d7856820b0b702c42ad18","0x3c9f6aff6e1e9d6ac20fda1d57dbd3ae94920606d26adf5ae1320ab97b72038d","0x1b81126aec50cfefeb3b4782047aba2329da16ac7d47c017fd78556d78184272","0x27d690a513924a1cc59c11c72612e62f8ab6d83444b85141c5e4e18f6ad09588","0xeab0235bf24dc2e2359df448781bc0a894540e657be4fa7e7633a10f51f20fc0","0x873b2c3b4a5afd4c05901e5f63d511d13446437f6083980b86b9c4965cc874c8","0xb0821de52aaa06df3d0041fd2b1de939b6b61143903d27e85d3ce1381d315f7b","0x334782083a380d61e3a9eaf9fa94961bbc8d9b2621f9187fed6afaab1b72e031","0x359e14b11afa7a529caa744015d70cc5394441d5e39179640bad96eca0870da6","0x553f40147893f1cbdd2ee1d0dc5752bbb7229038b0e6d6bd22373b819551f5a0","0xe1595718307d6ef4fe7be792ae44349e78c6c6c57f20315916736e9356d208c0","0x35539bea101352ba0eac67997821c142db055243ecc6cff54f82a2ac16503d27","0xa75f9dc20ef4d2a582a9a2cb81c635aecb28ea487625280d1cf49883eea15cf9","0x63d62fc909c202032f822ec249001fea897752a223b870cda3f6d79fda782182","0x5f815b65bc2cbf7462474a40a2070b81570a84496962029351f5436e8b81c738","0x2f272fcd59f8c96c8c4981ff18da9996423e026394bf929de286f8c51b0ee12c","0xee270fa07b37f0c69cb2f0781017b72e72bca3ef7fad4ceb14a6ddf68117d685","0xc323a02d3728af99e9b3bb8025d5d1b32b2c25c88e5890617bbb0b390cf221d0","0xc8a111cb49b8655afe8dc2ff88686f09cd7b814c6039e9e22bec05b5fa59f6f4","0x4e7f93d1d89a1991354a7a665019856a082598d1090583e91e886bf5bac40a55","0xf6dee35b63f72a5b430162e66ec60954f766f5056db41fb306cf1e543df8f08d","0x538ce201ee0a9923975333ee7fb992dee03fe65f19797cbf6c0ad8ce472b0864","0xa75116ddecedfaf49eebf4c8d26ee25ad7ea3bc14e9493919fefc68695dafd4c","0x8fb318f59097f0ee67c845c7ec1245c459e0e24728efca6b83f8e6783a178c84","0x60fd55f1493f32bba514e71bbb8530ddbd55de65c673402d38d7f2855a8a8ff4","0x7b6a18fb7cf38ab11c44afae1dd8b67c69cc22b6c79676af93852b5b2441ee18","0x5935be1b988d630e94b68b1fc212c38a9dcffef5022294256a7ac9983826f4d0","0x8e7e037f1ab8872092e7384dedeb9f5fe20c5b43748e22731ce857e9b286a68c","0xc2c54fca444f45aef695e3dd56c4a400346faf88246d9d259e8ff12fc05870a0","0x0cb4b6fbd4138d26a8ef00d0ed9b2097944b812879a5d3c415ff6138cd34b71f","0xbc7748c18d252b70ccf61b742078ee677e1aa61d0f2fac8fbcc4856aba9e142c","0x9a1061da83bb69267eb644654ad0ea8d74c326e28cac5da9801f7ece7c10ed88","0x4b3d4db599e833975e00dcc44f7060a5583ccdac0de4a9df3562a78d62007e93","0xad0c9d1b114d55232135cf0fd8b522b31510549f709252a5e4ad0f85523ac926","0xd4d84ad4d6aa8601ae4083f4f49b758286f025b9a394a70f6795b2b63877eb02","0x8c85524a9a214879300b2177c01295f6c57346b32b988afaea057fc27fa8bc57","0x1af2a8d6104153b39028c191176f5dd7916998e53d9396d9a6a83f5ef8bfd499","0xaadbf5f4a7117753d2daa4354405dc4e149ae78c6a885db7d4829d5d70b75477","0xb84b6737eb97b5f9edc9c0088eead1867363c257e3253c51955df24e4d3d4c43","0xac65405cdcfd0fce361e7f9659106b124d0b484c7fb605a81d72933ecfd1019e","0x0da296eadcd08bc4fc1f6bfd57b293ff3341d02245a94d392aee3a9da5bf8ae5","0xfbe72bc8afd51a387048e0b3c65d49f93e694e6fad563e362a6e31d6f688a45c","0x0dc94f04009b041b2e86063481bc1ac77b6ddbc890c044fff5d5fd25739c9a31","0x65fabb0f6458be0f0da983cb3ff3ebb18c63556c46ea27a4c7adbf1eee4e733e","0x3d904d13bb4d7dcd07e1f13b720d2a64abacd6c66af44784d7041193a25b845c","0xbf35a635913b4a1ea0a3e03a7677c381ad85f281e048a9aa48ef4fd84e0c8059","0x44d190f6af878ee59a7f6114e46bb7cc112a95622721c57736c267a09a95fa71","0x9b01c042bb3a530b63fdd50607f26228bb41ce3d870de0f8bd50767f10af36d2","0xef3f202daeb4f68085c046f26537c0a7e8959685791d66885b5dbd39042b8c42","0x64556be7866f722894db2d9b714ab275f7808b27079b90b0ab36a9c8ed354cfe","0xabae711066594294d42fd2dc398f6a24993d006e7905109da588a1237cf5d9ec","0xf7571d06bc13d3e5e3664342b91dc296be7b31b498c2694d9eb4642608c42ba1","0xec5c33d369f61e122efddae61d234b840de86e01f839ae1f6b6afe3cba02d5cc","0x527bdbec6852d2d874e851a540d19c1c24b7536be712ac209588a05a1f75a20c","0x091ac144134a4947e88748e7d464497f755a6bc1e3dae15c127da3720a3f0235","0xa7a12f10c6c6caec4a38c30331ee81d1648e85e9fcafc3ad4648133ae96ba5e3","0x6bdc754a1c45cb0f645986a1d9ac0c0b41064bb9fcd759d293cc7a6d0717d7d4","0xea0d9045dd635053b3ea1f9f1999228a7f055f30a84bf6e96bdd024c9786451e","0x60e982776861ba76843e54e6a5e7476099ac12e4c886b8770c1abda713fef1e8","0x956598051b6b228feb5bba3d42aabae700379523722950b4125e1edb50fb78b7","0xb20f48f824ffce17c6891fa8568f35b27065163bd6a5e18fa223dc0658da85a7","0x1ec1fc61bb4da619471ea1b375ef23cb1ee3772024ce918ca7df21e168a29143","0xe3ec52c46535b90770f8be22d657cd5b02ed9d431224dbc5ad5a719f44442f51","0xd39aa24bce88b1c269559c0eab306112d4d9e80fba59735102ec092d8f9c2a99","0xd7be45de98bcb7195da293f44612e2936c783af9f4621ad4f96418892b3bcda4","0x7491521271f96c882527dd8e1050ca4cf94610a7e8b17481be9db8bae60f7d4d","0x1106d1b8f345e613ace166f57f97143b5f82e31cfccfd2e2c496c3700ce9069f","0x4e34717247fecd356d46ba5f128fa745618d2c5c897cf3d1cae3824c8e9164a0","0x837817ce50ce0358a2484a1fc4a77c6f6b4d717dab4afed248f20d89c739b7db","0x49ae8e1c8f8ccea2ba33dbb7a8c754c4dc722b5fac18273583114f26bc752bd8","0xff0e7fb06a772549b088759bcd6db12d4616727f38b27c7e907a85b6655f1513","0x39781a4502307985b04c357eb240d50f5dd576cf7817fe6de46efe0e34b14c16","0x5632656c2f6cc39a7d36ba12938b8b25b0ef5a01cab607ab70c6cad2d813951c","0x0136426ee24f304d02e1ea522b71212051c6e96a613147bbc8979d452e5b28ac","0xafacaab83382c815ed33d327bcb5d9ceb19640c85fac3ff45b862e5fe486b0f2"],"uncles":[],"blobGasUsed":"0x40000","excessBlobGas":"0x0","parentBeaconBlockRoot":"0x0000000000000000000000000000000000000000000000000000000000000000","requestsHash":"0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","milliTimestamp":"0x19747a79524"}"#,
        );

        dbg!(header.hash());
        dbg!(header);
    }
}
