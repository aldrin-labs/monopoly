use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
// Pubkey already imported above

use crate::state::{Game, Player, Property, Card};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_account_pack() {
        use crate::board::create_board;

        // Create a test game state
        let mut game = Game {
            board: create_board(),
            players: vec![
                Player {
                    name: String::from("Player 1"),
                    cash: 1500,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![1, 3],
                    get_out_of_jail_cards: 1,
                },
                Player {
                    name: String::from("Player 2"),
                    cash: 2000,
                    position: 5,
                    jail_turns: 2,
                    properties: vec![6, 8, 9],
                    get_out_of_jail_cards: 0,
                },
            ],
            current_player: 1,
            free_parking: 500,
            initialized: true,
        };

        // Create a GameAccount
        let game_account = GameAccount {
            is_initialized: true,
            game: game.clone(),
        };

        // Test case 1: Successful packing and unpacking
        {
            let mut buffer = vec![0u8; GameAccount::LEN];
            game_account.pack_into_slice(&mut buffer);

            // Verify unpacking
            let unpacked = GameAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.is_initialized, game_account.is_initialized);
            assert_eq!(unpacked.game.current_player, game_account.game.current_player);
            assert_eq!(unpacked.game.free_parking, game_account.game.free_parking);
            assert_eq!(unpacked.game.initialized, game_account.game.initialized);

            // Verify player data
            assert_eq!(unpacked.game.players.len(), game_account.game.players.len());
            for (i, player) in game_account.game.players.iter().enumerate() {
                assert_eq!(unpacked.game.players[i].name, player.name);
                assert_eq!(unpacked.game.players[i].cash, player.cash);
                assert_eq!(unpacked.game.players[i].position, player.position);
                assert_eq!(unpacked.game.players[i].jail_turns, player.jail_turns);
                assert_eq!(unpacked.game.players[i].properties, player.properties);
                assert_eq!(unpacked.game.players[i].get_out_of_jail_cards, player.get_out_of_jail_cards);
            }

            // Verify board data
            assert_eq!(unpacked.game.board.len(), game_account.game.board.len());
            for (i, space) in game_account.game.board.iter().enumerate() {
                match (&unpacked.game.board[i], space) {
                    (TileType::Property(up), TileType::Property(og)) => {
                        assert_eq!(up.name, og.name);
                        assert_eq!(up.color, og.color);
                        assert_eq!(up.cost, og.cost);
                        assert_eq!(up.rent, og.rent);
                        assert_eq!(up.house_cost, og.house_cost);
                        assert_eq!(up.hotel_cost, og.hotel_cost);
                        assert_eq!(up.house_rent, og.house_rent);
                        assert_eq!(up.owner, og.owner);
                        assert_eq!(up.houses, og.houses);
                    },
                    (TileType::Special(up), TileType::Special(og)) => {
                        assert_eq!(up, og);
                    },
                    _ => panic!("Board tile types don't match")
                }
            }
        }

        // Test case 2: Uninitialized account
        {
            let uninitialized_account = GameAccount {
                is_initialized: false,
                game: Game {
                    board: vec![],
                    players: vec![],
                    current_player: 0,
                    free_parking: 0,
                    initialized: false,
                },
            };

            let mut buffer = vec![0u8; GameAccount::LEN];
            uninitialized_account.pack_into_slice(&mut buffer);

            let unpacked = GameAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.is_initialized, false);
            assert_eq!(unpacked.game.players.len(), 0);
            assert_eq!(unpacked.game.initialized, false);
        }

        // Test case 3: Buffer too small
        {
            let mut small_buffer = vec![0u8; 10]; // Too small for game state
            let result = std::panic::catch_unwind(|| {
                game_account.pack_into_slice(&mut small_buffer);
            });
            assert!(result.is_err()); // Should panic on buffer too small
        }

        // Test case 4: Maximum values
        {
            let max_game = Game {
                board: vec![TileType::Property(Property {
                    name: String::from("Max Property"),
                    color: Color::DarkBlue,
                    cost: u64::MAX,
                    rent: vec![u64::MAX; 6],
                    house_cost: u64::MAX,
                    hotel_cost: u64::MAX,
                    house_rent: vec![u64::MAX; 5],
                    owner: None,
                    houses: u8::MAX,
                })],
                players: vec![Player {
                    name: String::from("Max Player"),
                    cash: u32::MAX,
                    position: u8::MAX,
                    jail_turns: u8::MAX,
                    properties: vec![u8::MAX],
                    get_out_of_jail_cards: u8::MAX,
                }],
                current_player: u8::MAX,
                free_parking: u32::MAX,
                initialized: true,
            };
            let max_account = GameAccount {
                is_initialized: true,
                game: max_game,
            };

            let mut buffer = vec![0u8; GameAccount::LEN];
            max_account.pack_into_slice(&mut buffer);

            let unpacked = GameAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.game.players[0].cash, u32::MAX);
            assert_eq!(unpacked.game.players[0].position, u8::MAX);
            assert_eq!(unpacked.game.board[0].price, u32::MAX);
            assert_eq!(unpacked.game.board[0].houses, u8::MAX);
            assert_eq!(unpacked.game.free_parking, u32::MAX);
        }
    }

    #[test]
    fn test_player_account_pack() {
        use solana_program::pubkey::Pubkey;

        // Test case 1: Successful packing and unpacking
        {
            let player_account = PlayerAccount {
                is_initialized: true,
                player: Player {
                    name: String::from("Test Player"),
                    cash: 1500,
                    position: 10,
                    jail_turns: 2,
                    properties: vec![1, 3, 5, 7],
                    get_out_of_jail_cards: 1,
                },
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; PlayerAccount::LEN];
            player_account.pack_into_slice(&mut buffer);

            let unpacked = PlayerAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.is_initialized, player_account.is_initialized);
            assert_eq!(unpacked.player.name, player_account.player.name);
            assert_eq!(unpacked.player.cash, player_account.player.cash);
            assert_eq!(unpacked.player.position, player_account.player.position);
            assert_eq!(unpacked.player.jail_turns, player_account.player.jail_turns);
            assert_eq!(unpacked.player.properties, player_account.player.properties);
            assert_eq!(unpacked.player.get_out_of_jail_cards, player_account.player.get_out_of_jail_cards);
            assert_eq!(unpacked.game, player_account.game);
        }

        // Test case 2: Uninitialized account
        {
            let uninitialized_account = PlayerAccount {
                is_initialized: false,
                player: Player {
                    name: String::from(""),
                    cash: 0,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; PlayerAccount::LEN];
            uninitialized_account.pack_into_slice(&mut buffer);

            let unpacked = PlayerAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.is_initialized, false);
            assert_eq!(unpacked.player.name, "");
            assert_eq!(unpacked.player.cash, 0);
            assert_eq!(unpacked.player.properties.len(), 0);
        }

        // Test case 3: Buffer too small
        {
            let player_account = PlayerAccount {
                is_initialized: true,
                player: Player {
                    name: String::from("Test Player"),
                    cash: 1500,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
                game: Pubkey::new_unique(),
            };

            let mut small_buffer = vec![0u8; 10]; // Too small
            let result = std::panic::catch_unwind(|| {
                player_account.pack_into_slice(&mut small_buffer);
            });
            assert!(result.is_err());
        }

        // Test case 4: Maximum values
        {
            let max_player_account = PlayerAccount {
                is_initialized: true,
                player: Player {
                    name: String::from("Maximum Player Name Test"),
                    cash: u32::MAX,
                    position: u8::MAX,
                    jail_turns: u8::MAX,
                    properties: (0..40).collect(), // Max properties
                    get_out_of_jail_cards: u8::MAX,
                },
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; PlayerAccount::LEN];
            max_player_account.pack_into_slice(&mut buffer);

            let unpacked = PlayerAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.player.cash, u32::MAX);
            assert_eq!(unpacked.player.position, u8::MAX);
            assert_eq!(unpacked.player.jail_turns, u8::MAX);
            assert_eq!(unpacked.player.get_out_of_jail_cards, u8::MAX);
            assert_eq!(unpacked.player.properties.len(), 40);
        }
    }

    #[test]
    fn test_property_account_pack() {
        use solana_program::pubkey::Pubkey;

        // Test case 1: Successful packing and unpacking
        {
            let property_account = PropertyAccount {
                is_initialized: true,
                property: Property {
                    name: String::from("Boardwalk"),
                    price: 400,
                    rent: 50,
                    house_cost: 200,
                    houses: 3,
                    owner: Some(Pubkey::new_unique()),
                    mortgaged: false,
                },
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; PropertyAccount::LEN];
            property_account.pack_into_slice(&mut buffer);

            let unpacked = PropertyAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.is_initialized, property_account.is_initialized);
            assert_eq!(unpacked.property.name, property_account.property.name);
            assert_eq!(unpacked.property.price, property_account.property.price);
            assert_eq!(unpacked.property.rent, property_account.property.rent);
            assert_eq!(unpacked.property.house_cost, property_account.property.house_cost);
            assert_eq!(unpacked.property.houses, property_account.property.houses);
            assert_eq!(unpacked.property.owner, property_account.property.owner);
            assert_eq!(unpacked.property.mortgaged, property_account.property.mortgaged);
            assert_eq!(unpacked.game, property_account.game);
        }

        // Test case 2: Uninitialized account
        {
            let uninitialized_account = PropertyAccount {
                is_initialized: false,
                property: Property {
                    name: String::from(""),
                    price: 0,
                    rent: 0,
                    house_cost: 0,
                    houses: 0,
                    owner: None,
                    mortgaged: false,
                },
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; PropertyAccount::LEN];
            uninitialized_account.pack_into_slice(&mut buffer);

            let unpacked = PropertyAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.is_initialized, false);
            assert_eq!(unpacked.property.name, "");
            assert_eq!(unpacked.property.price, 0);
            assert_eq!(unpacked.property.rent, 0);
            assert_eq!(unpacked.property.owner, None);
        }

        // Test case 3: Buffer too small
        {
            let property_account = PropertyAccount {
                is_initialized: true,
                property: Property {
                    name: String::from("Test Property"),
                    price: 200,
                    rent: 10,
                    house_cost: 100,
                    houses: 0,
                    owner: None,
                    mortgaged: false,
                },
                game: Pubkey::new_unique(),
            };

            let mut small_buffer = vec![0u8; 10]; // Too small
            let result = std::panic::catch_unwind(|| {
                property_account.pack_into_slice(&mut small_buffer);
            });
            assert!(result.is_err());
        }

        // Test case 4: Maximum values
        {
            let max_property_account = PropertyAccount {
                is_initialized: true,
                property: Property {
                    name: String::from("Maximum Property Name Test"),
                    price: u32::MAX,
                    rent: u32::MAX,
                    house_cost: u32::MAX,
                    houses: u8::MAX,
                    owner: Some(Pubkey::new_unique()),
                    mortgaged: true,
                },
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; PropertyAccount::LEN];
            max_property_account.pack_into_slice(&mut buffer);

            let unpacked = PropertyAccount::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.property.price, u32::MAX);
            assert_eq!(unpacked.property.rent, u32::MAX);
            assert_eq!(unpacked.property.house_cost, u32::MAX);
            assert_eq!(unpacked.property.houses, u8::MAX);
            assert_eq!(unpacked.property.mortgaged, true);
            assert!(unpacked.property.owner.is_some());
        }
    }

    #[test]
    fn test_card_deck_pack() {
        use solana_program::pubkey::Pubkey;

        // Test case 1: Successful packing and unpacking
        {
            let card_deck = CardDeck {
                is_initialized: true,
                deck_type: DeckType::Chance,
                cards: vec![
                    Card {
                        description: String::from("Advance to GO"),
                        action: String::from("MOVE_TO:0"),
                    },
                    Card {
                        description: String::from("Go to Jail"),
                        action: String::from("GO_TO_JAIL"),
                    },
                ],
                current_card_index: 1,
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; CardDeck::LEN];
            card_deck.pack_into_slice(&mut buffer);

            let unpacked = CardDeck::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.is_initialized, card_deck.is_initialized);
            assert_eq!(unpacked.deck_type, card_deck.deck_type);
            assert_eq!(unpacked.cards.len(), card_deck.cards.len());
            for (unpacked_card, original_card) in unpacked.cards.iter().zip(card_deck.cards.iter()) {
                assert_eq!(unpacked_card.description, original_card.description);
                assert_eq!(unpacked_card.action, original_card.action);
            }
            assert_eq!(unpacked.current_card_index, card_deck.current_card_index);
            assert_eq!(unpacked.game, card_deck.game);
        }

        // Test case 2: Uninitialized account
        {
            let uninitialized_deck = CardDeck {
                is_initialized: false,
                deck_type: DeckType::CommunityChest,
                cards: vec![],
                current_card_index: 0,
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; CardDeck::LEN];
            uninitialized_deck.pack_into_slice(&mut buffer);

            let unpacked = CardDeck::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.is_initialized, false);
            assert_eq!(unpacked.cards.len(), 0);
            assert_eq!(unpacked.current_card_index, 0);
        }

        // Test case 3: Buffer too small
        {
            let card_deck = CardDeck {
                is_initialized: true,
                deck_type: DeckType::Chance,
                cards: vec![Card {
                    description: String::from("Test Card"),
                    action: String::from("TEST_ACTION"),
                }],
                current_card_index: 0,
                game: Pubkey::new_unique(),
            };

            let mut small_buffer = vec![0u8; 10]; // Too small
            let result = std::panic::catch_unwind(|| {
                card_deck.pack_into_slice(&mut small_buffer);
            });
            assert!(result.is_err());
        }

        // Test case 4: Maximum values
        {
            let max_cards: Vec<Card> = (0..16).map(|i| Card {
                description: format!("Max Card {}", i),
                action: format!("ACTION_{}", i),
            }).collect();

            let max_card_deck = CardDeck {
                is_initialized: true,
                deck_type: DeckType::Chance,
                cards: max_cards,
                current_card_index: u8::MAX,
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; CardDeck::LEN];
            max_card_deck.pack_into_slice(&mut buffer);

            let unpacked = CardDeck::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.cards.len(), 16);
            assert_eq!(unpacked.current_card_index, u8::MAX);
        }

        // Test case 5: Different deck types
        {
            let chance_deck = CardDeck {
                is_initialized: true,
                deck_type: DeckType::Chance,
                cards: vec![],
                current_card_index: 0,
                game: Pubkey::new_unique(),
            };

            let mut buffer = vec![0u8; CardDeck::LEN];
            chance_deck.pack_into_slice(&mut buffer);
            let unpacked = CardDeck::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.deck_type, DeckType::Chance);

            let community_chest_deck = CardDeck {
                is_initialized: true,
                deck_type: DeckType::CommunityChest,
                cards: vec![],
                current_card_index: 0,
                game: Pubkey::new_unique(),
            };

            community_chest_deck.pack_into_slice(&mut buffer);
            let unpacked = CardDeck::unpack_from_slice(&buffer).unwrap();
            assert_eq!(unpacked.deck_type, DeckType::CommunityChest);
        }
    }

    #[test]
    fn test_account_validation() {
        // Test GameAccount initialization
        {
            let game_account = GameAccount {
                is_initialized: true,
                game: Game {
                    board: vec![],
                    players: vec![],
                    current_player: 0,
                    free_parking: 0,
                    initialized: true,
                },
            };
            assert!(game_account.is_initialized());

            let uninitialized_account = GameAccount {
                is_initialized: false,
                game: Game {
                    board: vec![],
                    players: vec![],
                    current_player: 0,
                    free_parking: 0,
                    initialized: false,
                },
            };
            assert!(!uninitialized_account.is_initialized());
        }

        // Test PlayerAccount initialization
        {
            let player_account = PlayerAccount {
                is_initialized: true,
                player: Player {
                    name: String::from("Test Player"),
                    cash: 1500,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
                game: Pubkey::new_unique(),
            };
            assert!(player_account.is_initialized());

            let uninitialized_player = PlayerAccount {
                is_initialized: false,
                player: Player {
                    name: String::from(""),
                    cash: 0,
                    position: 0,
                    jail_turns: 0,
                    properties: vec![],
                    get_out_of_jail_cards: 0,
                },
                game: Pubkey::new_unique(),
            };
            assert!(!uninitialized_player.is_initialized());
        }

        // Test PropertyAccount initialization
        {
            let property_account = PropertyAccount {
                is_initialized: true,
                property: Property {
                    name: String::from("Test Property"),
                    price: 200,
                    rent: 10,
                    house_cost: 100,
                    houses: 0,
                    owner: None,
                    mortgaged: false,
                },
                game: Pubkey::new_unique(),
            };
            assert!(property_account.is_initialized());

            let uninitialized_property = PropertyAccount {
                is_initialized: false,
                property: Property {
                    name: String::from(""),
                    price: 0,
                    rent: 0,
                    house_cost: 0,
                    houses: 0,
                    owner: None,
                    mortgaged: false,
                },
                game: Pubkey::new_unique(),
            };
            assert!(!uninitialized_property.is_initialized());
        }

        // Test CardDeck initialization
        {
            let card_deck = CardDeck {
                is_initialized: true,
                cards: vec![],
                game: Pubkey::new_unique(),
                deck_type: DeckType::Chance,
            };
            assert!(card_deck.is_initialized());

            let uninitialized_deck = CardDeck {
                is_initialized: false,
                cards: vec![],
                game: Pubkey::new_unique(),
                deck_type: DeckType::CommunityChest,
            };
            assert!(!uninitialized_deck.is_initialized());
        }

        // Test validation functions
        // Test GameAccount validation
        assert!(validate_game_account(&game_account).is_ok());
        assert!(validate_game_account(&uninitialized_account).is_err());

        // Test PlayerAccount validation
        assert!(validate_player_account(&player_account).is_ok());
        assert!(validate_player_account(&uninitialized_player).is_err());

        // Test PropertyAccount validation
        assert!(validate_property_account(&property_account).is_ok());
        assert!(validate_property_account(&uninitialized_property).is_err());

        // Test CardDeck validation
        assert!(validate_card_deck(&card_deck).is_ok());
        assert!(validate_card_deck(&uninitialized_deck).is_err());
    }
}

