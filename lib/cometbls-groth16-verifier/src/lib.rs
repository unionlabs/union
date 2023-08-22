#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use core::ops::{AddAssign, Neg};

use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::{vec, vec::Vec, BigInt, Field, PrimeField, QuadExtField};
use ark_groth16::VerifyingKey;
use ark_serialize::CanonicalDeserialize;
use ethabi::{ethereum_types::U256, ParamType, Token};
use hex_literal::hex;
use sha3::Digest;

// TODO: link whitepaper equation (hmac)
pub const HMAC_O: &[u8] = &hex!("1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C");
pub const HMAC_I: &[u8] = &hex!("75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636");
pub const PRIME_R_MINUS_ONE: U256 = U256([
    4891460686036598784,
    2896914383306846353,
    13281191951274694749,
    3486998266802970665,
]);

fn make_g1(x: BigInt<4>, y: BigInt<4>) -> ark_bn254::G1Affine {
    ark_bn254::G1Affine::new(x.into(), y.into())
}

fn make_g2(x0: BigInt<4>, x1: BigInt<4>, y0: BigInt<4>, y1: BigInt<4>) -> ark_bn254::G2Affine {
    ark_bn254::G2Affine::new(
        QuadExtField {
            c0: x1.into(),
            c1: x0.into(),
        },
        QuadExtField {
            c0: y1.into(),
            c1: y0.into(),
        },
    )
}

pub fn devnet_vk() -> VerifyingKey<ark_bn254::Bn254> {
    VerifyingKey::<ark_bn254::Bn254> {
        alpha_g1: make_g1(
            BigInt!("1125671039621810089428171450188893727232365974023656248475833836938972484667"),
            BigInt!(
                "16708059979227448575127783600120361097122367964683038521677601550114485714490"
            ),
        ),
        beta_g2: make_g2(
            BigInt!(
                "16470019197094627162299087301831442275638218677130007222670205426156957100865"
            ),
            BigInt!(
                "14958059389656804321793202880891013104349254812250653654254419540045874479613"
            ),
            BigInt!("6462132794450663952942967898368307576456675700257099429655713611570640619298"),
            BigInt!(
                "17123404116076159959762469925340895612001057027173989919807096071077171764514"
            ),
        ),
        gamma_g2: make_g2(
            BigInt!(
                "11418548283192814327818020566212000543664079199800580120911544819799726125734"
            ),
            BigInt!("5920641909552990144642456894725912706474505958697724624280513022734799073628"),
            BigInt!(
                "13424292823323948119718142680493017498112842428096491321197851435006803504886"
            ),
            BigInt!("9535504282195516941649015444320702490439814259887213402473830583601855984569"),
        ),
        delta_g2: make_g2(
            BigInt!(
                "11704745517448317616837684982261126400037964268545711184519502786602539467064"
            ),
            BigInt!(
                "16888938809769781383314095342387355965362215790775205037422449393153218364207"
            ),
            BigInt!("8414763178397780322774566660669074914723220818750198492373048213419596194001"),
            BigInt!(
                "13109301182837003690701475443653586480186945433257156209129736993852895513512"
            ),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "18767338542014411857313993782381800397387924826542899835474681706360337468968"
                ),
                BigInt!(
                    "17085667352205594924878160128155941314151724412939678808952186650026637982496"
                ),
            ),
            make_g1(
                BigInt!(
                    "12504772314995318667085598268489910500204762055609759048236479468946268434715"
                ),
                BigInt!(
                    "8983729810354454237350709663955361486255804349684341343883945534227372772583"
                ),
            ),
            make_g1(
                BigInt!(
                    "21771893660210241160978984895410580596702195089317438720917623130541287562951"
                ),
                BigInt!(
                    "6826939490233329137937827410099630746030273673600789410359384510477424990528"
                ),
            ),
            make_g1(
                BigInt!(
                    "13483374748428281059028219899599319192039698231059011030408926432855884913059"
                ),
                BigInt!(
                    "1580546223836162786043487702113163279370733460000671658613748752775709896738"
                ),
            ),
            make_g1(
                BigInt!(
                    "878844219607870183522904288494135794814180324863533284370581898731325763878"
                ),
                BigInt!(
                    "5029967920102374989452979629721980248435538706167040492456898113056724473757"
                ),
            ),
            make_g1(
                BigInt!(
                    "19178552108232008347712559118598665047168074310079689947325291874527681136632"
                ),
                BigInt!(
                    "4887450214492350443087393893730872155113124252746183476199372947155640792254"
                ),
            ),
            make_g1(
                BigInt!(
                    "4357893561187570586884283989859896948424540980085231995327372677863129685190"
                ),
                BigInt!(
                    "17366381213742021981454676402842182355603824163159232749914711635677740694004"
                ),
            ),
            make_g1(
                BigInt!(
                    "11447136153403079013535596580526837954163849580215646916316817462954879467374"
                ),
                BigInt!(
                    "4022142721279929532160588989277562498040471874620641926810806863247508170238"
                ),
            ),
        ],
    }
}

