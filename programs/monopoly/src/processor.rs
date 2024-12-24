use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::clock::Clock,
    sysvar::Sysvar,
};

use crate::{
    error::MonopolyError,
    instruction::MonopolyInstruction,
    state::{Game, Player, TileType, SpecialTile, Card},
    account::{GameAccount, DeckType, validate_game_account, validate_card_deck},
    utils::{roll_dice, calculate_rent, can_build_house},
    board::{create_board, create_community_chest, create_chance_cards},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: MonopolyInstruction,
) -> ProgramResult {
    match instruction {
        MonopolyInstruction::InitGame { player_names } => {
            msg!("Instruction: InitGame");
            process_init_game(program_id, accounts, player_names)?
        }
        MonopolyInstruction::MovePlayer => {
            msg!("Instruction: MovePlayer");
            process_move_player(program_id, accounts)?
        }
        MonopolyInstruction::BuyProperty { property_index } => {
            msg!("Instruction: BuyProperty");
            process_buy_property(program_id, accounts, property_index)?
        }
        MonopolyInstruction::BuildHouse { property_index } => {
            msg!("Instruction: BuildHouse");
            process_build_house(program_id, accounts, property_index)?
        }
        MonopolyInstruction::PayRent { property_index } => {
            msg!("Instruction: PayRent");
            process_pay_rent(program_id, accounts, property_index)?
        }
        MonopolyInstruction::NextTurn => {
            msg!("Instruction: NextTurn");
            process_next_turn(program_id, accounts)?
        }
        MonopolyInstruction::CheckWinner => {
            msg!("Instruction: CheckWinner");
            process_check_winner(program_id, accounts)?
        }
        MonopolyInstruction::DrawCard { deck_type } => {
            msg!("Instruction: DrawCard");
            process_draw_card(program_id, accounts, deck_type)?
        }
    }
    Ok(())
}