#[derive(Clone, Debug)]
pub struct GameAccount {
    pub is_initialized: bool,
    pub game: Game,
}

impl Pack for GameAccount {
    const LEN: usize = 1 + Game::LEN; // is_initialized + game

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut offset = 0;
        dst[offset] = self.is_initialized as u8;
        offset += 1;
        self.game.pack_into_slice(&mut dst[offset..]);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut offset = 0;
        let is_initialized = src[offset] != 0;
        offset += 1;
        let game = Game::unpack_from_slice(&src[offset..])?;
        Ok(GameAccount {
            is_initialized,
            game,
        })
    }
}

impl Sealed for GameAccount {}

impl IsInitialized for GameAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

// Removed duplicate Pack implementation

#[derive(Clone, Debug)]
pub struct PlayerAccount {
    pub is_initialized: bool,
    pub player: Player,
    pub game: Pubkey,
}

impl Sealed for PlayerAccount {}

impl IsInitialized for PlayerAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for PlayerAccount {
    const LEN: usize = 1000; // Calculate exact size based on max player state

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut current = 0;
        let is_initialized = src[current] != 0;
        current += 1;

        let player = Player::unpack_from_slice(&src[current..])?;
        current += Player::LEN;

        let game = Pubkey::from(<[u8; 32]>::try_from(&src[current..current + 32]).unwrap());

