use ibc_union_spec::ConnectionId;
use pinocchio::program_error::ProgramError;

use super::Serializable;

pub struct LatestConnectionId(pub ConnectionId);

impl LatestConnectionId {
    pub fn increment(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }
}

impl TryFrom<&[u8]> for LatestConnectionId {
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

impl Serializable for LatestConnectionId {
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
