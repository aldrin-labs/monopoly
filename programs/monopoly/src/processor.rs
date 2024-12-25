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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_init_game() {
        // Create test accounts
        let program_id = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();

        let mut game_lamports = 0;
        let mut player_lamports = 0;

        let mut game_data = vec![0; 8192];
        let mut player_data = vec![0; 32];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
            0,
        );

        let player_account = AccountInfo::new(
            &player_key,
            true,  // Player must be signer
            true,
            &mut player_lamports,
            &mut player_data,
            &program_id,
            false,
            0,
        );

        // Test case 1: Successful game initialization
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            let player_names = vec![String::from("Player 1"), String::from("Player 2")];
            process_init_game(&program_id, accounts, player_names.clone()).unwrap();

            let game_state = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert!(game_state.is_initialized);
            assert!(game_state.game.initialized);
            assert_eq!(game_state.game.players.len(), 2);
            assert_eq!(game_state.game.players[0].name, "Player 1");
            assert_eq!(game_state.game.players[1].name, "Player 2");
            assert_eq!(game_state.game.players[0].cash, 1500); // Starting cash
            assert_eq!(game_state.game.current_player, 0);
            assert_eq!(game_state.game.free_parking, 0);
            assert_eq!(game_state.game.board.len(), 40); // Standard Monopoly board size
        }

        // Test case 2: Missing player signature
        {
            let mut unsigned_player_account = player_account.clone();
            unsigned_player_account.is_signer = false;

            let accounts = &[game_account.clone(), unsigned_player_account];
            let player_names = vec![String::from("Player 1")];
            let result = process_init_game(&program_id, accounts, player_names.clone());
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::MissingRequiredSignature);
        }

        // Test case 3: Invalid number of players (less than 2)
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            let player_names = vec![String::from("Player 1")];
            let result = process_init_game(&program_id, accounts, player_names.clone());
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InvalidPlayerCount.into());
        }

        // Test case 4: Invalid number of players (more than 8)
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            let player_names = vec![
                String::from("Player 1"),
                String::from("Player 2"),
                String::from("Player 3"),
                String::from("Player 4"),
                String::from("Player 5"),
                String::from("Player 6"),
                String::from("Player 7"),
                String::from("Player 8"),
                String::from("Player 9"),
            ];
            let result = process_init_game(&program_id, accounts, player_names.clone());
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InvalidPlayerCount.into());
        }

        // Test case 5: Reinitializing an already initialized game
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            let player_names = vec![String::from("Player 1"), String::from("Player 2")];
            process_init_game(&program_id, accounts, player_names.clone()).unwrap();

            let result = process_init_game(&program_id, accounts, player_names.clone());
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::AccountAlreadyInitialized);
        }
    }

    #[test]
    fn test_is_player_bankrupt() {
        // Test case 1: Player with cash and properties is not bankrupt
        let player = Player {
            name: String::from("Rich Player"),
            cash: 500,
            position: 0,
            jail_turns: 0,
            properties: vec![1, 2, 3],
            get_out_of_jail_cards: 0,
        };
        assert!(!is_player_bankrupt(&player));

        // Test case 2: Player with cash but no properties is not bankrupt
        let player = Player {
            name: String::from("Cash Only Player"),
            cash: 100,
            position: 0,
            jail_turns: 0,
            properties: vec![],
            get_out_of_jail_cards: 0,
        };
        assert!(!is_player_bankrupt(&player));

        // Test case 3: Player with no cash but has properties is not bankrupt
        let player = Player {
            name: String::from("Property Rich Player"),
            cash: 0,
            position: 0,
            jail_turns: 0,
            properties: vec![1],
            get_out_of_jail_cards: 0,
        };
        assert!(!is_player_bankrupt(&player));

        // Test case 4: Player with no cash and no properties is bankrupt
        let player = Player {
            name: String::from("Bankrupt Player"),
            cash: 0,
            position: 0,
            jail_turns: 0,
            properties: vec![],
            get_out_of_jail_cards: 0,
        };
        assert!(is_player_bankrupt(&player));
    }

    #[test]
    fn test_process_move_player() {
        // Create test accounts
        let program_id = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();

        let mut game_lamports = 0;
        let mut player_lamports = 0;

        let mut game_data = vec![0; 8192];
        let mut player_data = vec![0; 32];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
            0,
        );

        let player_account = AccountInfo::new(
            &player_key,
            true,  // Player must be signer
            true,
            &mut player_lamports,
            &mut player_data,
            &program_id,
            false,
            0,
        );

        // Initialize game state
        let mut game = Game {
            board: create_board(),
            players: vec![
                Player {
                    name: String::from("Player 1"),
                    cash: 1500,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
                Player {
                    name: String::from("Player 2"),
                    cash: 1500,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
            ],
            current_player: 0,
            free_parking: 0,
            initialized: true,
        };

        let game_state = GameAccount {
            is_initialized: true,
            game: game.clone(),
        };
        game_state.pack_into_slice(&mut game_data);

        // Test case 1: Normal movement
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            let dice_roll = 6;
            process_move_player(&program_id, accounts, 6).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.players[0].position, 6);
            assert_eq!(updated_game.game.players[0].cash, 1500); // No change in cash
        }

        // Test case 2: Passing GO (collect $200)
        {
            game.players[0].position = 39; // Last space before GO
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            process_move_player(&program_id, accounts, 3).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.players[0].position, 2); // Wrapped around to position 2
            assert_eq!(updated_game.game.players[0].cash, 1700); // Collected $200 for passing GO
        }

        // Test case 3: Landing on Income Tax
        {
            game.players[0].position = 0;
            game.players[0].cash = 1500;
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let dice_roll = 4; // Land on Income Tax
            process_move_player(&program_id, accounts, dice_roll).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.players[0].position, 4);
            assert_eq!(updated_game.game.players[0].cash, 1300); // Paid $200 income tax
            assert_eq!(updated_game.game.free_parking, 200); // Tax goes to Free Parking
        }

        // Test case 4: Landing on Luxury Tax
        {
            game.players[0].position = 36;
            game.players[0].cash = 1500;
            game.free_parking = 0;
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let dice_roll = 2; // Land on Luxury Tax
            process_move_player(&program_id, accounts, dice_roll).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.players[0].position, 38);
            assert_eq!(updated_game.game.players[0].cash, 1425); // Paid $75 luxury tax
            assert_eq!(updated_game.game.free_parking, 75); // Tax goes to Free Parking
        }

        // Test case 5: Missing player signature
        {
            let mut unsigned_player_account = player_account.clone();
            unsigned_player_account.is_signer = false;

            let accounts = &[game_account.clone(), unsigned_player_account];
            let dice_roll = 6;
            let result = process_move_player(&program_id, accounts, dice_roll);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::MissingRequiredSignature);
        }

        // Test case 6: Invalid dice roll (too high)
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            let dice_roll = 13; // Max should be 12 (double sixes)
            let result = process_move_player(&program_id, accounts, dice_roll);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InvalidDiceRoll.into());
        }

        // Test case 7: Invalid dice roll (zero or negative)
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            let dice_roll = 0;
            let result = process_move_player(&program_id, accounts, dice_roll);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InvalidDiceRoll.into());
        }
    }

    #[test]
    fn test_process_buy_property() {
        // Create test accounts
        let program_id = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();

        let mut game_lamports = 0;
        let mut player_lamports = 0;

        let mut game_data = vec![0; 8192];
        let mut player_data = vec![0; 32];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
            0,
        );

        let player_account = AccountInfo::new(
            &player_key,
            true,  // Player must be signer
            true,
            &mut player_lamports,
            &mut player_data,
            &program_id,
            false,
            0,
        );

        // Initialize game state with a player on a property space
        let mut game = Game {
            board: create_board(),
            players: vec![
                Player {
                    name: String::from("Player 1"),
                    cash: 1500,
                    position: 1, // Mediterranean Avenue
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
                Player {
                    name: String::from("Player 2"),
                    cash: 1500,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
            ],
            current_player: 0,
            free_parking: 0,
            initialized: true,
        };

        let game_state = GameAccount {
            is_initialized: true,
            game: game.clone(),
        };
        game_state.pack_into_slice(&mut game_data);

        // Test case 1: Successful property purchase
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            process_buy_property(&program_id, accounts, 1).unwrap(); // Mediterranean Avenue

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.players[0].cash, 1440); // 1500 - 60 (Mediterranean Ave cost)
            assert_eq!(updated_game.game.players[0].properties.len(), 1);
            assert_eq!(updated_game.game.players[0].properties[0], 1); // Mediterranean Ave index
        }

        // Test case 2: Insufficient funds
        {
            game.players[0].cash = 50; // Less than property cost
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let result = process_buy_property(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InsufficientFunds.into());
        }

        // Test case 3: Property already owned
        {
            game.players[0].cash = 1500;
            game.players[0].properties = vec![1]; // Already owns Mediterranean Ave
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let result = process_buy_property(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::PropertyAlreadyOwned.into());
        }

        // Test case 4: Not on a property space
        {
            game.players[0].position = 0; // GO space
            game.players[0].properties = vec![];
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let result = process_buy_property(&program_id, accounts, 0);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::NotAProperty.into());
        }

        // Test case 5: Missing player signature
        {
            game.players[0].position = 1; // Back to Mediterranean Ave
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);


            let mut unsigned_player_account = player_account.clone();
            unsigned_player_account.is_signer = false;

            let accounts = &[game_account.clone(), unsigned_player_account];
            let result = process_buy_property(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::MissingRequiredSignature);
        }

        // Test case 6: Not current player's turn
        {
            game.current_player = 1; // Switch to player 2's turn
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let result = process_buy_property(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::NotPlayerTurn.into());
        }
    }

    #[test]
    fn test_process_build_house() {
        // Create test accounts
        let program_id = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();

        let mut game_lamports = 0;
        let mut player_lamports = 0;

        let mut game_data = vec![0; 8192];
        let mut player_data = vec![0; 32];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
            0,
        );

        let player_account = AccountInfo::new(
            &player_key,
            true,  // Player must be signer
            true,
            &mut player_lamports,
            &mut player_data,
            &program_id,
            false,
            0,
        );

        // Initialize game state with a player owning a complete color set
        let mut game = Game {
            board: create_board(),
            players: vec![
                Player {
                    name: String::from("Player 1"),
                    cash: 1500,
                    position: 1,
                    jail_turns: 0,
                    properties: vec![1, 3], // Mediterranean and Baltic (complete brown set)
                    get_out_of_jail_cards: 0,
                },
                Player {
                    name: String::from("Player 2"),
                    cash: 1500,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
            ],
            current_player: 0,
            free_parking: 0,
            initialized: true,
        };

        let game_state = GameAccount {
            is_initialized: true,
            game: game.clone(),
        };
        game_state.pack_into_slice(&mut game_data);

        // Test case 1: Successful house build
        {
            let accounts = &[game_account.clone(), player_account.clone()];
            let property_index = 1; // Mediterranean Avenue
            process_build_house(&program_id, accounts, property_index).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.players[0].cash, 1450); // 1500 - 50 (house cost)
            assert_eq!(updated_game.game.board[property_index].houses, 1);
        }

        // Test case 2: Insufficient funds
        {
            game.players[0].cash = 40; // Less than house cost (50)
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let property_index = 1;
            let result = process_build_house(&program_id, accounts, property_index);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InsufficientFunds.into());
        }

        // Test case 3: Property not owned by player
        {
            game.players[0].cash = 1500;
            game.players[0].properties = vec![1]; // Only owns Mediterranean
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let property_index = 3; // Baltic Avenue (not owned)
            let result = process_build_house(&program_id, accounts, property_index);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::PropertyNotOwned.into());
        }

        // Test case 4: Incomplete color set
        {
            game.players[0].properties = vec![1]; // Only owns Mediterranean
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let property_index = 1;
            let result = process_build_house(&program_id, accounts, property_index);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::IncompleteColorSet.into());
        }

        // Test case 5: Maximum houses reached
        {
            game.players[0].properties = vec![1, 3]; // Complete brown set
            if let TileType::Property(ref mut prop) = game.board[1] {
                prop.houses = 5; // Max houses
            }
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let property_index = 1;
            let result = process_build_house(&program_id, accounts, property_index);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::MaxHousesReached.into());
        }

        // Test case 6: Missing player signature
        {
            if let TileType::Property(ref mut prop) = game.board[1] {
                prop.houses = 0;
            }
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let mut unsigned_player_account = player_account.clone();
            unsigned_player_account.is_signer = false;

            let accounts = &[game_account.clone(), unsigned_player_account];
            let property_index = 1;
            let result = process_build_house(&program_id, accounts, property_index);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::MissingRequiredSignature);
        }

        // Test case 7: Not current player's turn
        {
            game.current_player = 1; // Switch to player 2's turn
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let property_index = 1;
            let result = process_build_house(&program_id, accounts, property_index);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::NotPlayerTurn.into());
        }

        // Test case 8: Invalid property index
        {
            game.current_player = 0;
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let property_index = 40; // Invalid index
            let result = process_build_house(&program_id, accounts, property_index);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InvalidPropertyIndex.into());
        }
    }

    #[test]
    fn test_process_pay_rent() {
        // Create test accounts
        let program_id = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();
        let owner_key = Pubkey::new_unique();

        let mut game_lamports = 0;
        let mut player_lamports = 0;
        let mut owner_lamports = 0;

        let mut game_data = vec![0; 8192];
        let mut player_data = vec![0; 32];
        let mut owner_data = vec![0; 32];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
            0,
        );

        let player_account = AccountInfo::new(
            &player_key,
            true,  // Player must be signer
            true,
            &mut player_lamports,
            &mut player_data,
            &program_id,
            false,
            0,
        );

        let owner_account = AccountInfo::new(
            &owner_key,
            false,
            true,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
            false,
            0,
        );

        // Initialize game state with property owned by player 2
        let mut game = Game {
            board: create_board(),
            players: vec![
                Player {
                    name: String::from("Player 1"),
                    cash: 1500,
                    position: 1, // Mediterranean Avenue
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
                Player {
                    name: String::from("Player 2"),
                    cash: 1500,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![1], // Owns Mediterranean Avenue
                    get_out_of_jail_cards: 0,
                },
            ],
            current_player: 0,
            free_parking: 0,
            initialized: true,
        };

        let game_state = GameAccount {
            is_initialized: true,
            game: game.clone(),
        };
        game_state.pack_into_slice(&mut game_data);

        // Test case 1: Successful rent payment
        {
            let accounts = &[game_account.clone(), player_account.clone(), owner_account.clone()];
            process_pay_rent(&program_id, accounts, 1).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.players[0].cash, 1498); // 1500 - 2 (base rent)
            assert_eq!(updated_game.game.players[1].cash, 1502); // 1500 + 2 (received rent)
        }

        // Test case 2: Property with houses (increased rent)
        {
            if let TileType::Property(ref mut prop) = game.board[1] {
                prop.houses = 3; // Add 3 houses to increase rent
            }
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone(), owner_account.clone()];
            process_pay_rent(&program_id, accounts, 1).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.players[0].cash, 1448); // 1500 - 50 (rent with 3 houses)
            assert_eq!(updated_game.game.players[1].cash, 1550); // 1500 + 50 (received rent)
        }

        // Test case 3: Insufficient funds
        {
            game.players[0].cash = 40; // Less than rent amount
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone(), owner_account.clone()];
            let result = process_pay_rent(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InsufficientFunds.into());
        }

        // Test case 4: Property not owned
        {
            game.players[1].properties = vec![]; // Remove property ownership
            game.players[0].cash = 1500; // Reset cash
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone(), owner_account.clone()];
            let result = process_pay_rent(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::PropertyNotOwned.into());
        }

        // Test case 5: Player owns the property (no rent due)
        {
            game.players[0].properties = vec![1]; // Player owns the property
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);


            let accounts = &[game_account.clone(), player_account.clone(), owner_account.clone()];
            let result = process_pay_rent(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::OwnProperty.into());
        }

        // Test case 6: Missing player signature
        {
            game.players[0].properties = vec![]; // Remove property ownership
            game.players[1].properties = vec![1]; // Owner has property
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let mut unsigned_player_account = player_account.clone();
            unsigned_player_account.is_signer = false;

            let accounts = &[game_account.clone(), unsigned_player_account, owner_account.clone()];
            let result = process_pay_rent(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::MissingRequiredSignature);
        }

        // Test case 7: Not current player's turn
        {
            game.current_player = 1; // Switch to player 2's turn
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone(), owner_account.clone()];
            let result = process_pay_rent(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::NotPlayerTurn.into());
        }

        // Test case 8: Not on a property space
        {
            game.current_player = 0;
            game.players[0].position = 0; // GO space
            let game_state = GameAccount {
                is_initialized: true,
                game: game.clone(),
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone(), owner_account.clone()];
            let result = process_pay_rent(&program_id, accounts, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::NotAProperty.into());
        }
    }

    #[test]
    fn test_process_next_turn() {
        // Create test accounts
        let program_id = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();

        let mut game_lamports = 0;
        let mut player_lamports = 0;

        let mut game_data = vec![0; 8192];
        let mut player_data = vec![0; 32];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
            0,
        );

        let player_account = AccountInfo::new(
            &player_key,
            true,  // Player must be signer
            true,
            &mut player_lamports,
            &mut player_data,
            &program_id,
            false,
            0,
        );

        // Test case 1: Normal turn progression with no bankrupt players
        {
            let game = Game {
                board: create_board(),
                players: vec![
                    Player {
                        name: String::from("Player 1"),
                        cash: 1500,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                    Player {
                        name: String::from("Player 2"),
                        cash: 1500,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                ],
                current_player: 0,
                free_parking: 0,
                initialized: true,
            };

            let game_state = GameAccount {
                is_initialized: true,
                game,
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            process_next_turn(&program_id, accounts).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.current_player, 1, "Turn should advance to next player");
        }

        // Test case 2: Skip bankrupt players
        {
            let game = Game {
                board: create_board(),
                players: vec![
                    Player {
                        name: String::from("Player 1"),
                        cash: 1500,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                    Player {  // Bankrupt player
                        name: String::from("Player 2"),
                        cash: 0,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                    Player {
                        name: String::from("Player 3"),
                        cash: 1500,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                ],
                current_player: 0,
                free_parking: 0,
                initialized: true,
            };

            let game_state = GameAccount {
                is_initialized: true,
                game,
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            process_next_turn(&program_id, accounts).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.current_player, 2, "Turn should skip bankrupt player");
        }

        // Test case 3: Wrap around to first non-bankrupt player
        {
            let game = Game {
                board: create_board(),
                players: vec![
                    Player {
                        name: String::from("Player 1"),
                        cash: 1500,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                    Player {  // Bankrupt player
                        name: String::from("Player 2"),
                        cash: 0,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                ],
                current_player: 0,
                free_parking: 0,
                initialized: true,
            };

            let game_state = GameAccount {
                is_initialized: true,
                game,
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            process_next_turn(&program_id, accounts).unwrap();

            let updated_game = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert_eq!(updated_game.game.current_player, 0, "Turn should wrap around to first non-bankrupt player");
        }

        // Test case 4: Missing player signature
        {
            let mut unsigned_player_account = player_account.clone();
            unsigned_player_account.is_signer = false;

            let accounts = &[game_account.clone(), unsigned_player_account];
            let result = process_next_turn(&program_id, accounts);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::MissingRequiredSignature);
        }
    }

    #[test]
    fn test_process_check_winner() {
        // Create test accounts
        let program_id = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();

        let mut game_lamports = 0;
        let mut player_lamports = 0;

        let mut game_data = vec![0; 8192];
        let mut player_data = vec![0; 32];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
            0,
        );

        let player_account = AccountInfo::new(
            &player_key,
            true,  // Player must be signer
            true,
            &mut player_lamports,
            &mut player_data,
            &program_id,
            false,
            0,
        );

        // Test case 1: No winner yet (multiple active players)
        {
            let game = Game {
                board: create_board(),
                players: vec![
                    Player {
                        name: String::from("Player 1"),
                        cash: 1500,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                    Player {
                        name: String::from("Player 2"),
                        cash: 1500,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                ],
                current_player: 0,
                free_parking: 0,
                initialized: true,
            };

            let game_state = GameAccount {
                is_initialized: true,
                game,
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let result = process_check_winner(&program_id, accounts).unwrap();
            assert!(result.is_none(), "Should not have a winner yet");
        }

        // Test case 2: One winner (all other players bankrupt)
        {
            let game = Game {
                board: create_board(),
                players: vec![
                    Player {
                        name: String::from("Winner"),
                        cash: 1500,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                    Player {  // Bankrupt player
                        name: String::from("Loser 1"),
                        cash: 0,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                    Player {  // Bankrupt player
                        name: String::from("Loser 2"),
                        cash: 0,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                ],
                current_player: 0,
                free_parking: 0,
                initialized: true,
            };

            let game_state = GameAccount {
                is_initialized: true,
                game,
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let result = process_check_winner(&program_id, accounts).unwrap();
            assert!(result.is_some(), "Should have a winner");
            assert_eq!(result.unwrap(), 0, "Player 0 should be the winner");
        }

        // Test case 3: Missing player signature
        {
            let mut unsigned_player_account = player_account.clone();
            unsigned_player_account.is_signer = false;

            let accounts = &[game_account.clone(), unsigned_player_account];
            let result = process_check_winner(&program_id, accounts);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::MissingRequiredSignature);
        }

        // Test case 4: All players bankrupt (should never happen in normal gameplay)
        {
            let game = Game {
                board: create_board(),
                players: vec![
                    Player {
                        name: String::from("Bankrupt 1"),
                        cash: 0,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                    Player {
                        name: String::from("Bankrupt 2"),
                        cash: 0,
                        position: 0,
                        jail_turns: 0,
                        properties: vec![],
                        get_out_of_jail_cards: 0,
                    },
                ],
                current_player: 0,
                free_parking: 0,
                initialized: true,
            };

            let game_state = GameAccount {
                is_initialized: true,
                game,
            };
            game_state.pack_into_slice(&mut game_data);

            let accounts = &[game_account.clone(), player_account.clone()];
            let result = process_check_winner(&program_id, accounts).unwrap();
            assert!(result.is_none(), "Should not have a winner when all players are bankrupt");
        }
    }

    #[test]
    fn test_process_draw_card() {
        use solana_program::clock::Clock;
        use solana_program::account_info::IntoAccountInfo;
        use solana_program::sysvar::clock;
        use std::cell::RefCell;

        // Create test accounts
        let program_id = Pubkey::new_unique();
        let game_key = Pubkey::new_unique();
        let player_key = Pubkey::new_unique();
        let card_deck_key = Pubkey::new_unique();
        let clock_key = clock::ID;

        let mut game_lamports = 0;
        let mut player_lamports = 0;
        let mut card_deck_lamports = 0;
        let mut clock_lamports = 0;

        let mut game_data = vec![0; 8192];
        let mut player_data = vec![0; 32];
        let mut card_deck_data = vec![0; 1024];
        let mut clock_data = vec![0; std::mem::size_of::<Clock>()];

        let game_account = AccountInfo::new(
            &game_key,
            false,
            true,
            &mut game_lamports,
            &mut game_data,
            &program_id,
            false,
            0,
        );

        let player_account = AccountInfo::new(
            &player_key,
            true,  // Player must be signer
            true,
            &mut player_lamports,
            &mut player_data,
            &program_id,
            false,
            0,
        );

        let card_deck_account = AccountInfo::new(
            &card_deck_key,
            false,
            true,
            &mut card_deck_lamports,
            &mut card_deck_data,
            &program_id,
            false,
            0,
        );

        let clock_account = AccountInfo::new(
            &clock_key,
            false,
            true,
            &mut clock_lamports,
            &mut clock_data,
            &program_id,
            false,
            0,
        );

        // Initialize game state
        let mut game = Game {
            board: create_board(),
            players: vec![Player {
                name: String::from("Test Player"),
                cash: 1500,
                position: 0,
                jail_turns: 0,
                properties: vec![],
                get_out_of_jail_cards: 0,
            }],
            current_player: 0,
            free_parking: 0,
            initialized: true,
        };

        let game_state = GameAccount {
            is_initialized: true,
            game,
        };
        game_state.pack_into_slice(&mut game_data);

        // Test Community Chest card draw
        {
            let accounts = &[
                game_account.clone(),
                player_account.clone(),
                card_deck_account.clone(),
                clock_account.clone(),
            ];

            // Set clock data to control card selection
            let mut clock = Clock::default();
            clock.slot = 0; // Will draw first card
            let clock_bytes = bincode::serialize(&clock).unwrap();
            clock_data.copy_from_slice(&clock_bytes);

            process_draw_card(&program_id, accounts, DeckType::CommunityChest).unwrap();

            // Verify card was drawn and effects applied
            let game_data = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            assert!(game_data.game.players[0].cash != 1500, "Card effect not applied");
        }

        // Test Chance card draw
        {
            let accounts = &[
                game_account.clone(),
                player_account.clone(),
                card_deck_account.clone(),
                clock_account.clone(),
            ];

            // Set clock data for different card
            let mut clock = Clock::default();
            clock.slot = 1; // Will draw second card
            let clock_bytes = bincode::serialize(&clock).unwrap();
            clock_data.copy_from_slice(&clock_bytes);

            process_draw_card(&program_id, accounts, DeckType::Chance).unwrap();

            // Verify card was drawn and effects applied
            let game_data = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            let player = &game_data.game.players[0];
            assert!(player.cash != 1500 || player.position != 0 || player.get_out_of_jail_cards != 0,
                "Card effect not applied");
        }

        // Test error case - missing player signature
        {
            let mut unsigned_player_account = player_account.clone();
            unsigned_player_account.is_signer = false;

            let accounts = &[
                game_account.clone(),
                unsigned_player_account,
                card_deck_account.clone(),
                clock_account.clone(),
            ];


            let result = process_draw_card(&program_id, accounts, DeckType::CommunityChest);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), ProgramError::MissingRequiredSignature);
        }

        // Test error case - insufficient balance for PayMoney card
        {
            // Set player's cash to 0
            let mut game_data = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            game_data.game.players[0].cash = 0;
            game_data.pack_into_slice(&mut game_account.data.borrow_mut());

            // Set clock to draw PayMoney card
            let mut clock = Clock::default();
            clock.slot = 2; // Assume this draws a PayMoney card
            let clock_bytes = bincode::serialize(&clock).unwrap();
            clock_data.copy_from_slice(&clock_bytes);

            let accounts = &[
                game_account.clone(),
                player_account.clone(),
                card_deck_account.clone(),
                clock_account.clone(),
            ];

            let result = process_draw_card(&program_id, accounts, DeckType::CommunityChest);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MonopolyError::InsufficientBalance.into());
        }

        // Test card rotation
        {
            let accounts = &[
                game_account.clone(),
                player_account.clone(),
                card_deck_account.clone(),
                clock_account.clone(),
            ];

            // Draw same card twice
            let mut clock = Clock::default();
            clock.slot = 0;
            let clock_bytes = bincode::serialize(&clock).unwrap();
            clock_data.copy_from_slice(&clock_bytes);

            process_draw_card(&program_id, accounts, DeckType::CommunityChest).unwrap();
            
            // Second draw should get different card due to rotation
            let first_effect = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();
            process_draw_card(&program_id, accounts, DeckType::CommunityChest).unwrap();
            let second_effect = GameAccount::unpack_from_slice(&game_account.data.borrow()).unwrap();

            assert!(first_effect.game.players[0].cash != second_effect.game.players[0].cash,
                "Card was not rotated properly");
        }
    }
}

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

fn process_move_player(_program_id: &Pubkey, accounts: &[AccountInfo], dice_roll: u8) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let game_account = next_account_info(accounts_iter)?;
    let player_account = next_account_info(accounts_iter)?;
    let clock_sysvar = next_account_info(accounts_iter)?;

    if dice_roll < 2 || dice_roll > 12 {
        return Err(MonopolyError::InvalidDiceRoll.into());
    }

    if !player_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut game_data = validate_game_account(game_account)?;
    let current_player = game_data.game.current_player as usize;
    let player = &mut game_data.game.players[current_player];

    // Use provided dice roll
    let total_move = dice_roll;

    // Handle jail logic
    if player.jail_turns > 0 {
        // In jail, can't move unless paying fine
        player.jail_turns += 1;
        if player.jail_turns >= 3 {
            player.cash = player.cash.saturating_sub(50);
            player.jail_turns = 0;
        } else {
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
    let new_position_usize = new_position as usize;
    match &game_data.game.board[new_position_usize] {
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

    let property_index_usize = property_index as usize;
    match &mut game_data.game.board[property_index_usize] {
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

    let property_index_usize = property_index as usize;
    match &mut game_data.game.board[property_index_usize] {
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

    let property_index_usize = property_index as usize;
    match &game_data.game.board[property_index_usize] {
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