fn process_draw_card(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    deck_type: DeckType,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let game_account = next_account_info(accounts_iter)?;
    let player_account = next_account_info(accounts_iter)?;
    let card_deck_account = next_account_info(accounts_iter)?;
    let clock_sysvar = next_account_info(accounts_iter)?;

    if !player_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_data = validate_game_account(game_account)?;
    let mut card_deck = validate_card_deck(card_deck_account)?;
    
    // Initialize deck if needed
    if !card_deck.is_initialized {
        card_deck.cards = match deck_type {
            DeckType::CommunityChest => create_community_chest(),
            DeckType::Chance => create_chance_cards(),
        };
        card_deck.is_initialized = true;
        card_deck.game = *game_account.key;
        card_deck.deck_type = deck_type;
    }

    // Get current player
    let current_player = game_data.game.current_player as usize;
    let player = &mut game_data.game.players[current_player];

    // Draw a card using clock as randomization source
    let clock = Clock::from_account_info(clock_sysvar)?;
    let card_index = (clock.slot % card_deck.cards.len() as u64) as usize;
    let card = &card_deck.cards[card_index];

    // Process card effect
    match card {
        Card::CollectMoney(amount) => {
            player.cash += amount;
            msg!("Player collected {} from card", amount);
        },
        Card::PayMoney(amount) => {
            if player.cash < *amount {
                return Err(MonopolyError::InsufficientBalance.into());
            }
            player.cash -= amount;
            game_data.game.free_parking += amount;
            msg!("Player paid {} to free parking", amount);
        },
        Card::Move(position) => {
            // Pay GO bonus if passing GO
            if *position < player.position {
                player.cash += 200;
            }
            player.position = *position;
            msg!("Player moved to position {}", position);
        },
        Card::GetOutOfJail => {
            player.get_out_of_jail_cards += 1;
            msg!("Player received Get Out of Jail Free card");
        },
    }

    // Rotate card to bottom of deck
    let card = card_deck.cards.remove(card_index);
    card_deck.cards.push(card);

    // Save state
    game_data.pack_into_slice(&mut game_account.data.borrow_mut());
    card_deck.pack_into_slice(&mut card_deck_account.data.borrow_mut());

    Ok(())
}

    // Helper function to check if a player is bankrupt
    fn is_player_bankrupt(player: &Player) -> bool {
        player.cash == 0 && player.properties.is_empty()
    }

    fn process_next_turn(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let game_account = next_account_info(accounts_iter)?;
        let player_account = next_account_info(accounts_iter)?;

        if !player_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut game_data = validate_game_account(game_account)?;
        
        // Move to next player
        game_data.game.current_player = (game_data.game.current_player + 1) % game_data.game.players.len() as u8;
        
        // Skip bankrupt players
        while is_player_bankrupt(&game_data.game.players[game_data.game.current_player as usize]) {
            game_data.game.current_player = (game_data.game.current_player + 1) % game_data.game.players.len() as u8;
        }

        game_data.pack_into_slice(&mut game_account.data.borrow_mut());
        Ok(())
    }

    fn process_check_winner(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let game_account = next_account_info(accounts_iter)?;

        let game_data = validate_game_account(game_account)?;
        
        // Count non-bankrupt players
        let active_players: Vec<_> = game_data.game.players.iter()
            .enumerate()
            .filter(|(_, player)| !is_player_bankrupt(player))
            .collect();

        if active_players.len() == 1 {
            msg!("Game Over! Winner: {}", active_players[0].1.name);
            return Ok(());
        }

        // If more than one player is active, compare total worth
        let mut max_worth = 0;
        let mut winner_index = 0;
        
        for (i, player) in game_data.game.players.iter().enumerate() {
            if is_player_bankrupt(player) {
                continue;
            }

            let mut total_worth = player.cash;
            
            // Add property values
            for &prop_idx in &player.properties {
                if let TileType::Property(property) = &game_data.game.board[prop_idx as usize] {
                    total_worth += property.cost;
                    total_worth += property.house_cost * property.houses as u64;
                }
            }

            if total_worth > max_worth {
                max_worth = total_worth;
                winner_index = i;
            }
        }

        msg!("Current leader: {} with total worth: {}", 
            game_data.game.players[winner_index].name,
            max_worth
        );

        Ok(())
    }

fn process_init_game(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    player_names: Vec<String>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let game_account = next_account_info(accounts_iter)?;
    let rent_payer = next_account_info(accounts_iter)?;

    if !rent_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let game_data = GameAccount {
        is_initialized: true,
        game: Game {
            board: create_board(),
            players: player_names.iter().map(|name| Player {
                name: name.clone(),
                cash: 1500,
                position: 0,
                jail_turns: 0,
                properties: vec![],
                get_out_of_jail_cards: 0,
            }).collect(),
            current_player: 0,
            free_parking: 0,
            initialized: true,
        },
    };

    game_data.pack_into_slice(&mut game_account.data.borrow_mut());
    Ok(())
}

fn process_move_player(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let game_account = next_account_info(accounts_iter)?;
    let player_account = next_account_info(accounts_iter)?;
    let clock_sysvar = next_account_info(accounts_iter)?;

    if !player_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_data = validate_game_account(game_account)?;
    let current_player = game_data.game.current_player as usize;
    let player = &mut game_data.game.players[current_player];

    // Roll dice and move player
    let seed = Clock::from_account_info(clock_sysvar)?.slot;
    let (dice1, dice2) = roll_dice(clock_sysvar, seed)?;
    let total_move = dice1 + dice2;

    // Handle jail logic
    if player.jail_turns > 0 {
        if dice1 == dice2 {
            player.jail_turns = 0;
        } else {
            player.jail_turns += 1;
            if player.jail_turns >= 3 {
                player.cash = player.cash.saturating_sub(50);
                player.jail_turns = 0;
            }
            return Ok(());
        }
    }

    // Move player
    let new_position = (player.position + total_move) % 40;
    
    // Pass GO
    if new_position < player.position {
        player.cash += 200;
    }
    
    player.position = new_position;

    // Process landed tile
    match &game_data.game.board[new_position as usize] {
        TileType::Special(special) => match special {
            SpecialTile::GoToJail => {
                player.position = 10; // Jail position
                player.jail_turns = 1;
            },
            SpecialTile::IncomeTax => {
                player.cash = player.cash.saturating_sub(200);
            },
            SpecialTile::LuxuryTax => {
                player.cash = player.cash.saturating_sub(100);
            },
            _ => {},
        },
        _ => {},
    }

    game_data.pack_into_slice(&mut game_account.data.borrow_mut());
    Ok(())
}

