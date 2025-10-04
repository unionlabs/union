use ibc_union_spec::{ClientId, Connection, ConnectionId, ConnectionState};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
    ProgramResult,
};

use crate::{latest_connection_id::LatestConnectionId, TypedAccount};

pub struct ConnectionOpenInit<'a> {
    pub accounts: ConnectionOpenInitAccounts<'a>,
    pub instruction_data: ConnectionOpenInitData,
}

impl<'a> ConnectionOpenInit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let mut latest_connection_id = TypedAccount::<LatestConnectionId>::init_if_needed(
            LatestConnectionId(ConnectionId!(1)),
            self.accounts.latest_connection_id,
            self.accounts.payer,
            &[b"client_id"],
        )?;

        self.accounts
            .validate_accounts(latest_connection_id.as_ref().0)?;

        TypedAccount::<Connection>::init(
            Connection {
                state: ConnectionState::Init,
                client_id: self.instruction_data.client_id,
                counterparty_client_id: self.instruction_data.counterparty_client_id,
                counterparty_connection_id: None,
            },
            self.accounts.connection,
            self.accounts.payer,
            &[
                b"connection",
                &latest_connection_id.data.0.raw().to_le_bytes(),
            ],
        )?
        .save()?;

        latest_connection_id.as_mut().increment();
        latest_connection_id.save()?;

        Ok(())
    }
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for ConnectionOpenInit<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = ConnectionOpenInitAccounts::try_from(accounts)?;
        let instruction_data = ConnectionOpenInitData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

pub struct ConnectionOpenInitAccounts<'a> {
    pub payer: &'a AccountInfo,
    pub latest_connection_id: &'a AccountInfo,
    pub connection: &'a AccountInfo,
}

impl<'a> ConnectionOpenInitAccounts<'a> {
    /// An extra endpoint to validate `connection` accounts since
    /// the `connection_id` is determined at the runtime.
    pub fn validate_accounts(&self, connection_id: ConnectionId) -> ProgramResult {
        let (account, _) = find_program_address(
            &[b"connection", &connection_id.raw().to_le_bytes()],
            &crate::ID,
        );
        if self.connection.key() != &account {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for ConnectionOpenInitAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [payer, latest_connection_id, connection, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let (account, _) = find_program_address(&[b"latest_connection_id"], &crate::ID);
        if latest_connection_id.key() != &account {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            payer,
            latest_connection_id,
            connection,
        })
    }
}

pub struct ConnectionOpenInitData {
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

impl TryFrom<&[u8]> for ConnectionOpenInitData {
    type Error = ProgramError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            Err(ProgramError::InvalidInstructionData)
        } else {
            Ok(Self {
                client_id: ClientId::from_raw(u32::from_le_bytes(value[0..4].try_into().unwrap()))
                    .ok_or(ProgramError::InvalidArgument)?,
                counterparty_client_id: ClientId::from_raw(u32::from_le_bytes(
                    value[4..8].try_into().unwrap(),
                ))
                .ok_or(ProgramError::InvalidArgument)?,
            })
        }
    }
}

impl Into<Vec<u8>> for ConnectionOpenInitData {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.extend_from_slice(&self.client_id.raw().to_le_bytes());
        buf.extend_from_slice(&self.counterparty_client_id.raw().to_le_bytes());

        buf
    }
}
