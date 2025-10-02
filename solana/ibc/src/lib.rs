use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

pub mod helper;
pub mod instructions;
pub mod state;
pub use instructions::*;

use crate::create_client::CreateClient;

// TODO(aeryz): change this to the correct id
pinocchio_pubkey::declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

pinocchio::entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((CreateClient::DISCRIMINATOR, data)) => {
            CreateClient::try_from((data, accounts))?.process()
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[cfg(test)]
mod tests {
    use mollusk_svm::Mollusk;

    // use solana_sdk::{
    //     account::Account,
    //     instruction::{AccountMeta, Instruction},
    //     system_program,
    // };
    use super::*;

    #[test]
    fn test_create_client() {
        let mollusk = Mollusk::new(&ID.into(), "../../result/ibc_union_solana");

        // let instruction = Instruction::new_with_bytes(
        //     ID.into(),                           // Your program's ID
        //     &[0],                                // Instruction data (discriminator + parameters)
        //     vec![AccountMeta::new(payer, true)], // Account metadata
        // );
    }
}
