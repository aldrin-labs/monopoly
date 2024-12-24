use sha3::{Digest, Keccak256};
use solana_program::{
    clock::Clock,
    sysvar::Sysvar,
    account_info::AccountInfo,
    program_error::ProgramError,
};

use crate::state::{TileType, Color};

/// Generate a random number using on-chain data
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
    let color_properties: Vec<u8> = board.iter().enumerate()
        .filter_map(|(i, tile)| {
            if let TileType::Property(prop) = tile {
                if prop.color == color {
                    Some(i as u8)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    color_properties.iter().all(|&prop_idx| player_properties.contains(&prop_idx))
}

/// Check if a player can build a house on a property
pub fn can_build_house(
    property_index: u8,
    player_properties: &[u8],
    board: &[TileType],
) -> bool {
    if let TileType::Property(prop) = &board[property_index as usize] {
        if prop.houses >= 5 {
            return false;
        }
        owns_color_group(player_properties, board, prop.color.clone())
    } else {
        false
    }
}
