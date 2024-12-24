use bolt_lang::prelude::*;
use bolt_component::{BoltComponent, BoltMetadata, Component};
use bolt_lang::{BoltContext, BoltError};

#[derive(Component)]
pub struct Player {
    pub name: String,
    pub cash: u64,
    pub pos: u8,
    pub jail_turns: u8,
    pub properties: Vec<u8>,  // Property indices
    pub bolt_metadata: BoltMetadata,
}

impl BoltComponent for Player {
    fn init_space() -> usize {
        std::mem::size_of::<Self>()
    }
}

#[bolt_program(Player)]
pub mod player_component {
    use super::*;

    pub fn initialize(ctx: &mut BoltContext) -> Result<(), BoltError> {
        Ok(())
    }

    pub fn create_player(ctx: &mut BoltContext, name: String) -> Result<(), BoltError> {
        let player = Player {
            name,
            cash: 1500,  // Starting cash
            pos: 0,      // Start at GO
            jail_turns: 0,
            properties: vec![],
            bolt_metadata: BoltMetadata::default(),
        };
        ctx.world.add_component(player)?;
        Ok(())
    }
}
