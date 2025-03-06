use crate::{custom_types::Version, fork::Fork, slot::Slot};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode))]
pub struct ForkParameters {
    pub genesis_fork_version: Version,
    pub genesis_slot: Slot,
    pub altair: Fork,
    pub bellatrix: Fork,
    pub capella: Fork,
    pub deneb: Fork,
    // #[cfg_attr(feature = "serde", serde(default = "default_fork"))]
    // pub electra: Fork,
}

#[cfg(feature = "bincode")]
impl bincode::Decode for ForkParameters {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        // decoder.unclaim_bytes_read();

        Ok(Self {
            genesis_fork_version: bincode::Decode::decode(decoder)?,
            genesis_slot: bincode::Decode::decode(decoder)?,
            altair: bincode::Decode::decode(decoder)?,
            bellatrix: bincode::Decode::decode(decoder)?,
            capella: bincode::Decode::decode(decoder)?,
            deneb: bincode::Decode::decode(decoder)?,
            // electra: bincode::Decode::decode(decoder).or_else(|e| match dbg!(e) {
            //     bincode::error::DecodeError::UnexpectedEnd { additional: 4 } => Ok(default_fork()),
            //     e => Err(e),
            // })?,
        })
    }
}

#[cfg(feature = "bincode")]
bincode::impl_borrow_decode!(ForkParameters);

// #[cfg(test)]
// mod tests {
//     use hex_literal::hex;
//     use unionlabs::encoding::{Bincode, DecodeAs};

//     use super::*;

//     #[test]
//     fn bincode_default() {
//         let encoded_pre_electra = hex!("010203040100000000000000020304050200000000000000030405060300000000000000040506070400000000000000050607080500000000000000");

//         let fork_parameters = ForkParameters::decode_as::<Bincode>(&encoded_pre_electra).unwrap();

//         assert_eq!(
//             fork_parameters,
//             ForkParameters {
//                 genesis_fork_version: Version(hex!("01020304").into()),
//                 genesis_slot: Slot(1),
//                 altair: Fork {
//                     version: Version(hex!("02030405").into()),
//                     epoch: 2,
//                 },
//                 bellatrix: Fork {
//                     version: Version(hex!("03040506").into()),
//                     epoch: 3,
//                 },
//                 capella: Fork {
//                     version: Version(hex!("04050607").into()),
//                     epoch: 4,
//                 },
//                 deneb: Fork {
//                     version: Version(hex!("05060708").into()),
//                     epoch: 5,
//                 },
//                 electra: default_fork(),
//             }
//         );
//     }
// }
