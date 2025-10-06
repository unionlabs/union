use ibc_union_spec::{ClientId, Connection, ConnectionId};
use pinocchio::program_error::ProgramError;
use unionlabs_primitives::Bytes;

use super::Serializable;

impl Serializable for Connection {
    fn serialize_into(&self, data: &mut [u8]) {
        data[0] = self.state as u8;
        data[1..5].copy_from_slice(&self.client_id.raw().to_le_bytes());
        data[5..9].copy_from_slice(&self.counterparty_client_id.raw().to_le_bytes());

        let counterparty_connection_id = self
            .counterparty_connection_id
            .map_or(0, |x| x.raw())
            .to_le_bytes();
        data[9..13].copy_from_slice(&counterparty_connection_id);
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        Ok(Self {
            state: data[0]
                .try_into()
                .map_err(|_| ProgramError::InvalidArgument)?,
            client_id: ClientId::from_raw(u32::from_le_bytes(data[1..5].try_into().unwrap()))
                .ok_or(ProgramError::InvalidArgument)?,
            counterparty_client_id: ClientId::from_raw(u32::from_le_bytes(
                data[5..9].try_into().unwrap(),
            ))
            .ok_or(ProgramError::InvalidArgument)?,
            counterparty_connection_id: if data[10] == 0 {
                None
            } else {
                Some(
                    ConnectionId::from_raw(u32::from_le_bytes(data[11..15].try_into().unwrap()))
                        .ok_or(ProgramError::InvalidArgument)?,
                )
            },
        })
    }

    fn serialized_size(&self) -> usize {
        13
    }
}

impl Serializable for Bytes {
    fn serialized_size(&self) -> usize {
        self.len()
    }

    fn serialize_into(&self, data: &mut [u8]) {
        data[..self.len()].copy_from_slice(self.as_ref());
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        Ok(Bytes::new(data.to_vec()))
    }
}