        Ok(PlayerAccount {
            is_initialized,
            player,
            game,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut current = 0;
        dst[current] = self.is_initialized as u8;
        current += 1;
        self.player.pack_into_slice(&mut dst[current..current + Player::LEN]);
        current += Player::LEN;
        dst[current..current + 32].copy_from_slice(&self.game.to_bytes());
    }
}

#[derive(Clone, Debug)]
pub struct PropertyAccount {
    pub is_initialized: bool,
    pub property: Property,
    pub game: Pubkey,
}

impl Sealed for PropertyAccount {}

impl IsInitialized for PropertyAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for PropertyAccount {
    const LEN: usize = 500; // Calculate exact size based on max property state

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut current = 0;
        let is_initialized = src[current] != 0;
        current += 1;

        let property = Property::unpack_from_slice(&src[current..])?;
        current += Property::LEN;

        let game = Pubkey::from(<[u8; 32]>::try_from(&src[current..current + 32]).unwrap());

        Ok(PropertyAccount {
            is_initialized,
            property,
            game,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut current = 0;
        dst[current] = self.is_initialized as u8;
        current += 1;
        self.property.pack_into_slice(&mut dst[current..current + Property::LEN]);
        current += Property::LEN;
        dst[current..current + 32].copy_from_slice(&self.game.to_bytes());
    }
}

#[derive(Clone, Debug)]
pub struct CardDeck {
    pub is_initialized: bool,
    pub cards: Vec<Card>,
    pub game: Pubkey,
    pub deck_type: DeckType,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DeckType {
    CommunityChest,
    Chance,
}

impl Sealed for CardDeck {}

impl IsInitialized for CardDeck {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for CardDeck {
    const LEN: usize = 1000; // Calculate exact size based on max deck state

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut current = 0;
        let is_initialized = src[current] != 0;
        current += 1;

