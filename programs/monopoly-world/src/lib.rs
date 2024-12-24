use bolt_lang::prelude::*;
use anchor_lang::prelude::*;
use common::{Card, Color, Property, TileType, SpecialTile};
use player_component::Player;
use property_component::Property as PropertyComponent;
use game_state_component::GameState;

declare_id!("BUP7Ezw5qK7eRaLstChu9QVBzku7G6aGK5gQsSQrvB4W");

#[program]
pub mod monopoly_world {
    use super::*;

    pub fn initialize_game(ctx: Context<InitializeGame>) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        
        // Initialize board with properties and special tiles
        game_state.board = create_board();
        
        // Initialize card decks
        game_state.community_chest = create_community_chest_cards();
        game_state.chance = create_chance_cards();
        
        // Initialize other game state
        game_state.free_parking = 0;
        game_state.current_player = 0;
        game_state.players = vec![];
        
        Ok(())
    }

    pub fn join_game(ctx: Context<JoinGame>, name: String) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        let player = &mut ctx.accounts.player;
        
        // Initialize player
        *player = Player {
            name,
            cash: 1500, // Starting cash
            pos: 0,     // Start at GO
            jail_turns: 0,
            properties: vec![],
        };
        
        // Add player to game
        game_state.players.push(player.key());
        
        Ok(())
    }
}

fn create_board() -> Vec<TileType> {
    let mut board = Vec::with_capacity(40);
    
    // Add GO
    board.push(TileType::Special(SpecialTile::Go));
    
    // Brown properties
    board.push(TileType::Property(Property {
        name: "Mediterranean Avenue".to_string(),
        color: Color::Purple,
        cost: 60,
        rent: vec![2, 10, 30, 90, 160, 250],
        house_cost: 50,
        hotel_cost: 50,
        owner: None,
        houses: 0,
    }));
    
    // Community Chest
    board.push(TileType::Special(SpecialTile::CommunityChest));
    
    // Continue adding all properties and special tiles...
    // (Abbreviated for brevity - would include all 40 spaces)
    
    board
}

fn create_community_chest_cards() -> Vec<Card> {
    vec![
        Card::CollectMoney(200), // Bank error in your favor
        Card::PayMoney(50),      // Doctor's fee
        Card::Move(0),           // Advance to GO
        Card::GetOutOfJail,      // Get out of jail free
        // Add more community chest cards...
    ]
}

fn create_chance_cards() -> Vec<Card> {
    vec![
        Card::Move(0),           // Advance to GO
        Card::Move(24),          // Advance to Illinois Avenue
        Card::CollectMoney(50),  // Bank pays you dividend
        Card::GetOutOfJail,      // Get out of jail free
        // Add more chance cards...
    ]
}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    #[account(mut)]
    pub player: Account<'info, Player>,
}