pub fn testnet_vk() -> VerifyingKey<ark_bn254::Bn254> {
    VerifyingKey::<ark_bn254::Bn254> {
        alpha_g1: make_g1(
            BigInt!(
                "10900365700008785951386810059031907651998862503081677518760135848615814781151"
            ),
            BigInt!("2813918487984701514495866538640757245795310074506986809086187064257311477759"),
        ),
        beta_g2: make_g2(
            BigInt!(
                "18016272642940762206675806612642011201735472249051322975835744218682902713062"
            ),
            BigInt!(
                "16357750055644777342678346173266324254774518645338614913789308235366025659928"
            ),
            BigInt!(
                "19758911916407078708266271128615419628883042320273723741836641386408409501327"
            ),
            BigInt!("3554513121429906029657855924332897856861387724805388158790066103361201204210"),
        ),
        gamma_g2: make_g2(
            BigInt!(
                "19512875184572867569120416814748692776010528469274793083343283869434864471759"
            ),
            BigInt!("2549029521023100058468087326457927716968672157213288863707354374956257266809"),
            BigInt!("160761816387853926196115454931163650061973435705108488809338003258641051841"),
            BigInt!(
                "14034778810220884153712266420860367207624938118479081815653668110709504182639"
            ),
        ),
        delta_g2: make_g2(
            BigInt!(
                "10617559592538378308874786090228720967765022744707492676272766520035077403025"
            ),
            BigInt!(
                "12652367968539648179486762187587112566335954262614105271143741065670049337363"
            ),
            BigInt!("5377251285991785595312700760925386441208533509670674647478120822710314216997"),
            BigInt!(
                "12369114061254266277902937238404080678011257369662724502280673834418158554593"
            ),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "15306751257115996922046245789904024812171305466642942480064397212755507058688"
                ),
                BigInt!(
                    "12759999967270786331387453373858589735971137181671080110315892782079676131874"
                ),
            ),
            make_g1(
                BigInt!(
                    "3548223831724705000195252499885090339557210575683668213682733289524309515359"
                ),
                BigInt!(
                    "21632634098450235872830583689357432925039094162224165558266192563872100625705"
                ),
            ),
            make_g1(
                BigInt!(
                    "4321119025430213864295153592563322246832740792995914245200478999233805861628"
                ),
                BigInt!(
                    "18616911277202834760914976241332791800643185228033392418467404146733793044730"
                ),
            ),
            make_g1(
                BigInt!(
                    "15685223153963246445919442544425267373966467610106703623226227387118544549432"
                ),
                BigInt!(
                    "11473518119258233867048477098089798495777464828572892625319391999598481585842"
                ),
            ),
            make_g1(
                BigInt!(
                    "432736116405447307808714121564973089819517564416720461274423091613737786793"
                ),
                BigInt!(
                    "19517500262453478353076874678858196315854073460273448312742058224018278386413"
                ),
            ),
            make_g1(
                BigInt!(
                    "4379712019656232179050299831603961143239683484019111611172386826702435579618"
                ),
                BigInt!(
                    "6937816401058425690911323681153605066038715139288351049476647311903574154552"
                ),
            ),
            make_g1(
                BigInt!(
                    "21763089232083212206036601850568492720324890876918101295522159373088664991468"
                ),
                BigInt!(
                    "20840260120803223232278551010977437167156113810525213523555749327205351100725"
                ),
            ),
            make_g1(
                BigInt!(
                    "12684646261113662286697672937698669711405584900450108554594447254498940598914"
                ),
                BigInt!(
                    "2446400168549826690903655782917719879593427261438382262680944092055169957518"
                ),
            ),
        ],
    }
}

