use ethabi::ethereum_types::{Address, U256};
use sha3::Digest;

pub struct TakeLastXBytes(pub usize);

pub enum SolidityDataType<'a> {
    String(&'a str),
    Address(Address),
    Bytes(&'a [u8]),
    Bool(bool),
    Number(U256),
    NumberWithShift(U256, TakeLastXBytes),
}

/// Calculates the slot for a `path` at saved in the commitment map in `slot`
///
/// key: keccak256(keccak256(abi.encode_packed(path)) || slot)
pub fn generate_commitment_key<S: Into<U256>>(path: String, slot: S) -> Vec<u8> {
    let (encoded_key, _) = abi::encode_packed(&[SolidityDataType::String(&path)]);
    let mut h1 = sha3::Keccak256::new();
    h1.update(encoded_key);

    let mut slot_bytes = vec![0; 32];
    let slot: U256 = slot.into();
    slot.to_big_endian(&mut slot_bytes);

    sha3::Keccak256::new()
        .chain_update(h1.finalize())
        .chain_update(&slot_bytes)
        .finalize()
        .to_vec()
}

// pub fn encode_cometbls_consensus_state(data: RawCometConsensusState) -> Result<Vec<u8>, Error> {
//     Ok(ethabi::encode(&[ethabi::Token::Tuple(vec![
//         ethabi::Token::FixedBytes(data.root.clone().ok_or(Error::MissingProtoField)?.hash),
//         ethabi::Token::FixedBytes(data.next_validators_hash),
//     ])]))
// }

// pub fn encode_cometbls_client_state(data: RawCometClientState) -> Result<Vec<u8>, Error> {
//     Ok(ethabi::encode(&[ethabi::Token::Tuple(vec![
//         ethabi::Token::String(data.chain_id.clone()),
//         ethabi::Token::Tuple(vec![
//             ethabi::Token::Int(
//                 data.trust_level
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .numerator
//                     .into(),
//             ),
//             ethabi::Token::Int(
//                 data.trust_level
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .denominator
//                     .into(),
//             ),
//         ]),
//         ethabi::Token::Tuple(vec![
//             ethabi::Token::Int(
//                 data.trusting_period
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .seconds
//                     .into(),
//             ),
//             ethabi::Token::Int(
//                 data.trusting_period
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .nanos
//                     .into(),
//             ),
//         ]),
//         ethabi::Token::Tuple(vec![
//             ethabi::Token::Int(
//                 data.unbonding_period
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .seconds
//                     .into(),
//             ),
//             ethabi::Token::Int(
//                 data.unbonding_period
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .nanos
//                     .into(),
//             ),
//         ]),
//         ethabi::Token::Tuple(vec![
//             ethabi::Token::Int(
//                 data.max_clock_drift
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .seconds
//                     .into(),
//             ),
//             ethabi::Token::Int(
//                 data.max_clock_drift
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .nanos
//                     .into(),
//             ),
//         ]),
//         ethabi::Token::Tuple(vec![
//             ethabi::Token::Int(
//                 data.frozen_height
//                     .clone()
//                     .ok_or(Error::MissingProtoField)?
//                     .revision_number
//                     .into(),
//             ),
//             ethabi::Token::Int(
//                 data.frozen_height
//                     .ok_or(Error::MissingProtoField)?
//                     .revision_height
//                     .into(),
//             ),
//         ]),
//     ])]))
// }

pub mod abi {
    use super::SolidityDataType;

    /// Pack a single `SolidityDataType` into bytes
    fn pack(data_type: &SolidityDataType) -> Vec<u8> {
        let mut res = Vec::new();
        match data_type {
            SolidityDataType::String(s) => {
                res.extend(s.as_bytes());
            }
            SolidityDataType::Address(a) => {
                res.extend(a.0);
            }
            SolidityDataType::Number(n) => {
                for b in n.0.iter().rev() {
                    let bytes = b.to_be_bytes();
                    res.extend(bytes);
                }
            }
            SolidityDataType::Bytes(b) => {
                res.extend(*b);
            }
            SolidityDataType::Bool(b) => {
                if *b {
                    res.push(1);
                } else {
                    res.push(0);
                }
            }
            SolidityDataType::NumberWithShift(n, to_take) => {
                let local_res = n.0.iter().rev().fold(vec![], |mut acc, i| {
                    let bytes = i.to_be_bytes();
                    acc.extend(bytes);
                    acc
                });

                let to_skip = local_res.len() - (to_take.0 / 8);
                let local_res = local_res.into_iter().skip(to_skip).collect::<Vec<u8>>();
                res.extend(local_res);
            }
        };
        res
    }

    pub fn encode_packed(items: &[SolidityDataType]) -> (Vec<u8>, String) {
        let res = items.iter().fold(Vec::new(), |mut acc, i| {
            let pack = pack(i);
            acc.push(pack);
            acc
        });
        let res = res.join(&[][..]);
        let hexed = hex::encode(&res);
        (res, hexed)
    }
}
