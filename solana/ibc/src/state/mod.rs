use pinocchio::{
    account_info::{AccountInfo, Ref},
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

pub mod next_client_id;
pub mod next_connection_id;
pub mod spec;

/// Binds an account to a type that is serializable to make io operations easier, and well-typed.
pub struct TypedAccount<'a, T: Serializable<'a>> {
    pub data: Ref<'a, T>,
    pub account: &'a AccountInfo,
}

impl<'a, T: Serializable<'a>> TypedAccount<'a, T> {
    pub fn init_if_needed(
        account: &'a AccountInfo,
        payer: &AccountInfo,
        seeds: &[&[u8]],
    ) -> Result<Self, ProgramError>
    where
        T: StaticInit,
    {
        let data = T::static_init();
        let space = data.serialized_size();
        if !account.is_owned_by(&crate::ID) || account.data_len() < space {
            Self::init(data, account, payer, seeds)
        } else {
            Self::load(account)
        }
    }

    pub fn init(
        data: T,
        account: &'a AccountInfo,
        payer: &AccountInfo,
        seeds: &[&[u8]],
    ) -> Result<Self, ProgramError> {
        let space = data.serialized_size();
        let lamports = Rent::get()?.minimum_balance(space);

        let (seed, bump) = find_program_address(seeds, &crate::ID);

        let bump = [bump];
        let seeds = [Seed::from(&seed), Seed::from(&bump)];
        let signer = [Signer::from(&seeds)];

        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signer)?;

        data.serialize_into(account.try_borrow_mut_data()?.as_mut());

        Ok(Self { data, account })
    }

    pub fn load(account: &'a AccountInfo) -> Result<Self, ProgramError> {
        let data = Ref::try_map(account.try_borrow_data()?, |data| {
            match T::deserialize(data).ok_or(ProgramError::InvalidAccountData) {
                Ok(ok) => Ok(&ok),
                Err(err) => Err(err),
            }
        })
        .map_err(|(e, r)| r)?;

        Ok(Self { data, account })
    }

    pub fn save(&self) -> ProgramResult {
        self.data
            .serialize_into(self.account.try_borrow_mut_data()?.as_mut());

        Ok(())
    }
}

impl<'a, T: Serializable<'a>> AsRef<T> for TypedAccount<'a, T> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<'a, T: Serializable<'a>> AsMut<T> for TypedAccount<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

pub trait StaticInit {
    fn static_init() -> Self;
}

pub trait Serializable<'a>: Sized {
    fn serialized_size(&self) -> usize;

    fn serialize_into(&self, data: &mut [u8]);

    fn deserialize(data: &'a [u8]) -> Option<Self>;
}

pub trait TypedAccountData: Sized {
    fn is_valid(data: &[u8]) -> bool;

    fn from_account_data(data: &[u8]) -> Option<&Self> {
        if Self::is_valid(data) {
            Some(unsafe { &*(data.as_ptr().cast::<Self>()) })
        } else {
            None
        }
    }

    fn from_account_data_mut(data: &mut [u8]) -> Option<&mut Self> {
        if Self::is_valid(data) {
            Some(unsafe { &mut *(data.as_mut_ptr().cast::<Self>()) })
        } else {
            None
        }
    }
}
