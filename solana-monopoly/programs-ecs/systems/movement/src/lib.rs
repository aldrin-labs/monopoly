use bolt_lang::prelude::*;
use anchor_lang::prelude::*;
use common::{TileType, SpecialTile};
use player_component::Player;
use game_state_component::GameState;

declare_id!("8UP7Ezw5qK7eRaLstChu9QVBzku7G6aGK5gQsSQrvB4R");

#[system]
pub mod movement {
    use super::*;

    pub fn execute(ctx: Context<Components>, _args: Vec<u8>) -> Result<Components> {
        let game_state = &mut ctx.accounts.game_state;
        let player = &mut ctx.accounts.player;

        // Roll dice (2d6)
        let d1 = (ctx.accounts.random_seed.key().to_bytes()[0] % 6 + 1) as u8;
        let d2 = (ctx.accounts.random_seed.key().to_bytes()[1] % 6 + 1) as u8;
        
        // Handle jail logic
        if player.jail_turns > 0 {
            if d1 == d2 {
                player.jail_turns = 0;
            } else {
                player.jail_turns += 1;
                if player.jail_turns >= 3 {
                    player.cash = player.cash.saturating_sub(50);
                    player.jail_turns = 0;
                }
                return Ok(ctx.accounts);
            }
        }
        
        // Move player
        let old_pos = player.pos;
        player.pos = (player.pos + d1 + d2) % 40;
        
        // Pass GO
        if player.pos < old_pos {
            player.cash += 200;
        }
        
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        #[account(mut)]
        pub game_state: GameState,
        #[account(mut)]
        pub player: Player,
        /// Random seed account for dice rolls
        pub random_seed: AccountInfo<'info>,
    }
}