// TODO: optimize?
// TODO: link whitepaper equation
fn hmac_keccak(message: &[u8]) -> [u8; 32] {
    let mut hasher = sha3::Keccak256::new();
    hasher.update(
        HMAC_I
            .iter()
            .copied()
            .chain(message.to_vec())
            .into_iter()
            .collect::<Vec<_>>(),
    );
    let inner_hash = hasher.finalize();

    let mut hasher = sha3::Keccak256::new();
    hasher.update(
        HMAC_O
            .iter()
            .copied()
            .chain(inner_hash.to_vec())
            .into_iter()
            .collect::<Vec<_>>(),
    );

    hasher.finalize().into()
}

// TODO: optimize
// TODO: link whitepaper equation
fn hash_to_field2(message: &[u8]) -> (U256, U256) {
    (
        hash_to_field(
            vec![0]
                .into_iter()
                .chain(message.to_vec())
                .into_iter()
                .collect::<Vec<_>>()
                .as_ref(),
        ),
        hash_to_field(
            vec![1]
                .into_iter()
                .chain(message.to_vec())
                .into_iter()
                .collect::<Vec<_>>()
                .as_ref(),
        ),
    )
}

// TODO: link whitepaper equation
fn hash_to_field(message: &[u8]) -> U256 {
    (U256::from(&hmac_keccak(message)) % PRIME_R_MINUS_ONE) + 1
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EthAbiDecoding,
    InvalidPoint,
    InvalidProof,
}

pub fn verify_zkp(
    vk: VerifyingKey<ark_bn254::Bn254>,
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    zkp: &[u8],
) -> Result<(), Error> {
    verify_generic_zkp::<ark_bn254::Bn254>(
        vk,
        trusted_validators_hash,
        untrusted_validators_hash,
        message,
        zkp,
    )
}