fn process_buy_property(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    property_index: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let game_account = next_account_info(accounts_iter)?;
    let player_account = next_account_info(accounts_iter)?;

    if !player_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_data = validate_game_account(game_account)?;
    let current_player = game_data.game.current_player as usize;
    let player = &mut game_data.game.players[current_player];

    match &mut game_data.game.board[property_index as usize] {
        TileType::Property(property) => {
            if property.owner.is_some() {
                return Err(MonopolyError::PropertyAlreadyOwned.into());
            }

            if player.cash < property.cost {
                return Err(MonopolyError::InsufficientBalance.into());
            }

            player.cash -= property.cost;
            property.owner = Some(player_account.key.clone());
            player.properties.push(property_index);
        },
        _ => return Err(MonopolyError::InvalidProperty.into()),
    }

    game_data.pack_into_slice(&mut game_account.data.borrow_mut());
    Ok(())
}

fn process_build_house(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    property_index: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let game_account = next_account_info(accounts_iter)?;
    let player_account = next_account_info(accounts_iter)?;

    if !player_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_data = validate_game_account(game_account)?;
    let current_player = game_data.game.current_player as usize;
    let player = &mut game_data.game.players[current_player];

    if !can_build_house(property_index, &player.properties, &game_data.game.board) {
        return Err(MonopolyError::InvalidProperty.into());
    }

    match &mut game_data.game.board[property_index as usize] {
        TileType::Property(property) => {
            if property.owner != Some(player_account.key.clone()) {
                return Err(MonopolyError::NotPropertyOwner.into());
            }

            if property.houses >= 5 {
                return Err(MonopolyError::MaximumHousesReached.into());
            }

            if player.cash < property.house_cost {
                return Err(MonopolyError::InsufficientBalance.into());
            }

            player.cash -= property.house_cost;
            property.houses += 1;
        },
        _ => return Err(MonopolyError::InvalidProperty.into()),
    }

    game_data.pack_into_slice(&mut game_account.data.borrow_mut());
    Ok(())
}

fn process_pay_rent(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    property_index: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let game_account = next_account_info(accounts_iter)?;
    let player_account = next_account_info(accounts_iter)?;
    let owner_account = next_account_info(accounts_iter)?;

    if !player_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_data = validate_game_account(game_account)?;
    let current_player = game_data.game.current_player as usize;
    let player = &mut game_data.game.players[current_player];

    match &game_data.game.board[property_index as usize] {
        TileType::Property(property) => {
            if property.owner != Some(owner_account.key.clone()) {
                return Err(MonopolyError::NotRentOwner.into());
            }

            let rent = calculate_rent(property.rent[0], property.houses);
            if player.cash < rent {
                return Err(MonopolyError::InsufficientBalance.into());
            }

            player.cash -= rent;

            // Find owner in players list and add rent
            for p in game_data.game.players.iter_mut() {
                if p.properties.contains(&property_index) {
                    p.cash += rent;
                    break;
                }
            }
        },
        _ => return Err(MonopolyError::InvalidProperty.into()),
    }

    game_data.pack_into_slice(&mut game_account.data.borrow_mut());
    Ok(())
}
