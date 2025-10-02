use ibc_union_spec::ClientId;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
    ProgramResult,
};
use unionlabs_primitives::Bytes;

use crate::helper::{parse_bytes, parse_client_id, parse_string, write_client_id};

pub struct CreateClient<'a> {
    pub accounts: CreateClientAccounts<'a>,
    pub instruction_data: CreateClientData,
}

impl<'a> CreateClient<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        let mut current_client_id_data = self.accounts.client_id.try_borrow_mut_data()?;
        let current_client_id = parse_client_id(&current_client_id_data)?
            .checked_add(1)
            .unwrap();

        self.accounts.validate_accounts(current_client_id)?;

        let _client_state_bytes = self.accounts.client_state.try_borrow_data()?;
        let _consensus_state_bytes = self.accounts.consensus_state.try_borrow_data()?;

        // TODO(aeryz): lightclient.create_client

        // TODO(aeryz): i don't think the client and consensus state commitments are needed
        // for solana. So im thinking no need to bother with more accounts.

        write_client_id(current_client_id_data.as_mut(), current_client_id);

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

        let (account, _) = find_program_address(
            &[b"consensus_state", &client_id.raw().to_le_bytes()],
            &crate::ID,
        );
        if self.consensus_state.key() != &account {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for CreateClientAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [client_id, client_state, consensus_state, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let (account, _) = find_program_address(&[b"client_id"], &crate::ID);
        if client_id.key() != &account {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            client_id,
            client_state,
            consensus_state,
        })
    }
}

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

        let (_, relayer) = parse_string(data)?;

        Ok(Self {
            client_type,
            client_state_bytes,
            consensus_state_bytes,
            relayer,
        })
    }
}
