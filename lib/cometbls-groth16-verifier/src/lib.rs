#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::{marker::PhantomData, ops::AddAssign};

use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::{vec, vec::Vec, BigInt, PrimeField, QuadExtField};
use ark_groth16::{Groth16, Proof, VerifyingKey};
use ark_relations::r1cs::SynthesisError;
use ark_serialize::CanonicalDeserialize;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use ethabi::ethereum_types::U256;
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
            c0: x0.into(),
            c1: x1.into(),
        },
        QuadExtField {
            c0: y0.into(),
            c1: y1.into(),
        },
    )
}

#[deprecated(note = "the circuit has been generalized, use universal_vk() instead")]
pub fn devnet_vk() -> VerifyingKey<ark_bn254::Bn254> {
    VerifyingKey::<ark_bn254::Bn254> {
        alpha_g1: make_g1(
            BigInt!("9974399132350238449672423145167802132344597176432790937987673566759904354712"),
            BigInt!(
                "10396217607362300103655122228113983820745493114140883199476303464408811706471"
            ),
        ),
        beta_g2: make_g2(
            BigInt!(
                "20043334460449324572644561653520106968487299991365945714189067590923833559557"
            ),
            BigInt!("3782843380964690766572041754552260909078546283792951053210110465664576118592"),
            BigInt!("4546441854933490265510538123407299251387870046105247930781926195493537303978"),
            BigInt!(
                "19728170969753285624598425791670262520539566544285475380089632156164753610432"
            ),
        ),
        gamma_g2: make_g2(
            BigInt!(
                "15890984819252760833184574925585572560291816058221856734884092043888365097798"
            ),
            BigInt!(
                "13558421301005029939663494790802233493306340917537858200716018199215933051901"
            ),
            BigInt!("8951430351447595274973237553867518771312837295026859105316664000150429223102"),
            BigInt!("9774001800913153454819154173343364291874345033268265728436390595601923216347"),
        ),
        delta_g2: make_g2(
            BigInt!(
                "12986197120328725341178217804701057807111123287171378211441714126957192190146"
            ),
            BigInt!("6358811827968308311530932341706580062890352807488954904621603172031504605990"),
            BigInt!(
                "11008158088064515525514471643307844090914721261889690007083299787176620957920"
            ),
            BigInt!("1693569334322206029064251989727378784021135124521538321367277282710375215047"),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "19536632576810938663755749582603546087222638703180042738513910916616519682978"
                ),
                BigInt!(
                    "6698373882991209028988452302062701111718632869665993606888642342207051892975"
                ),
            ),
            make_g1(
                BigInt!(
                    "2175229953105907030386086995813356912474746827628735806700482499160750847843"
                ),
                BigInt!(
                    "19823529752927772060409556160428145736017998454909758668189685129708026335065"
                ),
            ),
            make_g1(
                BigInt!(
                    "219999636782629863970338640713754993296807671982705311132408472476488701731"
                ),
                BigInt!(
                    "11582191598571666113262523487623760501658738560317219321241346601375876165826"
                ),
            ),
            make_g1(
                BigInt!(
                    "914394202216898966177299917746741778977940677187377639141420924936000943248"
                ),
                BigInt!(
                    "8726710514357051704626909121942479242019757832647898014481949563241929367905"
                ),
            ),
            make_g1(
                BigInt!(
                    "410530762185814800540583115824275203642834613850491151197240739569603959187"
                ),
                BigInt!(
                    "5236570818789858673799951129197614899816105385688852546833382795763613513196"
                ),
            ),
            make_g1(
                BigInt!(
                    "13915041915789362048532482320640272960446035437675260680928350524425298814782"
                ),
                BigInt!(
                    "4402873937379531482689066168118493057889537848402358898771477872149907606547"
                ),
            ),
            make_g1(
                BigInt!(
                    "13581186194999365488187594952848464234662365346750156291065587645871146629135"
                ),
                BigInt!(
                    "3315129916730707978366419309543430205662621536944423844626436472087928543555"
                ),
            ),
            make_g1(
                BigInt!(
                    "13424886217355741741339780135239743700963066884441662888102595067007931455321"
                ),
                BigInt!(
                    "5117715757204980109056335794282910227777795377124703228976341447232892223753"
                ),
            ),
        ],
    }
}

#[deprecated(note = "the circuit has been generalized, use universal_vk() instead")]
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

