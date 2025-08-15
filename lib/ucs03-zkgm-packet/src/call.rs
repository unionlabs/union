use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::INSTR_VERSION_0;
use unionlabs_primitives::Bytes;

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Call {
    V0(CallV0),
}

impl Call {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => CallV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid call version: {invalid}"))?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallV0 {
    pub sender: Bytes,
    pub eureka: bool,
    pub contract_address: Bytes,
    pub contract_calldata: Bytes,
}

impl CallV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::Call {
            sender,
            eureka,
            contract_address,
            contract_calldata,
        } = ucs03_zkgm::com::Call::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            sender: sender.into(),
            eureka,
            contract_address: contract_address.into(),
            contract_calldata: contract_calldata.into(),
        })
    }
}
