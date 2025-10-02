use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

pub mod latest_client_id;

/// Binds an account to a type that is serializable to make io operations easier, and well-typed.
pub struct TypedAccount<'a, T: Serializable> {
    pub data: T,
    pub account: &'a AccountInfo,
}

impl<'a, T: Serializable> TypedAccount<'a, T> {
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
    fn serialize_into(&self, data: &mut [u8]);

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError>;
}