pub fn universal_vk() -> VerifyingKey<ark_bn254::Bn254> {
    VerifyingKey::<ark_bn254::Bn254> {
        alpha_g1: make_g1(
            BigInt!("9299508173272494929105865905051404951897332658127374605046930680488559435254"),
            BigInt!(
                "18449071807414126702267921501212908485048748329681149742020934064919667139808"
            ),
        ),
        beta_g2: make_g2(
            BigInt!("5096212357132918452251252704388234225082503905831131839538574831059776733824"),
            BigInt!("9871394374390025166143634557536008713523746310108852017986533167638746256254"),
            BigInt!(
                "11630913626067138601009400888928741950076197950077654728180339555088523423405"
            ),
            BigInt!("2994103247785810201667124008957848981310968782661487620522095886913084609572"),
        ),
        gamma_g2: make_g2(
            BigInt!("8728326510657556501512980085722704788008476669448966656332002127230351337993"),
            BigInt!("8364486770349664052628814504100073278919932121455819239030081350179461823255"),
            BigInt!("3986268596561538370989377729940576344664490279364331721506940985594782192171"),
            BigInt!("8369220198209372081204513643845585863738974245354439418988327483682903778265"),
        ),
        delta_g2: make_g2(
            BigInt!(
                "13803603616464982263018301486372576233533900013090456112322126315787178138538"
            ),
            BigInt!(
                "15824501983485336860943033585861669302973904504328308733732788275953716696237"
            ),
            BigInt!(
                "21593012186009510427175105404708291323627612061919152651390507502066844352715"
            ),
            BigInt!("8164899050757504836319827468282253277854985688387326676562281969478597813936"),
        ),
        gamma_abc_g1: vec![
            make_g1(
                BigInt!(
                    "15789004268582736978147923000559689535590265661013579550864224155679253949897"
                ),
                BigInt!(
                    "1378048636762829303302526908931588869607672607207967622330083376530840124919"
                ),
            ),
            make_g1(
                BigInt!(
                    "2222813113581775923098353798807000005994352347537085954054160366314881180586"
                ),
                BigInt!(
                    "10833463196202613792815504387972169711057769303503761560475306516141880631310"
                ),
            ),
            make_g1(
                BigInt!(
                    "18950818390236121779972718343821485543494748152479479013021078890349283103120"
                ),
                BigInt!(
                    "3967742757730080263928210937291599691657153408926332509289769604416050599011"
                ),
            ),
            make_g1(
                BigInt!(
                    "21291585552723056987506800233852525266397034135262812413707718006347913793952"
                ),
                BigInt!(
                    "17333157225545652033279143133357670093701423541263171910427135425472957379906"
                ),
            ),
            make_g1(
                BigInt!(
                    "14094587378870074193859910743132907717421241456133051336165176568216476766244"
                ),
                BigInt!(
                    "5252230952813796724951345120624842989580576711840279663039165584333195353196"
                ),
            ),
            make_g1(
                BigInt!(
                    "6364930290697954112798602306389412897383882966810085954781152845718560683087"
                ),
                BigInt!(
                    "11388818647852555134322205108540002610100119175840510285347738772685624275714"
                ),
            ),
        ],
    }
}

// TODO: link whitepaper equation
fn hmac_keccak(message: &[u8]) -> [u8; 32] {
    let mut hasher = sha3::Keccak256::new();
    hasher.update(
        HMAC_I
            .iter()
            .copied()
            .chain(message.iter().copied())
            .collect::<Vec<_>>(),
    );
    let inner_hash = hasher.finalize();

    let mut hasher = sha3::Keccak256::new();
    hasher.update(HMAC_O.iter().copied().chain(inner_hash).collect::<Vec<_>>());

    hasher.finalize().into()
}

// TODO: link whitepaper equation
fn hash_to_field2(message: impl Iterator<Item = u8> + Clone) -> (U256, U256) {
    (
        hash_to_field(
            [0].into_iter()
                .chain(message.clone())
                .collect::<Vec<_>>()
                .as_ref(),
        ),
        hash_to_field([1].into_iter().chain(message).collect::<Vec<_>>().as_ref()),
    )
}

// TODO: link whitepaper equation
fn hash_to_field(message: &[u8]) -> U256 {
    (U256::from(&hmac_keccak(message)) % PRIME_R_MINUS_ONE) + 1
}

pub const G1_SIZE: usize = 64;
pub const G2_SIZE: usize = 2 * G1_SIZE;
pub const COMMITMENT_HASH_SIZE: usize = 32;

