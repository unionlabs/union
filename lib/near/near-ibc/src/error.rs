use ibc_vm_rs::IbcError;

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum Error {
    #[error(transparent)]
    Ibc(#[from] IbcError),
}
