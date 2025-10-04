use ibc_union_spec::ClientId;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use super::{Serializable, TypedAccount2};

pub struct LatestClientId(pub ClientId);

pub struct LatestClientId2<'a> {
    pub client_id: ClientId,
    pub account_info: &'a AccountInfo,
}

impl<'a> TypedAccount2<'a> for LatestClientId2<'a> {
    type Data = ClientId;

    fn new(data: Self::Data, account: &'a AccountInfo) -> Self {
        Self {
            client_id: data,
            account_info: account,
        }
    }

    fn data(&self) -> &Self::Data {
        &self.client_id
    }

    fn account_info(&self) -> &'a AccountInfo {
        self.account_info
    }
}

impl Serializable for ClientId {
    fn serialized_size(&self) -> usize {
        4
    }

    fn serialize_into(&self, data: &mut [u8]) {
        data.copy_from_slice(self.raw().to_le_bytes().as_slice())
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        let client_id = u32::from_le_bytes(
            data[0..4]
                .try_into()
                .map_err(|_| ProgramError::InvalidArgument)?,
        );

        Ok(ClientId::from_raw(client_id).ok_or(ProgramError::InvalidArgument)?)
    }
}

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
    fn serialized_size(&self) -> usize {
        4
    }

    fn serialize_into(&self, data: &mut [u8]) {
        data.copy_from_slice(self.0.raw().to_le_bytes().as_slice())
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from(data)
    }
}
