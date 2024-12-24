use bolt_lang::prelude::*;
use common::{Card, TileType};
use player_component::Player;
use game_state_component::GameState;

declare_id!("AUP7Ezw5qK7eRaLstChu9QVBzku7G6aGK5gQsSQrvB4T");

#[system]
pub mod cards {
    use super::*;

    pub fn draw_community_chest(ctx: Context<Components>) -> Result<Components> {
        let game_state = &mut ctx.accounts.game_state;
        let player = &mut ctx.accounts.player;
        
        if game_state.community_chest.is_empty() {
            return Ok(ctx.accounts);
        }
        
        let card = game_state.community_chest.remove(0);
        match card {
            Card::CollectMoney(amount) => player.cash += amount,
            Card::PayMoney(amount) => {
                player.cash = player.cash.saturating_sub(amount);
                game_state.free_parking += amount;
            },
            Card::Move(pos) => player.pos = pos,
            Card::GetOutOfJail => {
                player.jail_turns = 0;
            }
        }
        
        // Return card to bottom of deck
        game_state.community_chest.push(card);
        
        Ok(ctx.accounts)
    }

    pub fn draw_chance(ctx: Context<Components>) -> Result<Components> {
        let game_state = &mut ctx.accounts.game_state;
        let player = &mut ctx.accounts.player;
        
        if game_state.chance.is_empty() {
            return Ok(ctx.accounts);
        }
        
        let card = game_state.chance.remove(0);
        match card {
            Card::CollectMoney(amount) => player.cash += amount,
            Card::PayMoney(amount) => {
                player.cash = player.cash.saturating_sub(amount);
                game_state.free_parking += amount;
            },
            Card::Move(pos) => player.pos = pos,
            Card::GetOutOfJail => {
                player.jail_turns = 0;
            }
        }
        
        // Return card to bottom of deck
        game_state.chance.push(card);
        
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        #[account(mut)]
        pub game_state: GameState,
        #[account(mut)]
        pub player: Player,
    }
}
