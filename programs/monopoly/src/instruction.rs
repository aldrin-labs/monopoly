use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    program_pack::{Pack, Sealed},
    pubkey::Pubkey,
};
use crate::account::DeckType;

#[derive(Debug)]
pub enum MonopolyInstruction {
    /// Initialize a new game
    /// Accounts expected:
    /// 0. `[writable]` Game account
    /// 1. `[signer]` Rent payer
    InitGame {
        player_names: Vec<String>,
    },

    /// Move player and process tile effects
    /// Accounts expected:
    /// 0. `[writable]` Game account
    /// 1. `[signer]` Current player
    MovePlayer,

    /// Buy property
    /// Accounts expected:
    /// 0. `[writable]` Game account
    /// 1. `[signer]` Player buying property
    BuyProperty {
        property_index: u8,
    },

    /// Build house/hotel
    /// Accounts expected:
    /// 0. `[writable]` Game account
    /// 1. `[signer]` Property owner
    BuildHouse {
        property_index: u8,
    },

    /// Pay rent
    /// Accounts expected:
    /// 0. `[writable]` Game account
    /// 1. `[signer]` Player paying rent
    /// 2. `[]` Property owner
    PayRent {
        property_index: u8,
    },

    /// End current player's turn and move to next player
    /// Accounts expected:
    /// 0. `[writable]` Game account
    /// 1. `[signer]` Current player
    NextTurn,

    /// Check if there is a winner
    /// Accounts expected:
    /// 0. `[writable]` Game account
    CheckWinner,

    /// Draw and process a card from Community Chest or Chance
    /// Accounts expected:
    /// 0. `[writable]` Game account
    /// 1. `[signer]` Current player
    /// 2. `[writable]` Card deck account
    /// 3. `[]` Clock sysvar
    DrawCard {
        deck_type: DeckType,
    },
}

impl Sealed for MonopolyInstruction {}

impl MonopolyInstruction {
    pub fn try_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        Self::unpack_from_slice(input)
    }

    pub fn to_instruction(
        &self,
        program_id: &Pubkey,
        accounts: &[&Pubkey],
    ) -> Instruction {
        let mut account_metas = Vec::new();
        for account in accounts {
            account_metas.push(AccountMeta::new(**account, true));
        }

        let mut data = Vec::with_capacity(Self::LEN);
        self.pack_into_slice(&mut data);

        Instruction {
            program_id: *program_id,
            accounts: account_metas,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_instruction_unpack() {
        // TODO: Implement test
    }
}

impl Pack for MonopolyInstruction {
    const LEN: usize = 1024; // Large enough for instruction data

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut current = 0;
        match self {
            MonopolyInstruction::InitGame { player_names } => {
                dst[current] = 0; // Variant index
                current += 1;
                
                dst[current..current + 4].copy_from_slice(&(player_names.len() as u32).to_le_bytes());
                current += 4;
                
                for name in player_names {
                    let name_bytes = name.as_bytes();
                    dst[current..current + 4].copy_from_slice(&(name_bytes.len() as u32).to_le_bytes());
                    current += 4;
                    dst[current..current + name_bytes.len()].copy_from_slice(name_bytes);
                    current += name_bytes.len();
                }
            }
            MonopolyInstruction::MovePlayer => {
                dst[current] = 1;
            }
            MonopolyInstruction::BuyProperty { property_index } => {
                dst[current] = 2;
                current += 1;
                dst[current] = *property_index;
            }
            MonopolyInstruction::BuildHouse { property_index } => {
                dst[current] = 3;
                current += 1;
                dst[current] = *property_index;
            }
            MonopolyInstruction::PayRent { property_index } => {
                dst[current] = 4;
                current += 1;
                dst[current] = *property_index;
            }
            MonopolyInstruction::NextTurn => {
                dst[current] = 5;
            }
            MonopolyInstruction::CheckWinner => {
                dst[current] = 6;
            }
            MonopolyInstruction::DrawCard { deck_type } => {
                dst[current] = 7;
                current += 1;
                dst[current] = match deck_type {
                    DeckType::CommunityChest => 0,
                    DeckType::Chance => 1,
                };
            }
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let variant = src[0];
        let mut current = 1;

        Ok(match variant {
            0 => {
                let names_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
                current += 4;
                let mut player_names = Vec::with_capacity(names_len);
                
                for _ in 0..names_len {
                    let name_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
                    current += 4;
                    let name = String::from_utf8(src[current..current + name_len].to_vec())
                        .map_err(|_| ProgramError::InvalidInstructionData)?;
                    current += name_len;
                    player_names.push(name);
                }
                
                MonopolyInstruction::InitGame { player_names }
            }
            1 => MonopolyInstruction::MovePlayer,
            2 => MonopolyInstruction::BuyProperty {
                property_index: src[current],
            },
            3 => MonopolyInstruction::BuildHouse {
                property_index: src[current],
            },
            4 => MonopolyInstruction::PayRent {
                property_index: src[current],
            },
            5 => MonopolyInstruction::NextTurn,
            6 => MonopolyInstruction::CheckWinner,
            7 => MonopolyInstruction::DrawCard {
                deck_type: match src[current] {
                    0 => DeckType::CommunityChest,
                    1 => DeckType::Chance,
                    _ => return Err(ProgramError::InvalidInstructionData),
                },
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
