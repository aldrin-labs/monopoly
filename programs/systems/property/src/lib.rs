use bolt_lang::prelude::*;
use anchor_lang::prelude::*;
use common::{Property, TileType};
use player_component::Player;
use game_state_component::GameState;

declare_id!("9UP7Ezw5qK7eRaLstChu9QVBzku7G6aGK5gQsSQrvB4S");

#[system]
pub mod property {
    use super::*;

    pub fn handle_property(ctx: Context<Components>) -> Result<Components> {
        let game_state = &mut ctx.accounts.game_state;
        let player = &mut ctx.accounts.player;
        let property = &mut ctx.accounts.property;
        
        // Unowned property
        if property.data.owner.is_none() && player.cash >= property.data.cost {
            player.cash = player.cash.saturating_sub(property.data.cost);
            property.data.owner = Some(player.key());
            player.properties.push(ctx.accounts.property_index.key().to_bytes()[0] as u8);
            return Ok(ctx.accounts);
        }
        
        // Owned by another player
        if let Some(owner) = property.data.owner {
            if owner != player.key() {
                let rent = property.data.rent[property.data.houses as usize];
                player.cash = player.cash.saturating_sub(rent);
                if let Some(owner_account) = ctx.accounts.property_owner.as_ref() {
                    owner_account.cash += rent;
                }
            }
        }
        
        Ok(ctx.accounts)
    }

    pub fn build_house(ctx: Context<Components>) -> Result<Components> {
        let player = &mut ctx.accounts.player;
        let property = &mut ctx.accounts.property;
        
        require!(
            property.data.owner == Some(player.key()),
            MonopolyError::NotPropertyOwner
        );
        
        require!(property.data.houses < 5, MonopolyError::MaxHousesReached);
        
        let cost = if property.data.houses == 4 {
            property.data.hotel_cost
        } else {
            property.data.house_cost
        };
        
        require!(player.cash >= cost, MonopolyError::InsufficientFunds);
        
        player.cash = player.cash.saturating_sub(cost);
        property.data.houses += 1;
        
        Ok(ctx.accounts)
    }

    #[system_input]
    pub struct Components {
        #[account(mut)]
        pub game_state: GameState,
        #[account(mut)]
        pub player: Player,
        #[account(mut)]
        pub property: Property,
        /// Optional owner account if property is owned
        pub property_owner: Option<Account<'info, Player>>,
        /// Used to get property index
        pub property_index: AccountInfo<'info>,
    }
}

#[error_code]
pub enum MonopolyError {
    #[msg("Player is not the property owner")]
    NotPropertyOwner,
    #[msg("Maximum number of houses/hotel reached")]
    MaxHousesReached,
    #[msg("Insufficient funds")]
    InsufficientFunds,
}
