use ibc_union_spec::ClientId;
use pinocchio::program_error::ProgramError;

use super::Serializable;

pub struct LatestClientId(pub ClientId);

impl LatestClientId {
    pub fn increment(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }
}

impl TryFrom<&[u8]> for LatestClientId {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let client_id = u32::from_le_bytes(
            data[0..4]
                .try_into()
                .map_err(|_| ProgramError::InvalidArgument)?,
        );

        Ok(Self(
            ClientId::from_raw(client_id).ok_or(ProgramError::InvalidArgument)?,
        ))
    }
}

impl Serializable for LatestClientId {
    fn serialize_into(&self, data: &mut [u8]) {
        data.copy_from_slice(self.0.raw().to_le_bytes().as_slice())
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from(data)
    }
}