pub struct G1Affine<FromOrder: ByteOrder, P: Pairing>(PhantomData<FromOrder>, P::G1Affine);
pub type G1AffineBE<P> = G1Affine<BigEndian, P>;
pub type G1AffineLE<P> = G1Affine<LittleEndian, P>;

impl<P: Pairing> TryFrom<[u8; G1_SIZE]> for G1AffineBE<P> {
    type Error = Error;
    fn try_from(mut value: [u8; G1_SIZE]) -> Result<Self, Self::Error> {
        value[0..32].reverse();
        value[32..64].reverse();
        let G1Affine(_, point) = G1Affine::<LittleEndian, P>::try_from(value)?;
        Ok(G1Affine(PhantomData, point))
    }
}

impl<P: Pairing> TryFrom<[u8; G1_SIZE]> for G1AffineLE<P> {
    type Error = Error;
    fn try_from(value: [u8; G1_SIZE]) -> Result<Self, Self::Error> {
        let point = <P::G1Affine as CanonicalDeserialize>::deserialize_uncompressed(value.as_ref())
            .map_err(|_| Error::InvalidPoint)?;
        Ok(G1Affine(PhantomData, point))
    }
}

pub struct G2Affine<FromOrder, P: Pairing>(PhantomData<FromOrder>, P::G2Affine);
pub type G2AffineBE<P> = G2Affine<BigEndian, P>;
pub type G2AffineLE<P> = G2Affine<LittleEndian, P>;

impl<P: Pairing> TryFrom<[u8; G2_SIZE]> for G2AffineBE<P> {
    type Error = Error;
    fn try_from(mut value: [u8; G2_SIZE]) -> Result<Self, Self::Error> {
        value[0..64].reverse();
        value[64..128].reverse();
        let G2Affine(_, point) = G2Affine::<LittleEndian, P>::try_from(value)?;
        Ok(G2Affine(PhantomData, point))
    }
}

impl<P: Pairing> TryFrom<[u8; G2_SIZE]> for G2AffineLE<P> {
    type Error = Error;
    fn try_from(value: [u8; G2_SIZE]) -> Result<Self, Self::Error> {
        let point = <P::G2Affine as CanonicalDeserialize>::deserialize_uncompressed(value.as_ref())
            .map_err(|_| Error::InvalidPoint)?;
        Ok(G2Affine(PhantomData, point))
    }
}

pub struct CommitmentHash<FromOrder>(PhantomData<FromOrder>, U256);
pub type CommitmentHashBE = CommitmentHash<BigEndian>;
pub type CommitmentHashLE = CommitmentHash<LittleEndian>;

impl From<[u8; COMMITMENT_HASH_SIZE]> for CommitmentHashBE {
    fn from(value: [u8; COMMITMENT_HASH_SIZE]) -> Self {
        CommitmentHash(PhantomData, U256::from_big_endian(&value))
    }
}

impl From<[u8; COMMITMENT_HASH_SIZE]> for CommitmentHashLE {
    fn from(value: [u8; COMMITMENT_HASH_SIZE]) -> Self {
        CommitmentHash(PhantomData, U256::from_little_endian(&value))
    }
}

#[derive(Debug)]
pub struct ZKP<FromOrder, P: Pairing> {
    pub proof: Proof<P>,
    pub commitment_hash: U256,
    pub proof_commitment: P::G1Affine,
    pub _marker: PhantomData<FromOrder>,
}

// G1 + G2 + G1 + U256 + G1
pub const EXPECTED_PROOF_SIZE: usize = G1_SIZE + G2_SIZE + G1_SIZE + COMMITMENT_HASH_SIZE + G1_SIZE;

// [a ... b ... c ... commitment_hash ... proof_commitment]
pub type RawZKP = [u8; EXPECTED_PROOF_SIZE];

