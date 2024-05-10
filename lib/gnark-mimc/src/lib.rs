use std::marker::PhantomData;

use ark_ff::PrimeField;
use num_bigint::BigUint;
use sha3::Digest;
use unionlabs::{
    errors::{ExpectedLength, InvalidLength},
    hash::H256,
};

// https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bn254/fr/mimc/mimc.go#L31
pub const GNARK_SEED: &[u8] = b"seed";

// https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bn254/fr/mimc/mimc.go#L30
pub const GNARK_BN254_ROUNDS: usize = 110;
// https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bn254/fr/mimc/mimc.go#L158
pub const GNARK_BN254_E: u64 = 5;

// https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bls12-377/fr/mimc/mimc.go#L30C17-L30C19
pub const GNARK_BLS12_377_ROUNDS: usize = 62;
// https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bls12-377/fr/mimc/mimc.go#L158
pub const GNARK_BLS12_377_E: u64 = 17;

pub type MiMCBn254Constants = MiMCConstants<ark_bn254::Fr, { GNARK_BN254_ROUNDS }>;
pub type MiMCBn254<'a> = MiMC<'a, ark_bn254::Fr, { GNARK_BN254_ROUNDS }, { GNARK_BN254_E }>;

pub type MiMCBls12377Constants = MiMCConstants<ark_bls12_377::Fr, { GNARK_BLS12_377_ROUNDS }>;
pub type MiMCBls12377<'a> =
    MiMC<'a, ark_bls12_377::Fr, { GNARK_BLS12_377_ROUNDS }, { GNARK_BLS12_377_E }>;

pub fn new_mimc_constants_bls12_377() -> MiMCBls12377Constants {
    MiMCConstants::new(GNARK_SEED)
}

pub fn new_mimc_bls12_377(constants: &MiMCBls12377Constants) -> MiMCBls12377 {
    MiMC::new(&constants)
}

pub fn mimc_sum_bl12377(
    constants: &MiMCBls12377Constants,
    elements: impl AsRef<[u8]>,
) -> Result<H256, Error> {
    Ok(new_mimc_bls12_377(constants)
        .update(elements)?
        .finalize()
        .try_into()
        .expect("impossible"))
}

pub fn new_mimc_constants_bn254() -> MiMCConstants<ark_bn254::Fr, { GNARK_BN254_ROUNDS }> {
    MiMCConstants::new(GNARK_SEED)
}

pub fn new_mimc_bn254(constants: &MiMCBn254Constants) -> MiMCBn254 {
    MiMC::new(&constants)
}

pub fn mimc_sum_bn254(
    constants: &MiMCBn254Constants,
    elements: impl AsRef<[u8]>,
) -> Result<H256, Error> {
    Ok(new_mimc_bn254(constants)
        .update(elements)?
        .finalize()
        .try_into()
        .expect("impossible"))
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("invalid length {0}")]
    InvalidLength(InvalidLength),
    #[error("invalid field element: {value:?}")]
    InvalidFieldElement { value: Vec<u8> },
}

#[derive(Debug, PartialEq)]
pub struct MiMCConstants<F, const K: usize>([F; K]);

impl<F, const K: usize> AsRef<[F; K]> for MiMCConstants<F, K> {
    fn as_ref(&self) -> &[F; K] {
        &self.0
    }
}

impl<F: PrimeField, const K: usize> MiMCConstants<F, K> {
    // TODO: move this to build.rs as a constant preset
    // https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bn254/fr/mimc/mimc.go#L179
    pub fn new(seed: &[u8]) -> Self {
        let keccak = |x: &[u8]| sha3::Keccak256::new().chain_update(x).finalize();
        let round_zero = keccak(seed);
        let (_, constants) =
            (0..K).fold((round_zero, [F::zero(); K]), |(round, mut constants), i| {
                let constant = keccak(&round);
                constants[i] = F::from_be_bytes_mod_order(&constant);
                (constant, constants)
            });
        Self(constants)
    }
}

pub struct MiMC<'a, F, const K: usize, const E: u64> {
    constants: &'a MiMCConstants<F, K>,
    data: Vec<F>,
    _marker: PhantomData<fn()>,
}

impl<'a, F: PrimeField, const K: usize, const E: u64> MiMC<'a, F, K, E> {
    pub const FIELD_ELEMENT_BYTES_LEN: usize =
        (F::MODULUS_BIT_SIZE.next_power_of_two() / 8) as usize;

    pub fn new(constants: &'a MiMCConstants<F, K>) -> Self {
        Self {
            _marker: PhantomData,
            constants,
            data: Vec::default(),
        }
    }

