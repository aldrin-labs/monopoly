use crate::state::{TileType, Property, SpecialTile, Color, Card};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_board() {
        let board = create_board();
        
        // Test board size
        assert_eq!(board.len(), 40, "Board should have 40 tiles");

        // Test GO tile (position 0)
        assert!(matches!(board[0], TileType::Special(SpecialTile::Go)));

        // Test Purple properties (positions 1-2)
        if let TileType::Property(prop) = &board[1] {
            assert_eq!(prop.name, "Solana Genesis Block");
            assert_eq!(prop.color, Color::Brown);
            assert_eq!(prop.cost, 60);
            assert_eq!(prop.rent, vec![2, 10, 30, 90, 160, 250]);
            assert_eq!(prop.house_cost, 50);
            assert_eq!(prop.hotel_cost, 250);
            assert!(prop.owner.is_none());
            assert_eq!(prop.houses, 0);
        } else {
            panic!("Expected Purple property at position 1");
        }

        // Test Community Chest (position 2)
        assert!(matches!(board[2], TileType::Special(SpecialTile::CommunityChest)));

        // Test Income Tax (position 4)
        assert!(matches!(board[4], TileType::Special(SpecialTile::IncomeTax)));

        // Test Railroad (position 5)
        if let TileType::Property(prop) = &board[5] {
            assert_eq!(prop.name, "Solana Network Rail");
            assert_eq!(prop.rent, vec![25, 50, 100, 200]);
            assert_eq!(prop.cost, 200);
        } else {
            panic!("Expected Railroad at position 5");
        }

        // Test Jail (position 10)
        assert!(matches!(board[10], TileType::Special(SpecialTile::Jail)));

        // Test Free Parking (position 20)
        assert!(matches!(board[20], TileType::Special(SpecialTile::FreeParking)));

        // Test Luxury Tax (position 38)
        assert!(matches!(board[38], TileType::Special(SpecialTile::LuxuryTax)));

        // Test most expensive property (Boardwalk equivalent - position 39)
        if let TileType::Property(prop) = &board[39] {
            assert_eq!(prop.name, "Solana Foundation");
            assert_eq!(prop.color, Color::DarkBlue);
            assert_eq!(prop.cost, 400);
            assert_eq!(prop.rent, vec![50, 200, 600, 1400, 1700, 2000]);
        } else {
            panic!("Expected Dark Blue property at position 39");
        }

        // Count property types
        let property_count = board.iter().filter(|tile| matches!(tile, TileType::Property(_))).count();
        let special_count = board.iter().filter(|tile| matches!(tile, TileType::Special(_))).count();
        
        assert_eq!(property_count, 28, "Should have 28 properties");
        assert_eq!(special_count, 12, "Should have 12 special tiles");

        // Verify all properties have no initial owner and no houses
        board.iter().filter_map(|tile| {
            if let TileType::Property(prop) = tile {
                Some(prop)
            } else {
                None
            }
        }).for_each(|prop| {
            assert!(prop.owner.is_none(), "Property should have no initial owner");
            assert_eq!(prop.houses, 0, "Property should have no initial houses");
        });
    }

    #[test]
    fn test_create_community_chest() {
        let cards = create_community_chest();
        
        // Test number of cards
        assert_eq!(cards.len(), 7, "Should have 7 community chest cards");

        // Test specific cards
        let mut collect_200 = false;
        let mut pay_50 = false;
        let mut get_out_of_jail = false;
        let mut move_to_go = false;
        let mut collect_100 = false;
        let mut pay_100 = false;
        let mut move_to_jail = false;

        for card in cards {
            match card {
                Card::CollectMoney(200) => collect_200 = true,
                Card::PayMoney(50) => pay_50 = true,
                Card::GetOutOfJail => get_out_of_jail = true,
                Card::Move(0) => move_to_go = true,
                Card::CollectMoney(100) => collect_100 = true,
                Card::PayMoney(100) => pay_100 = true,
                Card::Move(10) => move_to_jail = true,
                _ => panic!("Unexpected card in community chest"),
            }
        }

        assert!(collect_200, "Missing Collect $200 card");
        assert!(pay_50, "Missing Pay $50 card");
        assert!(get_out_of_jail, "Missing Get Out of Jail card");
        assert!(move_to_go, "Missing Move to GO card");
        assert!(collect_100, "Missing Collect $100 card");
        assert!(pay_100, "Missing Pay $100 card");
        assert!(move_to_jail, "Missing Move to Jail card");
    }

    #[test]
    fn test_create_chance_cards() {
        let cards = create_chance_cards();
        
        // Test number of cards
        assert_eq!(cards.len(), 6, "Should have 6 chance cards");

        // Test specific cards
        let mut collect_150 = false;
        let mut pay_15 = false;
        let mut move_to_go = false;
        let mut get_out_of_jail = false;
        let mut move_to_jail = false;
        let mut move_to_railroad = false;

        for card in cards {
            match card {
                Card::CollectMoney(150) => collect_150 = true,
                Card::PayMoney(15) => pay_15 = true,
                Card::Move(0) => move_to_go = true,
                Card::GetOutOfJail => get_out_of_jail = true,
                Card::Move(10) => move_to_jail = true,
                Card::Move(5) => move_to_railroad = true,
                _ => panic!("Unexpected card in chance deck"),
            }
        }

        assert!(collect_150, "Missing Collect $150 card");
        assert!(pay_15, "Missing Pay $15 card");
        assert!(move_to_go, "Missing Move to GO card");
        assert!(get_out_of_jail, "Missing Get Out of Jail card");
        assert!(move_to_jail, "Missing Move to Jail card");
        assert!(move_to_railroad, "Missing Move to Railroad card");
    }
}

