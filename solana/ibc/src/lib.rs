use pinocchio::{
    ProgramResult, account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey,
};

use crate::{create_client::CreateClient, instructions::connection_open_init::ConnectionOpenInit};

pub mod helper;
pub mod instructions;
pub mod state;
pub use instructions::*;
pub use state::*;

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
        Some((ConnectionOpenInit::DISCRIMINATOR, data)) => {
            ConnectionOpenInit::try_from((data, accounts))?.process()
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[cfg(test)]
mod tests {
    use mollusk_svm::Mollusk;
    use solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
    };

    use super::*;
    use crate::create_client::CreateClientData;

    #[test]
    fn test_create_client() {
        let mollusk = Mollusk::new(&ID.into(), "../../result/ibc_union_solana");

        let create_client: Vec<u8> = CreateClientData {
            client_type: "cometbls".into(),
            client_state_bytes: b"helloworld".into(),
            consensus_state_bytes: b"helloworld2".into(),
            relayer: "idunnomate".into(),
        }
        .into();

        let account_addresses = [
            solana_sdk::pubkey::Pubkey::find_program_address(&[b"client_id"], &ID.into()).0,
            solana_sdk::pubkey::Pubkey::find_program_address(
                &[b"client_state", &1u32.to_le_bytes()],
                &ID.into(),
            )
            .0,
            solana_sdk::pubkey::Pubkey::find_program_address(
                &[b"consensus_state", &1u32.to_le_bytes()],
                &ID.into(),
            )
            .0,
            solana_sdk::pubkey::Pubkey::find_program_address(&[b"bs"], &ID.into()).0,
        ];

        let instruction = Instruction::new_with_bytes(
            ID.into(),                                                  // Your program's ID
            &[0].into_iter().chain(create_client).collect::<Vec<u8>>(), // Instruction data (discriminator + parameters)
            account_addresses
                .iter()
                .map(|a| AccountMeta::new(a.to_owned().into(), true))
                .collect(),
        );

        let lamports = mollusk.sysvars.rent.minimum_balance(instruction.data.len());

        let client_id_account = Account {
            lamports,
            data: 1u32.to_le_bytes().into(),
            owner: ID.into(),
            executable: false,
            rent_epoch: 0,
        };

        panic!(
            "res: {:?}",
            mollusk.process_instruction(
                &instruction,
                [(
                    Into::<solana_sdk::pubkey::Pubkey>::into(account_addresses[0]),
                    client_id_account
                )]
                .into_iter()
                .chain(
                    account_addresses
                        .iter()
                        .skip(1)
                        .map(|a| (a.to_owned().into(), Account::new(lamports, 10, &ID.into())))
                )
                .collect::<Vec<(solana_sdk::pubkey::Pubkey, Account)>>()
                .as_slice()
            )
        );
    }
}