    // https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bn254/fr/mimc/mimc.go#L105
    pub fn update(mut self, elements: impl AsRef<[u8]>) -> Result<Self, Error> {
        // Slight difference, we only accept a multiple of the field. No hidden, implicit padding.
        let elements = elements.as_ref();
        if elements.len() % Self::FIELD_ELEMENT_BYTES_LEN != 0 {
            return Err(Error::InvalidLength(InvalidLength {
                expected: ExpectedLength::Exact(0),
                found: elements.len(),
            }));
        }
        let nb_of_field_elements = elements.len() / Self::FIELD_ELEMENT_BYTES_LEN;
        for i in 0..nb_of_field_elements {
            self.data.push(
                F::from_bigint(
                    BigUint::from_bytes_be(
                        &elements[i * Self::FIELD_ELEMENT_BYTES_LEN
                            ..i * Self::FIELD_ELEMENT_BYTES_LEN + Self::FIELD_ELEMENT_BYTES_LEN],
                    )
                    .try_into()
                    .map_err(|_| Error::InvalidFieldElement {
                        value: elements.to_vec(),
                    })?,
                )
                .ok_or(Error::InvalidFieldElement {
                    value: elements.to_vec(),
                })?,
            )
        }
        Ok(self)
    }

    // Inlined version of:
    // https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bn254/fr/mimc/mimc.go#L169
    pub fn finalize(&self) -> Vec<u8> {
        let sum = self.data.iter().fold(F::zero(), |k_acc, &data_i| {
            // encrypt
            let r = self.constants.as_ref().iter().fold(data_i, |m_acc, c_i| {
                // (m + k + c) ^ e
                let m_k_c = m_acc + k_acc + c_i;
                F::pow(&m_k_c, [E])
            }) + k_acc;
            k_acc + r + data_i
        });
        // Extremely ugly interface from arkworks, not gonna lie
        let mut buffer = vec![0u8; Self::FIELD_ELEMENT_BYTES_LEN];
        sum.serialize_uncompressed(&mut buffer[..])
            .expect("impossible");
        // No way to provide endianness when serializing, ark is little, gnark is big
        buffer.reverse();
        buffer
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{MiMC, MiMCConstants, GNARK_BN254_E, GNARK_BN254_ROUNDS, GNARK_SEED};

    #[derive(Deserialize)]
    struct Test {
        #[serde(rename = "in")]
        #[serde(with = "::serde_utils::hex_string_list")]
        i: Vec<Vec<u8>>,
        #[serde(rename = "out")]
        #[serde(with = "::serde_utils::hex_string")]
        o: Vec<u8>,
    }

    // The hex strings have been padded to even number of characters
    // https://github.com/Consensys/gnark-crypto/blob/564b6f724c3beac52d805e6e600d0a1fda9770b5/ecc/bn254/fr/mimc/test_vectors/vectors.json
    #[test]
    fn test_bn254_gnark_vectors() {
        let tests_vector = r#"
          [
	          {
		          "in": [
			          "0x105afe02a0f7648bee1669b05bf7ae69a37dbb6c86ebbee325dffe97ac1f8e64"
		          ],
		          "out": "0x263b9e754e6c611d646e65b16c48f51ab7bc0abedfae9c6ea04e2814ed28daf4"
	          },
	          {
		          "in": [
			          "0x00bc35f0589078e34d9139a357175d0e74b843e3de2f56bb6a5f18032ff3f627"
		          ],
		          "out": "0x103e2c8f50dec5248fd68f9778429252364ff239b123977a697356082f524f25"
	          },
	          {
		          "in": [
			          "0x208f0b283064057cf912b65eaa51e2cb2b85fdbe2fd0b2841f4bca59321ef1bf",
			          "0x226bee7671296d05c998a5b5b4b1d25f478696d5997ba4f4be1a682c56a69e11"
		          ],
		          "out": "0x1476ada1433d73817a69e45c84c5d452ad858f2dfdb1f7e4da203d3c4fd42222"
	          },
	          {
		          "in": [
			          "0x00995d448ab1fc86dd4874ebcbc0a7eea41acbe2c76e300aa73a1a0e63d5bc1b",
			          "0x2190a93f59d9f8cbb4f6236c5b7bf511aec80e88bec71dad4f5bbba9346ff5e4"
		          ],
		          "out": "0x0cd4ef5556a9413b6bb98d12aba6ed9b937f0adce41ba618a212fdcb1629737a"
	          },
	          {
		          "in": [
			          "0x06680de43f6cf410d4a8ed2893e58a8b740bac14f9dbdadbc8623c06027418a1",
			          "0x059323b0ab7043f559674eba263da812eae9e933b0c1bad55f8118d0caaa7479",
			          "0x16b161c8de7184ccc6b1b6fcddb562789a68eeaec174376f1157dfb3db310787"
		          ],
		          "out": "0x118e5255aabe7a3b6a5dde6ca28de461d36f802653885c665745fc4e6ca0f709"
	          },
	          {
		          "in": [
			          "0x1ab45102976d9ec683b46e7e7b4163055d1ab768d6bbd56cf95f3bca15d58020",
			          "0x18ff125903dc8352ca63c7a436f0425b4b7ddf7e487fb9ffd30f151993571b57",
			          "0x2cbfaa412f4b612d611acaab79a9e1c06b7094d8754fdbc085db28f2e4dd09ab"
		          ],
		          "out": "0x025fa55a9896d91d9617d9512e061d754336816f748bf07566591ec5cf4680dd"
	          },
	          {
		          "in": [
			          "0x2eddc35df3778e61c6571bcad90ab41dbf3cb61f4fd203d1922eb4fafde99136",
			          "0x0905c2010ece23e26373b38b6fc8b3c932a59443af656fb164e22b2bcf940b5a",
			          "0x22e63a3eb565d13c42c7d520c7b6112534b1c666653452f743b80bcc2d878455",
			          "0x096dff377f354f792685a7e740e3024409c24a379425ff63e3ce320b1e9bc471"
		          ],
		          "out": "0x18ea2fd58b4f3274193f3e73bded685426f114a8e7bc373c1aee3e6f0125787b"
	          },
	          {
		          "in": [
			          "0x05f3e89a9418877cd586de7c5cb061e6701a1bd69074cc7bd97c7c39d8f955eb",
			          "0x20cdf81f33b895b442d47357bd80e1eca03f410d808324f6d151dc68ab354a1f",
			          "0x12f4c27e5a2e80dd67fb33928c4e6219a8bdc89b498ed32acb02d725cec90076",
			          "0x1d6b52c237f0f74f0c50755627eed2610608488b54b0a3941a4623b1d435232a"
		          ],
		          "out": "0x2f237dea4570779296e2866383740b8e9ccf59577f8ff729880dadb58ae34d47"
	          },
	          {
		          "in": [
			          "0x262c77f7fdef59c80e0a9d4ece6d18fb6d64ebaacfc21921f44c5adc19698c6a",
			          "0x087bb7a78b27d19c5a502fbb087e48785d2777cff15d7b493901a8e528b64ee0",
			          "0x2a8a0e2a793fdd5bc340857b355f2b4c00c2723cefdf8515bda5beef458fca2b",
			          "0x2d4232cb721888f71997377de5ca195a5ae03d3eb9c87d2c04ef3664759036da",
			          "0x2f623ee75518430e291d42e7aaa75f5291a1bbfed125426d39270046a26be35a"
		          ],
		          "out": "0x06246dee7e2d9560a074c50a06e6525e4a58395cea4a893c49d71e373f19b9d6"
	          },
	          {
		          "in": [
			          "0x14b09f9af90cafa8a4e508f5289a6868804f98d3a724162999193e6c4bf752ea",
			          "0x0727359808271f360a6136389a9e2d5b1bb6ff3e8c4125ca03005892446ac17d",
			          "0x2b4abbd9943b201c1f75754833684f9eb15728a2ba646c53c2614bea7c9b968b",
			          "0x08e0ddb80366c4c6c7dcb9090f4862d64ef40677d324a76a82e06ca33ad29a09",
			          "0x170e8c954ca7e6526b743e92f796488afe5083a9c549358f730659c3e1cdbafa"
		          ],
		          "out": "0x1a2e7cffb5183898a8f4f6d4699bc272665ebffbb9d095576d2e21c45f012358"
	          }
          ]
        "#;
        let tests = serde_json::from_str::<Vec<Test>>(tests_vector).unwrap();
        let constants = MiMCConstants::<ark_bn254::Fr, { GNARK_BN254_ROUNDS }>::new(GNARK_SEED);
        for test in tests {
            let input = test.i.into_iter().flatten().collect::<Vec<_>>();
            let sum = MiMC::<_, { GNARK_BN254_ROUNDS }, { GNARK_BN254_E }>::new(&constants)
                .update(&input)
                .unwrap()
                .finalize();
            assert_eq!(sum, test.o)
        }
    }
}
