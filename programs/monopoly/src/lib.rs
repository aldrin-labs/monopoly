use solana_program::{
    declare_id,
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};

pub mod account;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;
pub mod account_ext;
pub mod board;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_instruction() {
        use solana_program::{
            account_info::AccountInfo,
            program_error::ProgramError,
            pubkey::Pubkey,
        };
        use std::cell::RefCell;
        use crate::instruction::MonopolyInstruction;
        use crate::processor;

        // Create test accounts
        let program_id = Pubkey::new_unique();
        let key = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![0; 100];
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &RefCell::new(data),
            &program_id,
            false,
            0,
        );
        let accounts = vec![account];

        // Test valid instruction
        let instruction = MonopolyInstruction::InitGame {
            player_names: vec!["Alice".to_string(), "Bob".to_string()]
        };
        let mut instruction_data = vec![];
        instruction.pack_into_slice(&mut instruction_data);

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert!(result.is_ok());

        // Test invalid instruction data
        let invalid_data = vec![255; 1]; // Invalid instruction data
        let result = process_instruction(&program_id, &accounts, &invalid_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ProgramError::InvalidInstructionData
        );

        // Test processor delegation
        let instruction = MonopolyInstruction::InitGame {
            player_names: vec!["Alice".to_string(), "Bob".to_string()]
        };
        let mut instruction_data = vec![];
        instruction.pack_into_slice(&mut instruction_data);
        
        // Mock processor to verify delegation
        let result = processor::process_instruction(&program_id, &accounts, instruction);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
pub mod test;

use instruction::MonopolyInstruction;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


// Program entrypoint
entrypoint!(process_instruction);

/// Main processing function for the program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Monopoly program entrypoint");
    
    let instruction = MonopolyInstruction::unpack_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
        
    processor::process_instruction(program_id, accounts, instruction)
}
