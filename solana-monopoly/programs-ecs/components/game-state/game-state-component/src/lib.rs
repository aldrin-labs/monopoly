use bolt_lang::prelude::*;
use anchor_lang::prelude::*;
use common::{Card, TileType};

#[derive(Component)]
pub struct GameState {
    pub board: Vec<TileType>,
    pub players: Vec<Pubkey>,
    pub community_chest: Vec<Card>,
    pub chance: Vec<Card>,
    pub free_parking: u64,
    pub current_player: u8,
}

#[program]
pub mod game_state_component {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_game(ctx: Context<Initialize>) -> Result<()> {
        let game_state = GameState {
            board: vec![],  // Will be initialized with create_board()
            players: vec![],
            community_chest: vec![],  // Will be initialized with create_community_chest()
            chance: vec![],  // Will be initialized with create_chance_cards()
            free_parking: 0,
            current_player: 0,
        };
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
