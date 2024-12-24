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
