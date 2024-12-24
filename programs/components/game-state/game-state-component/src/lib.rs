use bolt_lang::prelude::*;
use bolt_component::{BoltComponent, BoltMetadata, Component};
use bolt_lang::{BoltContext, BoltError};
use common::{Card, TileType};

#[derive(Component)]
pub struct GameState {
    pub board: Vec<TileType>,
    pub players: Vec<Pubkey>,
    pub community_chest: Vec<Card>,
    pub chance: Vec<Card>,
    pub free_parking: u64,
    pub current_player: u8,
    pub bolt_metadata: BoltMetadata,
}

impl BoltComponent for GameState {
    fn init_space() -> usize {
        std::mem::size_of::<Self>()
    }
}

#[bolt_program(GameState)]
pub mod game_state_component {
    use super::*;

    pub fn initialize(ctx: &mut BoltContext) -> Result<(), BoltError> {
        Ok(())
    }

    pub fn create_game(ctx: &mut BoltContext) -> Result<(), BoltError> {
        let game_state = GameState {
            board: vec![],  // Will be initialized with create_board()
            players: vec![],
            community_chest: vec![],  // Will be initialized with create_community_chest()
            chance: vec![],  // Will be initialized with create_chance_cards()
            free_parking: 0,
            current_player: 0,
            bolt_metadata: BoltMetadata::default(),
        };
        ctx.world.add_component(game_state)?;
        Ok(())
    }
}
