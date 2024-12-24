use crate::state::{TileType, Property, SpecialTile, Color, Card};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_board() {
        // TODO: Implement test
    }

    #[test]
    fn test_create_community_chest() {
        // TODO: Implement test
    }

    #[test]
    fn test_create_chance_cards() {
        // TODO: Implement test
    }
}

pub fn create_board() -> Vec<TileType> {
    vec![
        // Square 0 - GO
        TileType::Special(SpecialTile::Go),
        
        // Purple Group (1-2)
        TileType::Property(Property {
            name: "Solana Genesis Block".to_string(),
            color: Color::Purple,
            cost: 60,
            rent: vec![2, 10, 30, 90, 160, 250],
            house_cost: 50,
            hotel_cost: 250,
            owner: None,
            houses: 0,
        }),
        TileType::Special(SpecialTile::CommunityChest),
        TileType::Property(Property {
            name: "Metaplex Protocol".to_string(),
            color: Color::Purple,
            cost: 60,
            rent: vec![4, 20, 60, 180, 320, 450],
            house_cost: 50,
            hotel_cost: 250,
            owner: None,
            houses: 0,
        }),
        
        // Tax Square (4)
        TileType::Special(SpecialTile::IncomeTax),
        
        // First Railroad (5)
        TileType::Property(Property {
            name: "Solana Network Rail".to_string(),
            color: Color::Purple,
            cost: 200,
            rent: vec![25, 50, 100, 200],
            house_cost: 0,
            hotel_cost: 0,
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
