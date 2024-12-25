use sha3::{Digest, Keccak256};
use solana_program::{
    clock::Clock,
    sysvar::Sysvar,
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::state::{TileType, Color, Property};

/// Generate a random number using on-chain data
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_number() {
        use solana_program::clock::Clock;
        use std::cell::RefCell;
        
        // Create mock Clock sysvar account
        let program_id = Pubkey::new_unique();
        let key = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![0; std::mem::size_of::<Clock>()];
        let clock = Clock {
            slot: 100,
            epoch_start_timestamp: 1000,
            epoch: 1,
            leader_schedule_epoch: 1,
            unix_timestamp: 1000,
        };
        // Serialize clock data manually since pack_into_slice is not available
        let mut offset = 0;
        data[offset..offset + 8].copy_from_slice(&clock.slot.to_le_bytes());
        offset += 8;
        data[offset..offset + 8].copy_from_slice(&clock.epoch_start_timestamp.to_le_bytes());
        offset += 8;
        data[offset..offset + 8].copy_from_slice(&clock.epoch.to_le_bytes());
        offset += 8;
        data[offset..offset + 8].copy_from_slice(&clock.leader_schedule_epoch.to_le_bytes());
        offset += 8;
        data[offset..offset + 8].copy_from_slice(&clock.unix_timestamp.to_le_bytes());
        
        let clock_account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data.as_mut_slice(),
            &program_id,
            false,
            0,
        );

        // Test range boundaries
        for i in 0..10 {
            let result = generate_random_number(&clock_account, i, 10).unwrap();
            assert!(result >= 1 && result <= 10, "Generated number {} should be between 1 and 10", result);
        }
        
        // Test min == max
        let result = generate_random_number(&clock_account, 100, 5).unwrap();
        assert!(result >= 1 && result <= 5, "Result should be between 1 and 5");
    }

    #[test]
    fn test_roll_dice() {
        use solana_program::clock::Clock;
        use std::cell::RefCell;
        
        // Create mock Clock sysvar account
        let program_id = Pubkey::new_unique();
        let key = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![0; std::mem::size_of::<Clock>()];
        let clock = Clock {
            slot: 100,
            epoch_start_timestamp: 1000,
            epoch: 1,
            leader_schedule_epoch: 1,
            unix_timestamp: 1000,
        };
        // Serialize clock data manually since pack_into_slice is not available
        let mut offset = 0;
        data[offset..offset + 8].copy_from_slice(&clock.slot.to_le_bytes());
        offset += 8;
        data[offset..offset + 8].copy_from_slice(&clock.epoch_start_timestamp.to_le_bytes());
        offset += 8;
        data[offset..offset + 8].copy_from_slice(&clock.epoch.to_le_bytes());
        offset += 8;
        data[offset..offset + 8].copy_from_slice(&clock.leader_schedule_epoch.to_le_bytes());
        offset += 8;
        data[offset..offset + 8].copy_from_slice(&clock.unix_timestamp.to_le_bytes());
        
        let clock_account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data.as_mut_slice(),
            &program_id,
            false,
            0,
        );

        // Test multiple rolls to verify range
        for i in 0..10 {
            let (dice1, dice2) = roll_dice(&clock_account, i).unwrap();
            
            // Each die should be between 1 and 6
            assert!(dice1 >= 1 && dice1 <= 6, "First die value {} should be between 1 and 6", dice1);
            assert!(dice2 >= 1 && dice2 <= 6, "Second die value {} should be between 1 and 6", dice2);
            
            // Total should be between 2 and 12
            let total = dice1 + dice2;
            assert!(total >= 2 && total <= 12, "Total {} should be between 2 and 12", total);
        }
    }

    #[test]
    fn test_calculate_rent() {
        // Test base rent with no houses
        assert_eq!(calculate_rent(100, 0), 100);
        
        // Test rent with 1-4 houses
        assert_eq!(calculate_rent(100, 1), 500);  // 5x base rent
        assert_eq!(calculate_rent(100, 2), 1500); // 15x base rent
        assert_eq!(calculate_rent(100, 3), 4500); // 45x base rent
        assert_eq!(calculate_rent(100, 4), 8000); // 80x base rent
        
        // Test hotel (5 houses)
        assert_eq!(calculate_rent(100, 5), 12500); // 125x base rent
        
        // Test invalid number of houses (should return base rent)
        assert_eq!(calculate_rent(100, 6), 100);
    }

    #[test]
    fn test_owns_color_group() {
        use solana_program::pubkey::Pubkey;
        
        // Test case: Player owns all properties in color group
        let owner = Pubkey::new_unique();
        let board = vec![
            TileType::Property(Property {
                owner: Some(owner),
                color: Color::Brown,
                houses: 0,
                name: "Mediterranean Avenue".to_string(),
                cost: 60,
                rent: vec![2, 10, 30, 90, 160, 250],
                house_cost: 50,
                hotel_cost: 50,
                house_rent: vec![10, 30, 90, 160, 250],
            }),
            TileType::Property(Property {
                owner: Some(owner),
                color: Color::Brown,
                houses: 0,
                name: "Baltic Avenue".to_string(),
                cost: 60,
                rent: vec![4, 20, 60, 180, 320, 450],
                house_cost: 50,
                hotel_cost: 50,
                house_rent: vec![20, 60, 180, 320, 450],
            })
        ];
        
        let player_properties = vec![0, 1];
        assert!(owns_color_group(&player_properties, &board, Color::Brown));

        // Test case: Player owns some but not all properties
        let partial_properties = vec![0];
        assert!(!owns_color_group(&partial_properties, &board, Color::Brown));

        // Test case: Player owns none of the properties
        let empty_properties: Vec<u8> = vec![];
        assert!(!owns_color_group(&empty_properties, &board, Color::Brown));

        // Test case: Empty board
        let empty_board: Vec<TileType> = vec![];
        assert!(!owns_color_group(&empty_properties, &empty_board, Color::Purple));
    }

    #[test]
    fn test_can_build_house() {
        use solana_program::pubkey::Pubkey;
        
        // Setup test board with properties
        let owner = Pubkey::new_unique();
        let board = vec![
            TileType::Property(Property {
                owner: Some(owner),
                color: Color::DarkBlue,
                houses: 0,
                name: "Park Place".to_string(),
                cost: 350,
                rent: vec![35, 175, 500, 1100, 1300, 1500],
                house_cost: 200,
                hotel_cost: 200,
            }),
            TileType::Property(Property {
                owner: Some(owner),
                color: Color::DarkBlue,
                houses: 0,
                name: "Boardwalk".to_string(),
                cost: 400,
                rent: vec![50, 200, 600, 1400, 1700, 2000],
                house_cost: 200,
                hotel_cost: 200,
            })
        ];

        let player_properties = vec![0, 1];
        
        // Test case: Can build when owns color group and houses are even
        assert!(can_build_house(0, &player_properties, &board));

        // Test case: Cannot build when doesn't own all properties
        let partial_properties = vec![0];
        assert!(!can_build_house(0, &partial_properties, &board));

        // Test case: Cannot build on non-property tile
        let mut mixed_board = board.clone();
        mixed_board.push(TileType::Special(SpecialTile::Chance));
        assert!(!can_build_house(2, &player_properties, &mixed_board));

        // Test case: Cannot build more than 5 houses (hotel)
        let mut hotel_board = vec![
            TileType::Property(Property {
                owner: Some(owner),
                color: Color::DarkBlue,
                houses: 5,
                name: "Park Place".to_string(),
                cost: 350,
                rent: vec![35, 175, 500, 1100, 1300, 1500],
                house_cost: 200,
                hotel_cost: 200,
            }),
            TileType::Property(Property {
                owner: Some(owner),
                color: Color::DarkBlue,
                houses: 5,
                name: "Boardwalk".to_string(),
                cost: 400,
                rent: vec![50, 200, 600, 1400, 1700, 2000],
                house_cost: 200,
                hotel_cost: 200,
            })
        ];
        assert!(!can_build_house(0, &player_properties, &hotel_board));

        // Test case: Empty board
        let empty_board: Vec<TileType> = vec![];
        let empty_properties: Vec<u8> = vec![];
        assert!(!can_build_house(0, &empty_properties, &empty_board));
    }
}