        let cards_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;

        let data_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;

        let mut cards = Vec::new();
        let mut card_current = current;
        while card_current < current + data_len {
            let card_type = src[card_current];
            card_current += 1;
            
            let card = match card_type {
                0 => {
                    let value = u64::from_le_bytes(src[card_current..card_current + 8].try_into().unwrap());
                    card_current += 8;
                    Card::CollectMoney(value)
                },
                1 => {
                    let value = u64::from_le_bytes(src[card_current..card_current + 8].try_into().unwrap());
                    card_current += 8;
                    Card::PayMoney(value)
                },
                2 => {
                    let value = src[card_current];
                    card_current += 1;
                    Card::Move(value)
                },
                3 => {
                    Card::GetOutOfJail
                },
                _ => return Err(ProgramError::InvalidAccountData),
            };
            cards.push(card);
        }
        current += data_len;

        let game = Pubkey::from(<[u8; 32]>::try_from(&src[current..current + 32]).unwrap());
        current += 32;

        let deck_type = match src[current] {
            0 => DeckType::CommunityChest,
            1 => DeckType::Chance,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(CardDeck {
            is_initialized,
            cards,
            game,
            deck_type,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut current = 0;
        
        // Write initialization flag
        dst[current] = self.is_initialized as u8;
        current += 1;
        
        // Write cards length
        let cards_len = self.cards.len() as u32;
        dst[current..current + 4].copy_from_slice(&cards_len.to_le_bytes());
        current += 4;
        
        // Write cards data
        for card in &self.cards {
            match card {
                Card::CollectMoney(value) => {
                    dst[current] = 0;
                    current += 1;
                    dst[current..current + 8].copy_from_slice(&value.to_le_bytes());
                    current += 8;
                },
                Card::PayMoney(value) => {
                    dst[current] = 1;
                    current += 1;
                    dst[current..current + 8].copy_from_slice(&value.to_le_bytes());
                    current += 8;
                },
                Card::Move(value) => {
                    dst[current] = 2;
                    current += 1;
                    dst[current] = *value;
                    current += 1;
                },
                Card::GetOutOfJail => {
                    dst[current] = 3;
                    current += 1;
                },
            }
        }
        
        // Write game pubkey
        dst[current..current + 32].copy_from_slice(&self.game.to_bytes());
        current += 32;
        
        // Write deck type
        dst[current] = match self.deck_type {
            DeckType::CommunityChest => 0,
            DeckType::Chance => 1,
        };
    }
}

// Helper functions for account validation
pub fn validate_game_account(account: &AccountInfo) -> Result<GameAccount, ProgramError> {
    if account.owner != &crate::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    GameAccount::unpack(&account.data.borrow())
}

pub fn validate_player_account(account: &AccountInfo) -> Result<PlayerAccount, ProgramError> {
    if account.owner != &crate::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    PlayerAccount::unpack(&account.data.borrow())
}

pub fn validate_property_account(account: &AccountInfo) -> Result<PropertyAccount, ProgramError> {
    if account.owner != &crate::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    PropertyAccount::unpack(&account.data.borrow())
}

pub fn validate_card_deck(account: &AccountInfo) -> Result<CardDeck, ProgramError> {
    if account.owner != &crate::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    CardDeck::unpack(&account.data.borrow())
}
