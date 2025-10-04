use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

pub mod latest_client_id;
pub mod latest_connection_id;
pub mod spec;

pub trait TypedAccount2<'a>: Sized {
    type Data: Serializable;

    fn new(data: Self::Data, account: &'a AccountInfo) -> Self;

    fn data(&self) -> &Self::Data;

    fn account_info(&self) -> &'a AccountInfo;

    fn init_if_needed(
        data: Self::Data,
        account: &'a AccountInfo,
        payer: &AccountInfo,
        seeds: &[&[u8]],
    ) -> Result<Self, ProgramError> {
        let space = data.serialized_size();
        if !account.is_owned_by(&crate::ID) || account.data_len() < space {
            Self::init(data, account, payer, seeds)
        } else {
            Self::load(account)
        }
    }

    fn init(
        data: Self::Data,
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

        Ok(Self::new(data, account))
    }

    fn load(account: &'a AccountInfo) -> Result<Self, ProgramError> {
        let data = Self::Data::deserialize(account.try_borrow_data()?.as_ref())?;

        Ok(Self::new(data, account))
    }

    fn save(&self) -> ProgramResult {
        self.data()
            .serialize_into(self.account_info().try_borrow_mut_data()?.as_mut());

        Ok(())
    }
}

/// Binds an account to a type that is serializable to make io operations easier, and well-typed.
pub struct TypedAccount<'a, T: Serializable> {
    pub data: T,
    pub account: &'a AccountInfo,
}

impl<'a, T: Serializable> TypedAccount<'a, T> {
    pub fn init_if_needed(
        data: T,
        account: &'a AccountInfo,
        payer: &AccountInfo,
        seeds: &[&[u8]],
    ) -> Result<Self, ProgramError> {
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
        let data = T::deserialize(account.try_borrow_data()?.as_ref())?;

        Ok(Self { data, account })
    }

    pub fn save(&self) -> ProgramResult {
        self.data
            .serialize_into(self.account.try_borrow_mut_data()?.as_mut());

        Ok(())
    }
}

impl<'a, T: Serializable> AsRef<T> for TypedAccount<'a, T> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<'a, T: Serializable> AsMut<T> for TypedAccount<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

pub trait Serializable: Sized {
    fn serialized_size(&self) -> usize;

    fn serialize_into(&self, data: &mut [u8]);

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError>;
}