pub fn generate_random_number(
    clock_sysvar_info: &AccountInfo,
    seed: u64,
    range: u8,
) -> Result<u8, ProgramError> {
    let clock = Clock::from_account_info(clock_sysvar_info)?;
    
    // Combine multiple sources of entropy
    let mut hasher = Keccak256::new();
    hasher.update(&clock.slot.to_le_bytes());
    hasher.update(&clock.unix_timestamp.to_le_bytes());
    hasher.update(&seed.to_le_bytes());
    
    let hash = hasher.finalize();
    let random_value = u8::from_le_bytes([hash[0]]);
    
    // Map to range [1, range]
    Ok((random_value % range) + 1)
}

/// Roll two dice using on-chain randomization
pub fn roll_dice(
    clock_sysvar_info: &AccountInfo,
    seed: u64,
) -> Result<(u8, u8), ProgramError> {
    let dice1 = generate_random_number(clock_sysvar_info, seed, 6)?;
    let dice2 = generate_random_number(clock_sysvar_info, seed.wrapping_add(1), 6)?;
    Ok((dice1, dice2))
}

/// Calculate rent for a property based on number of houses
pub fn calculate_rent(base_rent: u64, houses: u8) -> u64 {
    match houses {
        0 => base_rent,
        1 => base_rent * 5,
        2 => base_rent * 15,
        3 => base_rent * 45,
        4 => base_rent * 80,
        5 => base_rent * 125,  // Hotel
        _ => base_rent,
    }
}

/// Check if a player owns all properties of a color group
pub fn owns_color_group(
    player_properties: &[u8],
    board: &[TileType],
    color: Color,
) -> bool {
    let color_properties: Vec<usize> = board.iter().enumerate()
        .filter_map(|(i, tile)| {
            if let TileType::Property(prop) = tile {
                if prop.color == color {
                    Some(i)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if color_properties.is_empty() {
        return false;
    }

    color_properties.iter()
        .all(|&idx| player_properties.contains(&(idx as u8)))
}

/// Check if a player can build a house on a property
pub fn can_build_house(
    property_index: u8,
    player_properties: &[u8],
    board: &[TileType],
) -> bool {
    let idx = property_index as usize;
    if idx >= board.len() {
        return false;
    }

    if let TileType::Property(prop) = &board[idx] {
        // Can't build more than 5 houses (hotel)
        if prop.houses >= 5 {
            return false;
        }

        // Must own all properties of the same color
        owns_color_group(player_properties, board, prop.color.clone())
    } else {
        false
    }
}