// TODO: optimize as the ETH encoding is linear without prefix for
// U256 => we can directly read the zkp bytes
fn verify_generic_zkp<P: Pairing>(
    vk: VerifyingKey<P>,
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    zkp: &[u8],
) -> Result<(), Error> {
    let values = ethabi::decode(
        &[
            // a = G1
            ParamType::Int(256),
            ParamType::Int(256),
            // b = G2
            ParamType::Int(256),
            ParamType::Int(256),
            ParamType::Int(256),
            ParamType::Int(256),
            // c = G1
            ParamType::Int(256),
            ParamType::Int(256),
            // commitmentHash
            ParamType::Int(256),
            // proofCommitment
            ParamType::Int(256),
            ParamType::Int(256),
        ],
        zkp,
    )
    .map_err(|_| Error::EthAbiDecoding)?;

    let (neg_a, b, c, commitment_hash, proof_commitment) = match &values[..] {
        &[Token::Int(a_x), Token::Int(a_y), Token::Int(b_x0), Token::Int(b_x1), Token::Int(b_y0), Token::Int(b_y1), Token::Int(c_x), Token::Int(c_y), Token::Int(commitment_hash), Token::Int(proof_commitment_x), Token::Int(proof_commitment_y)] =>
        {
            let mut buffer_g1 = [0u8; 64];
            let mut decode_g1 = move |x: U256, y: U256| {
                x.to_little_endian(&mut buffer_g1[0..32]);
                y.to_little_endian(&mut buffer_g1[32..64]);
                <P::G1Affine as CanonicalDeserialize>::deserialize_uncompressed(&buffer_g1[..])
                    .map_err(|_| Error::InvalidPoint)
            };

            let mut buffer_g2 = [0u8; 128];
            let mut decode_g2 = move |x0: U256, x1: U256, y0: U256, y1: U256| {
                x1.to_little_endian(&mut buffer_g2[0..32]);
                x0.to_little_endian(&mut buffer_g2[32..64]);
                y1.to_little_endian(&mut buffer_g2[64..96]);
                y0.to_little_endian(&mut buffer_g2[96..128]);
                <P::G2Affine as CanonicalDeserialize>::deserialize_uncompressed(&buffer_g2[..])
                    .map_err(|_| Error::InvalidPoint)
            };

            let a = decode_g1(a_x, a_y)?;
            let b = decode_g2(b_x0, b_x1, b_y0, b_y1)?;
            let c = decode_g1(c_x, c_y)?;
            let proof_commitment = decode_g1(proof_commitment_x, proof_commitment_y)?;

            Ok((
                P::G1Prepared::from(a.into_group().neg()),
                P::G2Prepared::from(b),
                P::G1Prepared::from(c),
                commitment_hash,
                proof_commitment,
            ))
        }
        _ => Err(Error::EthAbiDecoding),
    }?;

    let mut buffer_scalar = [0u8; 32];
    let mut decode_scalar = move |x: U256| {
        x.to_little_endian(&mut buffer_scalar);
        // NOTE: This would silently fail if the input do not fit the scalar
        // field, which is unlikely to happen unless the parameters have been
        // tampered. The pairing check would obviously fail in this case.
        <P::ScalarField as PrimeField>::from_le_bytes_mod_order(&buffer_scalar)
    };

    let mut trusted_validators_hash_high = trusted_validators_hash;
    trusted_validators_hash_high >>= 128;
    let trusted_validators_hash_low = trusted_validators_hash.low_u128().into();

    let mut untrusted_validators_hash_high = untrusted_validators_hash;
    untrusted_validators_hash_high >>= 128;
    let untrusted_validators_hash_low = untrusted_validators_hash.low_u128().into();

    let (message_x, message_y) = hash_to_field2(message);

    let public_inputs: [P::ScalarField; 7] = [
        decode_scalar(trusted_validators_hash_high),
        decode_scalar(trusted_validators_hash_low),
        decode_scalar(untrusted_validators_hash_high),
        decode_scalar(untrusted_validators_hash_low),
        decode_scalar(message_x),
        decode_scalar(message_y),
        decode_scalar(commitment_hash),
    ];

    let mut g_ic = vk.gamma_abc_g1[0].into_group();
    // Gnark specific, we need to aggregate the proof commitment
    // See https://github.com/ConsenSys/gnark/issues/652
    g_ic.add_assign(proof_commitment);
    for (i, b) in public_inputs
        .into_iter()
        .zip(vk.gamma_abc_g1.iter().skip(1))
    {
        g_ic.add_assign(&b.mul_bigint(i.into_bigint()));
    }

    let result = P::multi_pairing(
        [neg_a, vk.alpha_g1.into(), g_ic.into_affine().into(), c],
        [b, vk.beta_g2.into(), vk.gamma_g2.into(), vk.delta_g2.into()],
    );

    if result.0 == P::TargetField::ONE {
        Ok(())
    } else {
        Err(Error::InvalidProof)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use ethabi::ethereum_types::U256;

    use super::*;

    #[test]
    fn test_valid() {
        assert_eq!(
            verify_zkp(
                testnet_vk(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                &hex!("650802113e0200000000000022480a207022627e60ed78120d2fe8dc7acdb58a2321b0304f8912d2dfb86ce038e23ca812240801122041b8793236ee0980e2eaf1a2fad268c4a3d8979a0c432f06e284eec5e74dd69c320e756e696f6e2d6465766e65742d31"),
                &hex!("25670583A18A0FA734EE839824AEB2EFAF00F2704178C951B70A01E956C164F32CA7B62707FF3916D88F02F67C1C9334C1EC929F37551212DFCF667903C93C2E0E4D493A02092736D6ADD9A66AAE2B55028FA72FB6137639547BBF4C47EB073E2BB2BE616A4182F3B278C7185E4D21EE535BBA1F44F260D23F869F3E2B3F27400318AAC18834CBDE7001AB47637B05ADDF2C0101CCC1BED2BAB0981AB76225F4212F72E61FED29327F9C81E06DB3C9B67FBF6542BF7742CE807DD0B38134DD652C01BB21CF6B5C01AC3C1E749E9E6859DCD8FAA24C32AC976CD5EF8989E37D6D2896AE7082AC48A94B1BF6BCFCAC412EAD66A22986366C78FA8072060DCC95781159E6255C367EAFBFDAE0C611935C2E6FEEA3F76810FBA9F95FA45700EFA5A017D399707E896688C2CCBB13D014D5189F523D6912AE3D01D0AE5F2EC6B05FA80466F421D4936925454BB6941FD367C93AC498C2CE3503DCB41A58C0C437F39E")
            ),
            Ok(())
        );
    }

    #[test]
    fn test_invalid_vote() {
        assert_eq!(
            verify_zkp(
                testnet_vk(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                &hex!("650802113e0200000000000022480a207022627e60ed78120d2fe8dc7acdb58a2321b0304f8912d2dfb86ce038e23ca812240801122041b8793236ee0980e2eaf1a2fad268c4a3d8979a0c432f06e284eec5e74dd69c320e756e696f6e2d6465766e65742d32"),
                &hex!("25670583A18A0FA734EE839824AEB2EFAF00F2704178C951B70A01E956C164F32CA7B62707FF3916D88F02F67C1C9334C1EC929F37551212DFCF667903C93C2E0E4D493A02092736D6ADD9A66AAE2B55028FA72FB6137639547BBF4C47EB073E2BB2BE616A4182F3B278C7185E4D21EE535BBA1F44F260D23F869F3E2B3F27400318AAC18834CBDE7001AB47637B05ADDF2C0101CCC1BED2BAB0981AB76225F4212F72E61FED29327F9C81E06DB3C9B67FBF6542BF7742CE807DD0B38134DD652C01BB21CF6B5C01AC3C1E749E9E6859DCD8FAA24C32AC976CD5EF8989E37D6D2896AE7082AC48A94B1BF6BCFCAC412EAD66A22986366C78FA8072060DCC95781159E6255C367EAFBFDAE0C611935C2E6FEEA3F76810FBA9F95FA45700EFA5A017D399707E896688C2CCBB13D014D5189F523D6912AE3D01D0AE5F2EC6B05FA80466F421D4936925454BB6941FD367C93AC498C2CE3503DCB41A58C0C437F39E")),
            Err(Error::InvalidProof)
        );
    }

    #[test]
    fn test_invalid_untrusted_validators_hash() {
        assert_eq!(
            verify_zkp(
                testnet_vk(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065878").unwrap(),
                &hex!("650802113e0200000000000022480a207022627e60ed78120d2fe8dc7acdb58a2321b0304f8912d2dfb86ce038e23ca812240801122041b8793236ee0980e2eaf1a2fad268c4a3d8979a0c432f06e284eec5e74dd69c320e756e696f6e2d6465766e65742d31"),
                &hex!("25670583A18A0FA734EE839824AEB2EFAF00F2704178C951B70A01E956C164F32CA7B62707FF3916D88F02F67C1C9334C1EC929F37551212DFCF667903C93C2E0E4D493A02092736D6ADD9A66AAE2B55028FA72FB6137639547BBF4C47EB073E2BB2BE616A4182F3B278C7185E4D21EE535BBA1F44F260D23F869F3E2B3F27400318AAC18834CBDE7001AB47637B05ADDF2C0101CCC1BED2BAB0981AB76225F4212F72E61FED29327F9C81E06DB3C9B67FBF6542BF7742CE807DD0B38134DD652C01BB21CF6B5C01AC3C1E749E9E6859DCD8FAA24C32AC976CD5EF8989E37D6D2896AE7082AC48A94B1BF6BCFCAC412EAD66A22986366C78FA8072060DCC95781159E6255C367EAFBFDAE0C611935C2E6FEEA3F76810FBA9F95FA45700EFA5A017D399707E896688C2CCBB13D014D5189F523D6912AE3D01D0AE5F2EC6B05FA80466F421D4936925454BB6941FD367C93AC498C2CE3503DCB41A58C0C437F39E")
            ),
            Err(Error::InvalidProof)
        );
    }

    #[test]
    fn test_invalid_trusted_validators_hash() {
        assert_eq!(
            verify_zkp(
                testnet_vk(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065878").unwrap(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                &hex!("650802113e0200000000000022480a207022627e60ed78120d2fe8dc7acdb58a2321b0304f8912d2dfb86ce038e23ca812240801122041b8793236ee0980e2eaf1a2fad268c4a3d8979a0c432f06e284eec5e74dd69c320e756e696f6e2d6465766e65742d31"),
                &hex!("25670583A18A0FA734EE839824AEB2EFAF00F2704178C951B70A01E956C164F32CA7B62707FF3916D88F02F67C1C9334C1EC929F37551212DFCF667903C93C2E0E4D493A02092736D6ADD9A66AAE2B55028FA72FB6137639547BBF4C47EB073E2BB2BE616A4182F3B278C7185E4D21EE535BBA1F44F260D23F869F3E2B3F27400318AAC18834CBDE7001AB47637B05ADDF2C0101CCC1BED2BAB0981AB76225F4212F72E61FED29327F9C81E06DB3C9B67FBF6542BF7742CE807DD0B38134DD652C01BB21CF6B5C01AC3C1E749E9E6859DCD8FAA24C32AC976CD5EF8989E37D6D2896AE7082AC48A94B1BF6BCFCAC412EAD66A22986366C78FA8072060DCC95781159E6255C367EAFBFDAE0C611935C2E6FEEA3F76810FBA9F95FA45700EFA5A017D399707E896688C2CCBB13D014D5189F523D6912AE3D01D0AE5F2EC6B05FA80466F421D4936925454BB6941FD367C93AC498C2CE3503DCB41A58C0C437F39E"
                )
            ),
            Err(Error::InvalidProof)
        );
    }

    // Tamper a point in the valid zkp
    #[test]
    fn test_invalid_point() {
        assert_eq!(
            verify_zkp(
                testnet_vk(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                &hex!("650802113e0200000000000022480a207022627e60ed78120d2fe8dc7acdb58a2321b0304f8912d2dfb86ce038e23ca812240801122041b8793236ee0980e2eaf1a2fad268c4a3d8979a0c432f06e284eec5e74dd69c320e756e696f6e2d6465766e65742d31"),
                &hex!("25670583A18A0FA734EE839824AEB2EFAF00F2704178C951B70A01E956C164F32CA7B62707FF3916D88F02F67C1C9334C1EC929F37551212DFCF667903C93C2E0E4D493A02092736D6ADD9A66AAE2B55028FA72FB6137639547BBF4C47EB073E2BB2BE616A4182F3B278C7185E4D21EE535BBA1F44F260D23F869F3E2B3F27400318AAC18834CBDE7001AB47637B05ADDF2C0101CCC1BED2BAB0981AB76225F4212F72E61FED29327F9C81E06DB3C9B67FBF6542BF7742CE807DD0B38134DD652C01BB21CF6B5C01AC3C1E749E9E6859DCD8FAA24C32AC976CD5EF8989E37D6D2896AE7082AC48A94B1BF6BCFCAC412EAD66A22986366C78FA8072060DCC95781159E6255C367EAFBFDAE0C611935C2E6FEEA3F76810FBA9F95FA45700EFA5A017D399707E896688C2CCBB13D014D5189F523D6912AE3D01D0AE5F2EC6B05FA80466F421D4936925454BB6941FD367C93AC498C2CE3503DCB41A58C0C437F39F")
            ),
            Err(Error::InvalidPoint)
        );
    }

    // Valid ZKP but not linked to this public inputs
    #[test]
    fn test_invalid_zkp() {
        assert_eq!(
            verify_zkp(
                testnet_vk(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                U256::from_str("0x1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                &hex!("650802113e0200000000000022480a207022627e60ed78120d2fe8dc7acdb58a2321b0304f8912d2dfb86ce038e23ca812240801122041b8793236ee0980e2eaf1a2fad268c4a3d8979a0c432f06e284eec5e74dd69c320e756e696f6e2d6465766e65742d31"),
                &hex!("09f57b8b308d9c57bd1e30cd493212314f5b680e685bc91402193fac45389c42064968d6db298707b5405431621b96bd73756907b2b5137ca4966c270d0b9b461a60936e4cdf9b77b993f25cdeb7d1c5623f082dc2c88b20d33a9b40c14dc39115aba4e371dc0443465b4d9b69aece3a4f15f0503c6d0f56dc1237356c32de80271ca20c7eb2bcb9bc56be7256a93d925fa3bae73829dbc53c4e9056f99046b80277d0bbc45741e6eb1e6a6a9e1d795f384cca1d3836e29ffecdebf6b0a9db5e15ec13c943d68283a8a781f4d5cb330ca1b02a7515990eb8c3c3e4da4ba9ef1717980acd29ff4c6ba58036337faae8def7355243b2449b9c5637f85ebecec1a42e4570af6b520476ccc96665d3d92dc7a22c0864169072332f17f7664223ea4004e3860aa093da597dc7f6b28284c45d9bc56e4d8e44ee5b784ec23b5309649116db1c88de8aaa9057b41b94939806fd8910bc1d5b33f3d4db7f568197c63f55")
            ),
            Err(Error::InvalidProof)
        );
    }
}
