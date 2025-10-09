#[cfg(test)]
mod tests;

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
) -> Result<(Option<VoteAttestation>, Valset), ExtraDataDecodeError> {
    if data.len() <= EXTRA_DATA_MIN_LEN {
        return Err(ExtraDataDecodeError::InvalidExtraDataLen);
    }

    let data = &data[EXTRA_VANITY_LEN..(data.len() - EXTRA_SEAL_LEN)];

    let num = data[0];
    let vals = data[1..]
        .as_chunks::<VAL_ENTRY_LEN>()
        .0
        .iter()
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

    let vote_attestation_rlp_data =
        &data[(VAL_COUNT_SIZE + (VAL_ENTRY_LEN * num as usize) + NEXT_TURN_LENGTH_SIZE)..];

    let va = if vote_attestation_rlp_data.is_empty() {
        None
    } else {
        Some(rlp::decode::<VoteAttestation>(vote_attestation_rlp_data)?)
    };

    Ok((va, Valset::new(vals)))
}

pub fn parse_header_extra_data(
    data: &[u8],
) -> Result<Option<VoteAttestation>, ExtraDataDecodeError> {
    if data.len() <= EXTRA_DATA_MIN_LEN {
        return Err(ExtraDataDecodeError::InvalidExtraDataLen);
    }

    let data = &data[EXTRA_VANITY_LEN..(data.len() - EXTRA_SEAL_LEN)];

    let va = if data.is_empty() {
        None
    } else {
        Some(rlp::decode::<VoteAttestation>(data)?)
    };

    Ok(va)
}

pub fn get_vote_attestation_from_header_extra_data(
    header: &ParliaHeader,
) -> Result<Option<VoteAttestation>, ExtraDataDecodeError> {
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
    #[error("not enough headers provided in update chain")]
    NotEnoughHeaders,
    #[error("invalid chain")]
    InvalidChain,
    #[error("invalid attestation")]
    InvalidAttestation,
    #[error("no attestation present in extra data")]
    NoAttestation,
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

/// Given a chain of headers `C`, with source `S = C[-3]`, target `T = C[-2]`, and attestation `A = C[-1]`:
/// 1. ensure that `A` is not expired
/// 2. verify that `C` is a valid chain (i.e. `C[0] ∈ C[1] ∈ ... ∈ C[-2] ∈ C[-1]`)
/// 3. ensure that `A` contains the vote data for `S` and `T`
/// 4. validate the signature contained in `A` with the valset that signed it
/// 5. if `C[0]` is an epoch change block, return the epoch change block number and the new valset
#[expect(clippy::type_complexity, reason = "clippy is afraid of tuples")]
pub fn verify_header<C: VerificationContext>(
    chain: &[ParliaHeader],
    unbond_period: Duration,
    trusted_valset_epoch_block_number: u64,
    ctx: C,
) -> Result<(&ParliaHeader, Option<(u64, Valset)>), Error<C::Error>> {
    let [rest @ .., source, target, attestation] = chain else {
        return Err(Error::NotEnoughHeaders);
    };

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
    let oldest_parent_header = [target, source]
        .into_iter()
        .chain(rest.iter().rev())
        .try_fold(attestation, |child, parent| {
            if child.number == parent.number + U256::ONE && child.parent_hash == parent.hash() {
                Ok(parent)
            } else {
                Err(Error::InvalidChain)
            }
        })?;

    // 3.
    let vote_attestation =
        get_vote_attestation_from_header_extra_data(attestation)?.ok_or(Error::NoAttestation)?;

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

    // 4.
    ctx.verify(
        signing_valset.map(|x| &x.1.1),
        vote_attestation.data.hash().get(),
        vote_attestation.agg_signature,
    )
    .map_err(Error::ContextError)?;

    // 5.
    let maybe_epoch_rotation_data = if is_epoch_rotation_header(oldest_parent_header) {
        let (_, new_valset) = parse_epoch_rotation_header_extra_data(&source.extra_data)?;
        Some((
            oldest_parent_header
                .number
                .try_into()
                .map_err(|()| Error::BlockNumberTooLarge)?,
            new_valset,
        ))
    } else {
        None
    };

    Ok((oldest_parent_header, maybe_epoch_rotation_data))
}
