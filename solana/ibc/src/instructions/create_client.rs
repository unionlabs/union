use ibc_union_spec::ClientId;
use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{create_program_address, find_program_address},
};
use unionlabs_primitives::Bytes;

use crate::{
    TypedAccount,
    helper::{parse_bytes, parse_string},
    next_client_id::NextClientId,
};

pub struct CreateClient<'a> {
    pub accounts: CreateClientAccounts<'a>,
    pub instruction_data: CreateClientData,
}

impl<'a> CreateClient<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        let mut latest_client_id = TypedAccount::<NextClientId>::init_if_needed(
            self.accounts.client_id,
            self.accounts.payer,
            NextClientId::seed(),
        )?;

        self.accounts
            .validate_accounts(latest_client_id.as_ref().0)?;

        let _client_state_bytes = self.accounts.client_state.try_borrow_data()?;
        let _consensus_state_bytes = self.accounts.consensus_state.try_borrow_data()?;

        // TODO(aeryz): lightclient.create_client

        // TODO(aeryz): i don't think the client and consensus state commitments are needed
        // for solana. So im thinking no need to bother with more accounts.

        latest_client_id.as_mut().increment();
        latest_client_id.save()?;

        Ok(())
    }
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for CreateClient<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = CreateClientAccounts::try_from(accounts)?;
        let instruction_data = CreateClientData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

pub struct CreateClientAccounts<'a> {
    pub payer: &'a AccountInfo,
    pub client_id: &'a AccountInfo,
    pub client_state: &'a AccountInfo,
    pub consensus_state: &'a AccountInfo,
}

impl<'a> CreateClientAccounts<'a> {
    /// An extra endpoint to validate `client_state` and `consensus_state` accounts since
    /// the `client_id` is determined at the runtime.
    pub fn validate_accounts(&self, client_id: ClientId) -> ProgramResult {
        let (account, _) = find_program_address(
            &[b"client_state", &client_id.raw().to_le_bytes()],
            &crate::ID,
        );
        if self.client_state.key() != &account {
            return Err(ProgramError::InvalidAccountData);
        }

        let account = create_program_address(
            &[b"consensus_state", &client_id.raw().to_le_bytes()],
            &crate::ID,
        )?;
        if self.consensus_state.key() != &account {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for CreateClientAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [payer, client_id, client_state, consensus_state, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let account = create_program_address(&[b"client_id"], &crate::ID)?;
        if client_id.key() != &account {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            payer,
            client_id,
            client_state,
            consensus_state,
        })
    }
}

#[derive(Debug)]
pub struct CreateClientData {
    pub client_type: String,
    pub client_state_bytes: Bytes,
    pub consensus_state_bytes: Bytes,
    pub relayer: String,
}

impl TryFrom<&[u8]> for CreateClientData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = 0;
        let (n_read, client_type) = parse_string(data)?;

        cursor += n_read;
        let (n_read, client_state_bytes) = parse_bytes(&data[cursor..]);

        cursor += n_read;
        let (n_read, consensus_state_bytes) = parse_bytes(&data[cursor..]);

        cursor += n_read;
        let (_, relayer) = parse_string(&data[cursor..])?;

        Ok(Self {
            client_type,
            client_state_bytes,
            consensus_state_bytes,
            relayer,
        })
    }
}

impl Into<Vec<u8>> for CreateClientData {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.extend_from_slice(&(self.client_type.len() as u32).to_le_bytes());
        buf.extend_from_slice(self.client_type.as_bytes());

        buf.extend_from_slice(&(self.client_state_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.client_state_bytes);

        buf.extend_from_slice(&(self.consensus_state_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.consensus_state_bytes);

        buf.extend_from_slice(&(self.relayer.len() as u32).to_le_bytes());
        buf.extend_from_slice(self.relayer.as_bytes());

        buf
    }
}
