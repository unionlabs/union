use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::INSTR_VERSION_0;
use unionlabs_primitives::U256;

use crate::{root::Root, Result};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Forward {
    V0(ForwardV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardShape {
    V0,
}

impl Forward {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => ForwardV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid forward version: {invalid}"))?,
        }
    }

    pub(crate) fn shape(&self) -> ForwardShape {
        match self {
            Forward::V0(_) => ForwardShape::V0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardV0 {
    path: U256,
    // TODO: Forward v2 to remove this field
    timeout_height: u64,
    timeout_timestamp: u64,
    instruction: Box<Root>,
}

impl ForwardV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::Forward {
            path,
            timeout_height,
            timeout_timestamp,
            instruction,
        } = ucs03_zkgm::com::Forward::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            path: path.into(),
            timeout_height,
            timeout_timestamp,
            instruction: Box::new(Root::from_raw(instruction)?),
        })
    }
}