pub fn create_board() -> Vec<TileType> {
    vec![
        // Square 0 - GO
        TileType::Special(SpecialTile::Go),
        
        // Purple Group (1-2)
        TileType::Property(Property {
            name: "Solana Genesis Block".to_string(),
            color: Color::Brown,
            cost: 60,
            rent: vec![2, 10, 30, 90, 160, 250],
            house_cost: 50,
            hotel_cost: 250,
            house_rent: vec![10, 30, 90, 160, 250],
            owner: None,
            houses: 0,
        }),
        TileType::Special(SpecialTile::CommunityChest),
        TileType::Property(Property {
            name: "Metaplex Protocol".to_string(),
            color: Color::Brown,
            cost: 60,
            rent: vec![4, 20, 60, 180, 320, 450],
            house_cost: 50,
            hotel_cost: 250,
            house_rent: vec![20, 60, 180, 320, 450],
            owner: None,
            houses: 0,
        }),
        
        // Tax Square (4)
        TileType::Special(SpecialTile::IncomeTax),
        
        // First Railroad (5)
        TileType::Property(Property {
            name: "Solana Network Rail".to_string(),
            color: Color::Brown,
            cost: 200,
            rent: vec![25, 50, 100, 200],
            house_cost: 0,
            hotel_cost: 0,
            house_rent: vec![50, 100, 200],
            owner: None,
            houses: 0,
        }),
        
        // Light Blue Group (6-8)
        TileType::Property(Property {
            name: "Serum DEX".to_string(),
            color: Color::LightBlue,
            cost: 100,
            rent: vec![6, 30, 90, 270, 400, 550],
            house_cost: 50,
            hotel_cost: 250,
            house_rent: vec![30, 90, 270, 400, 550],
            owner: None,
            houses: 0,
        }),
        TileType::Special(SpecialTile::Chance),
        TileType::Property(Property {
            name: "Raydium AMM".to_string(),
            color: Color::LightBlue,
            cost: 100,
            rent: vec![6, 30, 90, 270, 400, 550],
            house_cost: 50,
            hotel_cost: 250,
            house_rent: vec![30, 90, 270, 400, 550],
            owner: None,
            houses: 0,
        }),
        TileType::Property(Property {
            name: "Orca DEX".to_string(),
            color: Color::LightBlue,
            cost: 120,
            rent: vec![8, 40, 100, 300, 450, 600],
            house_cost: 50,
            hotel_cost: 250,
            house_rent: vec![40, 100, 300, 450, 600],
            owner: None,
            houses: 0,
        }),
        
        // Jail (10)
        TileType::Special(SpecialTile::Jail),
        
        // Pink Group (11-13)
        TileType::Property(Property {
            name: "Magic Eden".to_string(),
            color: Color::Pink,
            cost: 140,
            rent: vec![10, 50, 150, 450, 625, 750],
            house_cost: 100,
            hotel_cost: 500,
            house_rent: vec![50, 150, 450, 625, 750],
            owner: None,
            houses: 0,
        }),
        TileType::Property(Property {
            name: "Tensor".to_string(),
            color: Color::Pink,
            cost: 140,
            rent: vec![10, 50, 150, 450, 625, 750],
            house_cost: 100,
            hotel_cost: 500,
            house_rent: vec![50, 150, 450, 625, 750],
            owner: None,
            houses: 0,
        }),
        TileType::Property(Property {
            name: "Hyperspace".to_string(),
            color: Color::Pink,
            cost: 160,
            rent: vec![12, 60, 180, 500, 700, 900],
            house_cost: 100,
            hotel_cost: 500,
            owner: None,
            houses: 0,
        }),

        // Orange Group (14-16)
        TileType::Property(Property {
            name: "Marinade".to_string(),
            color: Color::Orange,
            cost: 180,
            rent: vec![14, 70, 200, 550, 750, 950],
            house_cost: 100,
            hotel_cost: 500,
            house_rent: vec![70, 200, 550, 750, 950],
            owner: None,
            houses: 0,
        }),
        TileType::Special(SpecialTile::CommunityChest),
        TileType::Property(Property {
            name: "Lido".to_string(),
            color: Color::Orange,
            cost: 180,
            rent: vec![14, 70, 200, 550, 750, 950],
            house_cost: 100,
            hotel_cost: 500,
            owner: None,
            houses: 0,
        }),
        TileType::Property(Property {
            name: "JPool".to_string(),
            color: Color::Orange,
            cost: 200,
            rent: vec![16, 80, 220, 600, 800, 1000],
            house_cost: 100,
            hotel_cost: 500,
            owner: None,
            houses: 0,
        }),

        // Free Parking (20)
        TileType::Special(SpecialTile::FreeParking),

        // Red Group (21-23)
        TileType::Property(Property {
            name: "Jupiter".to_string(),
            color: Color::Red,
            cost: 220,
            rent: vec![18, 90, 250, 700, 875, 1050],
            house_cost: 150,
            hotel_cost: 750,
            owner: None,
            houses: 0,
        }),
        TileType::Special(SpecialTile::Chance),
        TileType::Property(Property {
            name: "Orca".to_string(),
            color: Color::Red,
            cost: 220,
            rent: vec![18, 90, 250, 700, 875, 1050],
            house_cost: 150,
            hotel_cost: 750,
            owner: None,
            houses: 0,
        }),
        TileType::Property(Property {
            name: "Raydium".to_string(),
            color: Color::Red,
            cost: 240,
            rent: vec![20, 100, 300, 750, 925, 1100],
            house_cost: 150,
            hotel_cost: 750,
            house_rent: vec![100, 300, 750, 925, 1100],
            owner: None,
            houses: 0,
        }),

        // Yellow Group (24-26)
        TileType::Property(Property {
            name: "Pyth".to_string(),
            color: Color::Yellow,
            cost: 260,
            rent: vec![22, 110, 330, 800, 975, 1150],
            house_cost: 150,
            hotel_cost: 750,
            house_rent: vec![110, 330, 800, 975, 1150],
            owner: None,
            houses: 0,
        }),
        TileType::Property(Property {
            name: "Switchboard".to_string(),
            color: Color::Yellow,
            cost: 260,
            rent: vec![22, 110, 330, 800, 975, 1150],
            house_cost: 150,
            hotel_cost: 750,
            owner: None,
            houses: 0,
        }),
        TileType::Property(Property {
            name: "Chainlink".to_string(),
            color: Color::Yellow,
            cost: 280,
            rent: vec![24, 120, 360, 850, 1025, 1200],
            house_cost: 150,
            hotel_cost: 750,
            house_rent: vec![120, 360, 850, 1025, 1200],
            owner: None,
            houses: 0,
        }),

        // Go To Jail (30)
        TileType::Special(SpecialTile::GoToJail),

        // Green Group (31-33)
        TileType::Property(Property {
            name: "Metaplex".to_string(),
            color: Color::Green,
            cost: 300,
            rent: vec![26, 130, 390, 900, 1100, 1275],
            house_cost: 200,
            hotel_cost: 1000,
            house_rent: vec![130, 390, 900, 1100, 1275],
            owner: None,
            houses: 0,
        }),
        TileType::Property(Property {
            name: "Cardinal".to_string(),
            color: Color::Green,
            cost: 300,
            rent: vec![26, 130, 390, 900, 1100, 1275],
            house_cost: 200,
            hotel_cost: 1000,
            house_rent: vec![130, 390, 900, 1100, 1275],
            owner: None,
            houses: 0,
        }),
        TileType::Special(SpecialTile::CommunityChest),
        TileType::Property(Property {
            name: "Goki".to_string(),
            color: Color::Green,
            cost: 320,
            rent: vec![28, 150, 450, 1000, 1200, 1400],
            house_cost: 200,
            hotel_cost: 1000,
            house_rent: vec![150, 450, 1000, 1200, 1400],
            owner: None,
            houses: 0,
        }),

        // Dark Blue Group (37-39)
        TileType::Property(Property {
            name: "Solana Labs".to_string(),
            color: Color::DarkBlue,
            cost: 350,
            rent: vec![35, 175, 500, 1100, 1300, 1500],
            house_cost: 200,
            hotel_cost: 1000,
            house_rent: vec![175, 500, 1100, 1300, 1500],
            owner: None,
            houses: 0,
        }),
        TileType::Special(SpecialTile::LuxuryTax),
        TileType::Property(Property {
            name: "Solana Foundation".to_string(),
            color: Color::DarkBlue,
            cost: 400,
            rent: vec![50, 200, 600, 1400, 1700, 2000],
            house_cost: 200,
            hotel_cost: 1000,
            house_rent: vec![200, 600, 1400, 1700, 2000],
            owner: None,
            houses: 0,
        }),
    ]
}

pub fn create_community_chest() -> Vec<Card> {
    vec![
        Card::CollectMoney(200),
        Card::PayMoney(50),
        Card::GetOutOfJail,
        Card::Move(0), // GO
        Card::CollectMoney(100),
        Card::PayMoney(100),
        Card::Move(10), // Jail
    ]
}

pub fn create_chance_cards() -> Vec<Card> {
    vec![
        Card::CollectMoney(150),
        Card::PayMoney(15),
        Card::Move(0), // Advance to GO
        Card::GetOutOfJail,
        Card::Move(10), // Go to Jail
        Card::Move(5),  // Move to first Railroad
    ]
}
