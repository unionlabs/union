use ibc_union_spec::{ChannelId, ClientId, Connection, ConnectionId};
use pinocchio::program_error::ProgramError;
use unionlabs_primitives::Bytes;

use crate::{helper::peel_u8, Serializable};

macro_rules! id {
    ($Id:ident) => {
        impl<'a> Serializable<'a> for $Id {
            fn serialized_size(&self) -> usize {
                4
            }

            fn serialize_into(&self, data: &mut [u8]) {
                data.copy_from_slice(self.raw().to_le_bytes().as_slice())
            }

            fn deserialize(data: &[u8]) -> Option<Self> {
                let client_id = u32::from_le_bytes(
                    data[0..4]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidArgument)?,
                );

                $Id::from_raw(client_id).ok_or(ProgramError::InvalidArgument)
            }
        }
    };
}

id!(ClientId);
id!(ConnectionId);
id!(ChannelId);

impl<'a> Serializable<'a> for Connection {
    fn serialized_size(&self) -> usize {
        13
    }

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

    fn deserialize(mut data: &[u8]) -> Option<Self> {
        let data = &mut data;

        Ok(Self {
            state: peel_u8(data)
                .ok_or(ProgramError::InvalidArgument)?
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
}
