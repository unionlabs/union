use ibc_union_spec::ClientId;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{create_program_address, find_program_address, Pubkey},
    ProgramResult,
};

use crate::{
    helper::{peel_bytes, peel_str},
    next_client_id::NextClientId,
    Instruction, TypedAccount,
};

pub struct CreateClient<'a> {
    pub accounts: &'a CreateClientAccounts<'a>,
    pub instruction_data: &'a CreateClientData,
}

impl<'a> Instruction<'a> for CreateClient<'a> {
    const DISCRIMINATOR: u8 = 0;

    fn process(self) -> ProgramResult {
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

    fn from_input(accounts: &'a [AccountInfo], data: &'a [u8]) -> Result<Self, ProgramError> {
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
#[repr(C)]
struct Idx<const I: usize, const L: usize> {
    idx: [u8; I],
    len: [u8; L],
}

impl<const I: usize, const L: usize> Idx<I, L>
where
    Self: IdxLen + IdxIdx,
{
    fn encoded_len(&self) -> usize {
        I + L + (self.data_len().into() as usize)
    }

    fn slice<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[(self.data_idx().into() as usize)
            ..(self.data_idx().into() as usize) + (self.data_len().into() as usize)]
    }
}

pub trait IdxLen {
    type Len: Into<u32>;

    fn data_len(&self) -> Self::Len;
}

pub trait IdxIdx {
    type Idx: Into<u32>;

    fn data_idx(&self) -> Self::Idx;
}

macro_rules! idx_impl {
    ($Ty:ty) => {
        impl<const I: usize> IdxLen for Idx<I, { (<$Ty>::BITS / 8) as usize }> {
            type Len = $Ty;

            fn data_len(&self) -> Self::Len {
                <$Ty>::from_le_bytes(self.len)
            }
        }

        impl<const I: usize> IdxIdx for Idx<I, { (<$Ty>::BITS / 8) as usize }> {
            type Idx = $Ty;

            fn data_idx(&self) -> Self::Idx {
                <$Ty>::from_le_bytes(self.len)
            }
        }
    };
}

idx_impl!(u8);
idx_impl!(u16);
idx_impl!(u32);

#[derive(Debug)]
#[repr(C)]
pub struct CreateClientData {
    client_type: Idx<4, 1>,
    client_state_bytes: Idx<4, 4>,
    consensus_state_bytes: Idx<4, 4>,
    relayer: Pubkey,
    data: [u8],
}

impl CreateClientData {
    fn encoded_len(&self) -> usize {
        self.client_type.encoded_len()
            + self.client_state_bytes.encoded_len()
            + self.consensus_state_bytes.encoded_len()
            + self.relayer.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for CreateClientData<'a> {
    type Error = ProgramError;

    fn try_from(mut data: &'a [u8]) -> Result<Self, Self::Error> {
        let data = &mut data;
        Ok(Self {
            client_type: peel_str(data).ok_or(ProgramError::InvalidArgument)?,
            client_state_bytes: peel_bytes(data).ok_or(ProgramError::InvalidArgument)?,
            consensus_state_bytes: peel_bytes(data).ok_or(ProgramError::InvalidArgument)?,
            relayer: peel_str(data).ok_or(ProgramError::InvalidArgument)?,
        })
    }
}

impl From<CreateClientData> for Vec<u8> {
    fn from(val: CreateClientData) -> Self {
        let mut buf = Vec::with_capacity(val.encoded_len());

        buf.extend_from_slice(&(val.client_type.len() as u32).to_le_bytes());
        buf.extend_from_slice(val.client_type.as_bytes());

        buf.extend_from_slice(&(val.client_state_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(val.client_state_bytes);

        buf.extend_from_slice(&(val.consensus_state_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(val.consensus_state_bytes);

        buf.extend_from_slice(&(val.relayer.len() as u32).to_le_bytes());
        buf.extend_from_slice(val.relayer.as_bytes());

        buf
    }
}
