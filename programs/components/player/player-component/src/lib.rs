use bolt_lang::prelude::*;
use anchor_lang::prelude::*;

#[derive(Component)]
pub struct Player {
    pub name: String,
    pub cash: u64,
    pub pos: u8,
    pub jail_turns: u8,
    pub properties: Vec<u8>,  // Property indices
}

#[program]
pub mod player_component {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_player(ctx: Context<Initialize>, name: String) -> Result<()> {
        let player = Player {
            name,
            cash: 1500,  // Starting cash
            pos: 0,      // Start at GO
            jail_turns: 0,
            properties: vec![],
        };
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
