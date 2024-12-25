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
        // Test InitGame instruction
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let init_instruction = MonopolyInstruction::InitGame { player_names: player_names.clone() };
        let mut packed = vec![0u8; MonopolyInstruction::LEN];
        init_instruction.pack_into_slice(&mut packed);
        let unpacked = MonopolyInstruction::try_from_slice(&packed).unwrap();
        assert!(matches!(unpacked, MonopolyInstruction::InitGame { player_names: ref names } if names == &player_names));

        // Test MovePlayer instruction
        let move_instruction = MonopolyInstruction::MovePlayer;
        let mut packed = vec![0u8; MonopolyInstruction::LEN];
        move_instruction.pack_into_slice(&mut packed);
        let unpacked = MonopolyInstruction::try_from_slice(&packed).unwrap();
        assert!(matches!(unpacked, MonopolyInstruction::MovePlayer));

        // Test BuyProperty instruction
        let buy_instruction = MonopolyInstruction::BuyProperty { property_index: 5 };
        let mut packed = vec![0u8; MonopolyInstruction::LEN];
        buy_instruction.pack_into_slice(&mut packed);
        let unpacked = MonopolyInstruction::try_from_slice(&packed).unwrap();
        assert!(matches!(unpacked, MonopolyInstruction::BuyProperty { property_index } if property_index == 5));

        // Test BuildHouse instruction
        let build_instruction = MonopolyInstruction::BuildHouse { property_index: 3 };
        let mut packed = vec![0u8; MonopolyInstruction::LEN];
        build_instruction.pack_into_slice(&mut packed);
        let unpacked = MonopolyInstruction::try_from_slice(&packed).unwrap();
        assert!(matches!(unpacked, MonopolyInstruction::BuildHouse { property_index } if property_index == 3));

        // Test PayRent instruction
        let rent_instruction = MonopolyInstruction::PayRent { property_index: 7 };
        let mut packed = vec![0u8; MonopolyInstruction::LEN];
        rent_instruction.pack_into_slice(&mut packed);
        let unpacked = MonopolyInstruction::try_from_slice(&packed).unwrap();
        assert!(matches!(unpacked, MonopolyInstruction::PayRent { property_index } if property_index == 7));

        // Test NextTurn instruction
        let next_turn_instruction = MonopolyInstruction::NextTurn;
        let mut packed = vec![0u8; MonopolyInstruction::LEN];
        next_turn_instruction.pack_into_slice(&mut packed);
        let unpacked = MonopolyInstruction::try_from_slice(&packed).unwrap();
        assert!(matches!(unpacked, MonopolyInstruction::NextTurn));

        // Test CheckWinner instruction
        let check_winner_instruction = MonopolyInstruction::CheckWinner;
        let mut packed = vec![0u8; MonopolyInstruction::LEN];
        check_winner_instruction.pack_into_slice(&mut packed);
        let unpacked = MonopolyInstruction::try_from_slice(&packed).unwrap();
        assert!(matches!(unpacked, MonopolyInstruction::CheckWinner));

        // Test DrawCard instruction
        let draw_instruction = MonopolyInstruction::DrawCard { deck_type: DeckType::CommunityChest };
        let mut packed = vec![0u8; MonopolyInstruction::LEN];
        draw_instruction.pack_into_slice(&mut packed);
        let unpacked = MonopolyInstruction::try_from_slice(&packed).unwrap();
        assert!(matches!(unpacked, MonopolyInstruction::DrawCard { deck_type } if matches!(deck_type, DeckType::CommunityChest)));
    }

    #[test]
    fn test_instruction_unpack() {
        // Test InitGame instruction
        let mut init_data = vec![0u8]; // Variant index 0
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let names_len = player_names.len() as u32;
        init_data.extend_from_slice(&names_len.to_le_bytes());
        for name in &player_names {
            let name_bytes = name.as_bytes();
            init_data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            init_data.extend_from_slice(name_bytes);
        }
        let mut rest = &init_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_ok());
        if let MonopolyInstruction::InitGame { player_names: unpacked_names } = result.unwrap() {
            assert_eq!(unpacked_names, player_names);
        } else {
            panic!("Expected InitGame instruction");
        }

        // Test MovePlayer instruction
        let move_data = vec![1u8];
        let mut rest = &move_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), MonopolyInstruction::MovePlayer));

        // Test BuyProperty instruction
        let mut buy_data = vec![2u8];
        buy_data.push(5); // property_index
        let mut rest = &buy_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_ok());
        if let MonopolyInstruction::BuyProperty { property_index } = result.unwrap() {
            assert_eq!(property_index, 5);
        } else {
            panic!("Expected BuyProperty instruction");
        }

        // Test BuildHouse instruction
        let mut build_data = vec![3u8];
        build_data.push(3); // property_index
        let mut rest = &build_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_ok());
        if let MonopolyInstruction::BuildHouse { property_index } = result.unwrap() {
            assert_eq!(property_index, 3);
        } else {
            panic!("Expected BuildHouse instruction");
        }

        // Test PayRent instruction
        let mut rent_data = vec![4u8];
        rent_data.push(7); // property_index
        let mut rest = &rent_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_ok());
        if let MonopolyInstruction::PayRent { property_index } = result.unwrap() {
            assert_eq!(property_index, 7);
        } else {
            panic!("Expected PayRent instruction");
        }

        // Test NextTurn instruction
        let next_turn_data = vec![5u8];
        let mut rest = &next_turn_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), MonopolyInstruction::NextTurn));

        // Test CheckWinner instruction
        let check_winner_data = vec![6u8];
        let mut rest = &check_winner_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), MonopolyInstruction::CheckWinner));

        // Test DrawCard instruction
        let mut draw_data = vec![7u8];
        draw_data.push(0); // CommunityChest
        let mut rest = &draw_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_ok());
        if let MonopolyInstruction::DrawCard { deck_type } = result.unwrap() {
            assert!(matches!(deck_type, DeckType::CommunityChest));
        } else {
            panic!("Expected DrawCard instruction");
        }

        // Test invalid variant
        let invalid_data = vec![255u8];
        let mut rest = &invalid_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_err());

        // Test empty data
        let empty_data: Vec<u8> = vec![];
        let mut rest = &empty_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_err());

        // Test truncated data
        let mut truncated_data = vec![2u8]; // BuyProperty without property_index
        let mut rest = &truncated_data[..];
        let result = MonopolyInstruction::unpack_from_slice(&mut rest);
        assert!(result.is_err());
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