impl<FromOrder: ByteOrder, P: Pairing> TryFrom<&[u8]> for ZKP<FromOrder, P>
where
    G1Affine<FromOrder, P>: TryFrom<[u8; G1_SIZE], Error = Error>,
    G2Affine<FromOrder, P>: TryFrom<[u8; G2_SIZE], Error = Error>,
    CommitmentHash<FromOrder>: From<[u8; COMMITMENT_HASH_SIZE]>,
{
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value = RawZKP::try_from(value).map_err(|_| Error::InvalidRawProof)?;
        // sadly, no const sub-slicing...
        let G1Affine(_, a) =
            G1Affine::<FromOrder, P>::try_from(value[0..G1_SIZE].try_into().expect("impossible"))?;
        let G2Affine(_, b) = G2Affine::<FromOrder, P>::try_from(
            value[G1_SIZE..G1_SIZE + G2_SIZE]
                .try_into()
                .expect("impossible"),
        )?;
        let G1Affine(_, c) = G1Affine::<FromOrder, P>::try_from(
            value[G1_SIZE + G2_SIZE..G1_SIZE + G2_SIZE + G1_SIZE]
                .try_into()
                .expect("impossible"),
        )?;
        let CommitmentHash(_, commitment_hash) = CommitmentHash::<FromOrder>::from(
            value[G1_SIZE + G2_SIZE + G1_SIZE..G1_SIZE + G2_SIZE + G1_SIZE + COMMITMENT_HASH_SIZE]
                .try_into()
                .expect("impossible"),
        );
        let G1Affine(_, proof_commitment) = G1Affine::<FromOrder, P>::try_from(
            value[G1_SIZE + G2_SIZE + G1_SIZE + COMMITMENT_HASH_SIZE
                ..G1_SIZE + G2_SIZE + G1_SIZE + COMMITMENT_HASH_SIZE + G1_SIZE]
                .try_into()
                .expect("impossible"),
        )?;
        Ok(Self {
            proof: Proof { a, b, c },
            commitment_hash,
            proof_commitment,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EthAbiDecoding,
    InvalidPoint,
    InvalidProof(SynthesisError),
    InvalidVerifyingKey,
    InvalidRawProof,
}

pub fn verify_zkp(
    _vk: VerifyingKey<ark_bn254::Bn254>,
    _trusted_validators_hash: U256,
    _untrusted_validators_hash: U256,
    _message: &[u8],
    _zkp: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    Err(Error::InvalidProof(SynthesisError::MalformedVerifyingKey))
}

pub fn verify_zkp_v2(
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    zkp: impl Into<Vec<u8>>,
) -> Result<(), Error> {
    verify_generic_zkp_v2::<ark_bn254::Bn254>(
        universal_vk(),
        trusted_validators_hash,
        untrusted_validators_hash,
        message,
        ZKP::try_from(zkp.into().as_ref())?,
    )
}

fn verify_generic_zkp_v2<P: Pairing>(
    vk: VerifyingKey<P>,
    trusted_validators_hash: U256,
    untrusted_validators_hash: U256,
    message: &[u8],
    zkp: ZKP<BigEndian, P>,
) -> Result<(), Error> {
    let mut buffer = [0u8; 32];
    let mut decode_scalar = move |x: U256| {
        x.to_little_endian(&mut buffer);
        // NOTE: This would silently fail if the input do not fit the scalar
        // field, which is unlikely to happen unless the parameters have been
        // tampered. The pairing check would obviously fail in this case.
        <P::ScalarField as PrimeField>::from_le_bytes_mod_order(&buffer)
    };

    let (message_x, message_y) = hash_to_field2(message.iter().copied());

    let public_inputs: [P::ScalarField; 5] = [
        decode_scalar(trusted_validators_hash),
        decode_scalar(untrusted_validators_hash),
        decode_scalar(message_x),
        decode_scalar(message_y),
        decode_scalar(zkp.commitment_hash),
    ];

    let mut prepared_inputs = vk
        .gamma_abc_g1
        .first()
        .ok_or(Error::InvalidVerifyingKey)?
        .into_group();
    // Gnark specific, we need to aggregate the proof commitment
    // See https://github.com/ConsenSys/gnark/issues/652
    prepared_inputs.add_assign(zkp.proof_commitment);
    for (i, b) in public_inputs
        .into_iter()
        .zip(vk.gamma_abc_g1.iter().skip(1))
    {
        prepared_inputs.add_assign(&b.mul_bigint(i.into_bigint()));
    }

    Groth16::<P>::verify_proof_with_prepared_inputs(&vk.into(), &zkp.proof, &prepared_inputs)
        .map_err(Error::InvalidProof)
        .and_then(|v| {
            if v {
                Ok(())
            } else {
                Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
            }
        })
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use ethabi::ethereum_types::U256;

    use super::*;

    #[test]
    fn test_universal_valid() {
        assert_eq!(
            verify_zkp_v2(
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F3").unwrap(),
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F3").unwrap(),
                &hex!("650802113E0200000000000022480A207022627E60ED78120D2FE8DC7ACDB58A2321B0304F8912D2DFB86CE038E23CA812240801122041B8793236EE0980E2EAF1A2FAD268C4A3D8979A0C432F06E284EEC5E74DD69C320E756E696F6E2D6465766E65742D31"),
                hex!("1482C010E882F0FE7CD37AD477C6568E4B99FD26B04774F1EB51D6DC67FBB6520A7904F5B3BEBF08AD768C950DE6C79CA343FE5B821059B1614343C51AFCB23F22D28D595C6C2CFBC85516D5D3A59492135607BD21273FC2BA15484AF9B345EA19523F294E6D1A55A54F1B20B80206E62607929EA7FC84E788530E11A41BC7C801D9397F61C56245253F71AC972B854F4FA3C4731EBA98562298B62EEB503FA61114DBB928F6CDD347D85466EF1D53934BAAC7EA547E1239099FCC25B1206BF718C2DD48CA81FCF13D2C9F017BD163876064FE1E23E5A98B58CBE3D2C410E376100EEAA247A439843031C9AC1BDEC04255E360787512CF1B91E9D4E9551E653007D7B739A721543A27414FBB924711EE29904FA19CD907C50A49D0BBF3C6ABE503C2912FBE6004F74A0B803D33654960AA6CF9CE5E2C3B03F8ADD8D843D9F5CC22CB2F56833579ADB3EBA5C408EC5DBEC694BFA6FC30E4A89AF4EE761F111CAB")
            ),
            Ok(())
        );
    }

    #[test]
    fn test_universal_invalid_vote() {
        assert_eq!(
            verify_zkp_v2(
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F3").unwrap(),
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F3").unwrap(),
                &hex!("650802113E0200000000000022480A207022627E60ED78120D2FE8DC7ACDB58A2321B0304F8912D2DFB86CE038E23CA812240801122041B8793236EE0980E2EAF1A2FAD268C4A3D8979A0C432F06E284EEC5E74DD69C320E756E696F6E2D6465766E65742D32"),
                hex!("1482C010E882F0FE7CD37AD477C6568E4B99FD26B04774F1EB51D6DC67FBB6520A7904F5B3BEBF08AD768C950DE6C79CA343FE5B821059B1614343C51AFCB23F22D28D595C6C2CFBC85516D5D3A59492135607BD21273FC2BA15484AF9B345EA19523F294E6D1A55A54F1B20B80206E62607929EA7FC84E788530E11A41BC7C801D9397F61C56245253F71AC972B854F4FA3C4731EBA98562298B62EEB503FA61114DBB928F6CDD347D85466EF1D53934BAAC7EA547E1239099FCC25B1206BF718C2DD48CA81FCF13D2C9F017BD163876064FE1E23E5A98B58CBE3D2C410E376100EEAA247A439843031C9AC1BDEC04255E360787512CF1B91E9D4E9551E653007D7B739A721543A27414FBB924711EE29904FA19CD907C50A49D0BBF3C6ABE503C2912FBE6004F74A0B803D33654960AA6CF9CE5E2C3B03F8ADD8D843D9F5CC22CB2F56833579ADB3EBA5C408EC5DBEC694BFA6FC30E4A89AF4EE761F111CAB")
            ),
            Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
        );
    }

    #[test]
    fn test_universal_invalid_zkp() {
        assert_eq!(
            verify_zkp_v2(
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F3").unwrap(),
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F3").unwrap(),
                &hex!("650802113E0200000000000022480A207022627E60ED78120D2FE8DC7ACDB58A2321B0304F8912D2DFB86CE038E23CA812240801122041B8793236EE0980E2EAF1A2FAD268C4A3D8979A0C432F06E284EEC5E74DD69C320E756E696F6E2D6465766E65742D31"),
                hex!("0D32AEA09C8BDE6B6680E477080F326074FD6DBAB7944A794105D086A8C15072087613FDCA4E04EC0630B71915155A3581DABCB5C989F20E191D1519C58929231A0BAECD7194E1B4FEF8AA0432583912FEA40B24CBFA0D55C1AE04D1F60F32FC0F330468B82DA59AC8AA9F7DDCD48D8DA4301EA0794CD7B2F1AE9FE8622510BF296A2D238FCBF2A88B38BB41762FD698FE80FBE029904DC4A900D36B813501EB26A02CA5C57DB0A7345F0E20583B193B9055BC0F69980741D819A59FCA9FED060F1F04AE76EF5F67DA13DE20D6D8722259835BDB06F53764E5AE0FFF07796DB417FF67BDF4E081E8F22424AA896BFC952631BDDBE671E382B5E55CD0598B0F14033C552D3F946C983B72ADB3DB09AC0A4612C52702CDE50AFA9CA6762E684CB503BA11C6A436412C26AA12DB969DE631DAABECB6172BC4D51081CCA176737CB913D74E0F8D2C91817D7E5EB8E81C409D938C2012C63FA853515AB634F321B88F")
            ),
            Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
        );
    }

    #[test]
    fn test_universal_invalid_trusted_validators_hash() {
        assert_eq!(
            verify_zkp_v2(
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F4").unwrap(),
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F3").unwrap(),
                &hex!("650802113E0200000000000022480A207022627E60ED78120D2FE8DC7ACDB58A2321B0304F8912D2DFB86CE038E23CA812240801122041B8793236EE0980E2EAF1A2FAD268C4A3D8979A0C432F06E284EEC5E74DD69C320E756E696F6E2D6465766E65742D31"),
                hex!("1482C010E882F0FE7CD37AD477C6568E4B99FD26B04774F1EB51D6DC67FBB6520A7904F5B3BEBF08AD768C950DE6C79CA343FE5B821059B1614343C51AFCB23F22D28D595C6C2CFBC85516D5D3A59492135607BD21273FC2BA15484AF9B345EA19523F294E6D1A55A54F1B20B80206E62607929EA7FC84E788530E11A41BC7C801D9397F61C56245253F71AC972B854F4FA3C4731EBA98562298B62EEB503FA61114DBB928F6CDD347D85466EF1D53934BAAC7EA547E1239099FCC25B1206BF718C2DD48CA81FCF13D2C9F017BD163876064FE1E23E5A98B58CBE3D2C410E376100EEAA247A439843031C9AC1BDEC04255E360787512CF1B91E9D4E9551E653007D7B739A721543A27414FBB924711EE29904FA19CD907C50A49D0BBF3C6ABE503C2912FBE6004F74A0B803D33654960AA6CF9CE5E2C3B03F8ADD8D843D9F5CC22CB2F56833579ADB3EBA5C408EC5DBEC694BFA6FC30E4A89AF4EE761F111CAB")
            ),
            Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
        );
    }

    #[test]
    fn test_universal_invalid_untrusted_validators_hash() {
        assert_eq!(
            verify_zkp_v2(
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F3").unwrap(),
                U256::from_str("0x2EDEEA99530287CF06CEAF8AAF877B5E03AE8ED5D7359F232C9E8AAF3B0205F4").unwrap(),
                &hex!("650802113E0200000000000022480A207022627E60ED78120D2FE8DC7ACDB58A2321B0304F8912D2DFB86CE038E23CA812240801122041B8793236EE0980E2EAF1A2FAD268C4A3D8979A0C432F06E284EEC5E74DD69C320E756E696F6E2D6465766E65742D31"),
                hex!("1482C010E882F0FE7CD37AD477C6568E4B99FD26B04774F1EB51D6DC67FBB6520A7904F5B3BEBF08AD768C950DE6C79CA343FE5B821059B1614343C51AFCB23F22D28D595C6C2CFBC85516D5D3A59492135607BD21273FC2BA15484AF9B345EA19523F294E6D1A55A54F1B20B80206E62607929EA7FC84E788530E11A41BC7C801D9397F61C56245253F71AC972B854F4FA3C4731EBA98562298B62EEB503FA61114DBB928F6CDD347D85466EF1D53934BAAC7EA547E1239099FCC25B1206BF718C2DD48CA81FCF13D2C9F017BD163876064FE1E23E5A98B58CBE3D2C410E376100EEAA247A439843031C9AC1BDEC04255E360787512CF1B91E9D4E9551E653007D7B739A721543A27414FBB924711EE29904FA19CD907C50A49D0BBF3C6ABE503C2912FBE6004F74A0B803D33654960AA6CF9CE5E2C3B03F8ADD8D843D9F5CC22CB2F56833579ADB3EBA5C408EC5DBEC694BFA6FC30E4A89AF4EE761F111CAB")
            ),
            Err(Error::InvalidProof(SynthesisError::UnexpectedIdentity))
        );
    }
}
