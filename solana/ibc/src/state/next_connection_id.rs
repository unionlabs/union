use ibc_union_spec::ConnectionId;
use pinocchio::program_error::ProgramError;

use super::{Serializable, StaticInit};

pub struct NextConnectionId(pub ConnectionId);

impl NextConnectionId {
    pub const fn seed<'a>() -> &'a [&'a [u8]] {
        &[b"next_connection_id"]
    }

    pub fn increment(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }
}

impl StaticInit for NextConnectionId {
    fn static_init() -> Self {
        NextConnectionId(ConnectionId!(1))
    }
}

impl TryFrom<&[u8]> for NextConnectionId {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let client_id = u32::from_le_bytes(
            data[0..4]
                .try_into()
                .map_err(|_| ProgramError::InvalidArgument)?,
        );

        Ok(Self(
            ConnectionId::from_raw(client_id).ok_or(ProgramError::InvalidArgument)?,
        ))
    }
}

impl Serializable for NextConnectionId {
    fn serialize_into(&self, data: &mut [u8]) {
        data.copy_from_slice(self.0.raw().to_le_bytes().as_slice())
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from(data)
    }

    fn serialized_size(&self) -> usize {
        4
    }
}
