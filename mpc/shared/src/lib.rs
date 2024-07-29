pub mod types;

use std::ffi::{c_char, c_int};

pub const CONTRIBUTION_SIZE: usize = 306032532;

#[link(name = "galois")]
extern "C" {
    fn Phase2Contribute(
        phase2_payload_raw: *const c_char,
        phase2_contrib_raw: *mut c_char,
        len: c_int,
    ) -> bool;

    fn Phase2Verify(
        phase2_previous_raw: *const c_char,
        phase2_contrib_raw: *const c_char,
        len: c_int,
    ) -> bool;
}

#[derive(thiserror::Error, Debug, Copy, Clone)]
pub enum Phase2ContributionError {
    #[error("Failed to convert initial payload size, based af is you reached this.")]
    FailedToConvertPayloadSize,
    #[error("Looks like you spent time contributing for no reason because it failed.")]
    FailedToContribute,
}

#[derive(thiserror::Error, Debug, Copy, Clone)]
pub enum Phase2VerificationError {
    #[error("Failed to convert contribution payload size, based af is you reached this.")]
    FailedToConvertPayloadSize,
    #[error("1 BTC = 1 BTC, what are you trying to achieve?")]
    InconsistentPayloadSize,
    #[error("Cheating is great, but not allowed. You may lose your slot if the coordinator chose to :'(.")]
    Phase2VerificationFailed,
}

pub fn phase2_contribute(phase2_payload: &[u8]) -> Result<Vec<u8>, Phase2ContributionError> {
    let payload_len = phase2_payload
        .len()
        .try_into()
        .map_err(|_| Phase2ContributionError::FailedToConvertPayloadSize)?;
    let mut phase2_contrib_raw = vec![0u8; phase2_payload.len()];
    let result = unsafe {
        Phase2Contribute(
            phase2_payload.as_ptr() as *const _,
            phase2_contrib_raw.as_mut_ptr() as *mut _,
            payload_len,
        )
    };
    if result {
        Ok(phase2_contrib_raw)
    } else {
        Err(Phase2ContributionError::FailedToContribute)
    }
}

pub fn phase2_verify(
    phase2_payload: &[u8],
    phase2_contrib: &[u8],
) -> Result<(), Phase2VerificationError> {
    let payload_len = phase2_payload
        .len()
        .try_into()
        .map_err(|_| Phase2VerificationError::FailedToConvertPayloadSize)?;
    if phase2_contrib.len() != phase2_payload.len() {
        Err(Phase2VerificationError::InconsistentPayloadSize)
    } else {
        let result = unsafe {
            Phase2Verify(
                phase2_payload.as_ptr() as *const _,
                phase2_contrib.as_ptr() as *mut _,
                payload_len,
            )
        };
        if result {
            Ok(())
        } else {
            Err(Phase2VerificationError::Phase2VerificationFailed)
        }
    }
}
