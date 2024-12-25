// Removed unused import
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

impl Sealed for Color {}
impl Sealed for Property {}
impl Sealed for SpecialTile {}
impl Sealed for TileType {}
impl Sealed for Player {}
impl Sealed for Game {}
impl Sealed for Card {}

impl IsInitialized for Game {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl IsInitialized for Player {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl IsInitialized for Property {
    fn is_initialized(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Brown,
    LightBlue,
    Pink,
    Orange,
    Red,
    Yellow,
    Green,
    Blue,
    DarkBlue,
}

impl Pack for Color {
    const LEN: usize = 1;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = match self {
            Color::Brown => 0,
            Color::LightBlue => 1,
            Color::Pink => 2,
            Color::Orange => 3,
            Color::Red => 4,
            Color::Yellow => 5,
            Color::Green => 6,
            Color::Blue => 7,
            Color::DarkBlue => 8,
        };
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Ok(match src[0] {
            0 => Color::Brown,
            1 => Color::LightBlue,
            2 => Color::Pink,
            3 => Color::Orange,
            4 => Color::Red,
            5 => Color::Yellow,
            6 => Color::Green,
            7 => Color::Blue,
            8 => Color::DarkBlue,
            _ => return Err(ProgramError::InvalidAccountData),
        })
    }
}

impl Property {
    pub fn as_property(&self) -> Option<&Property> {
        Some(self)
    }
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub color: Color,
    pub cost: u64,
    pub rent: Vec<u64>,
    pub house_cost: u64,
    pub hotel_cost: u64,
    pub house_rent: Vec<u64>,
    pub owner: Option<Pubkey>,
    pub houses: u8,
}

impl Pack for Property {
    const LEN: usize = 1024; // Large enough for property data

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut current = 0;
        
        // Write name length and bytes
        let name_bytes = self.name.as_bytes();
        dst[current..current + 4].copy_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        current += 4;
        dst[current..current + name_bytes.len()].copy_from_slice(name_bytes);
        current += name_bytes.len();
        
        // Write color
        self.color.pack_into_slice(&mut dst[current..current + 1]);
        current += 1;
        
        // Write cost
        dst[current..current + 8].copy_from_slice(&self.cost.to_le_bytes());
        current += 8;
        
        // Write rent array
        dst[current..current + 4].copy_from_slice(&(self.rent.len() as u32).to_le_bytes());
        current += 4;
        for rent in &self.rent {
            dst[current..current + 8].copy_from_slice(&rent.to_le_bytes());
            current += 8;
        }
        
        // Write house and hotel costs
        dst[current..current + 8].copy_from_slice(&self.house_cost.to_le_bytes());
        current += 8;
        dst[current..current + 8].copy_from_slice(&self.hotel_cost.to_le_bytes());
        current += 8;

        // Write house_rent array
        dst[current..current + 4].copy_from_slice(&(self.house_rent.len() as u32).to_le_bytes());
        current += 4;
        for rent in &self.house_rent {
            dst[current..current + 8].copy_from_slice(&rent.to_le_bytes());
            current += 8;
        }
        
        // Write owner
        dst[current] = self.owner.is_some() as u8;
        current += 1;
        if let Some(owner) = &self.owner {
            dst[current..current + 32].copy_from_slice(&owner.to_bytes());
            current += 32;
        }
        
        // Write houses
        dst[current] = self.houses;
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut current = 0;
        
        let name_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;
        let name = String::from_utf8(src[current..current + name_len].to_vec())
            .map_err(|_| ProgramError::InvalidAccountData)?;
        current += name_len;
        
        let color = Color::unpack_from_slice(&src[current..current + 1])?;
        current += 1;
        
        let cost = u64::from_le_bytes(src[current..current + 8].try_into().unwrap());
        current += 8;
        
        let rent_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;
        let mut rent = Vec::with_capacity(rent_len);
        for _ in 0..rent_len {
            rent.push(u64::from_le_bytes(src[current..current + 8].try_into().unwrap()));
            current += 8;
        }
        
        let house_cost = u64::from_le_bytes(src[current..current + 8].try_into().unwrap());
        current += 8;
        let hotel_cost = u64::from_le_bytes(src[current..current + 8].try_into().unwrap());
        current += 8;

        // Read house_rent array
        let house_rent_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;
        let mut house_rent = Vec::with_capacity(house_rent_len);
        for _ in 0..house_rent_len {
            house_rent.push(u64::from_le_bytes(src[current..current + 8].try_into().unwrap()));
            current += 8;
        }
        
        let has_owner = src[current] != 0;
        current += 1;
        let owner = if has_owner {
            Some(Pubkey::from(<[u8; 32]>::try_from(&src[current..current + 32]).unwrap()))
        } else {
            None
        };
        if has_owner {
            current += 32;
        }
        
        let houses = src[current];
        
        Ok(Property {
            name,
            color,
            cost,
            rent,
            house_cost,
            hotel_cost,
            house_rent,
            owner,
            houses,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialTile {
    Go,
    Jail,
    FreeParking,
    GoToJail,
    CommunityChest,
    Chance,
    IncomeTax,
    LuxuryTax,
}

impl Pack for SpecialTile {
    const LEN: usize = 1;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = match self {
            SpecialTile::Go => 0,
            SpecialTile::Jail => 1,
            SpecialTile::FreeParking => 2,
            SpecialTile::GoToJail => 3,
            SpecialTile::CommunityChest => 4,
            SpecialTile::Chance => 5,
            SpecialTile::IncomeTax => 6,
            SpecialTile::LuxuryTax => 7,
        };
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Ok(match src[0] {
            0 => SpecialTile::Go,
            1 => SpecialTile::Jail,
            2 => SpecialTile::FreeParking,
            3 => SpecialTile::GoToJail,
            4 => SpecialTile::CommunityChest,
            5 => SpecialTile::Chance,
            6 => SpecialTile::IncomeTax,
            7 => SpecialTile::LuxuryTax,
            _ => return Err(ProgramError::InvalidAccountData),
        })
    }
}

#[derive(Debug, Clone)]
pub enum TileType {
    Property(Property),
    Special(SpecialTile),
}

impl TileType {
    pub fn as_property(&self) -> Option<&Property> {
        match self {
            TileType::Property(prop) => Some(prop),
            _ => None,
        }
    }
}

impl Pack for TileType {
    const LEN: usize = 1025; // 1 byte for type + max(Property::LEN, SpecialTile::LEN)

    fn pack_into_slice(&self, dst: &mut [u8]) {
        match self {
            TileType::Property(property) => {
                dst[0] = 0;
                property.pack_into_slice(&mut dst[1..]);
            }
            TileType::Special(special) => {
                dst[0] = 1;
                special.pack_into_slice(&mut dst[1..]);
            }
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        match src[0] {
            0 => Ok(TileType::Property(Property::unpack_from_slice(&src[1..])?)),
            1 => Ok(TileType::Special(SpecialTile::unpack_from_slice(&src[1..])?)),
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub cash: u64,
    pub position: u8,
    pub jail_turns: u8,
    pub properties: Vec<u8>,
    pub get_out_of_jail_cards: u8,
}


impl Pack for Player {
    const LEN: usize = 512; // Large enough for player data

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut current = 0;
        
        // Write name length and bytes
        let name_bytes = self.name.as_bytes();
        dst[current..current + 4].copy_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        current += 4;
        dst[current..current + name_bytes.len()].copy_from_slice(name_bytes);
        current += name_bytes.len();
        
        // Write player state
        dst[current..current + 8].copy_from_slice(&self.cash.to_le_bytes());
        current += 8;
        dst[current] = self.position;
        current += 1;
        dst[current] = self.jail_turns;
        current += 1;
        
        // Write properties
        dst[current..current + 4].copy_from_slice(&(self.properties.len() as u32).to_le_bytes());
        current += 4;
        dst[current..current + self.properties.len()].copy_from_slice(&self.properties);
        current += self.properties.len();
        
        // Write jail cards
        dst[current] = self.get_out_of_jail_cards;
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut current = 0;
        
        let name_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;
        let name = String::from_utf8(src[current..current + name_len].to_vec())
            .map_err(|_| ProgramError::InvalidAccountData)?;
        current += name_len;
        
        let cash = u64::from_le_bytes(src[current..current + 8].try_into().unwrap());
        current += 8;
        
        let position = src[current];
        current += 1;
        
        let jail_turns = src[current];
        current += 1;
        
        let properties_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;
        let properties = src[current..current + properties_len].to_vec();
        current += properties_len;
        
        let get_out_of_jail_cards = src[current];
        
        Ok(Player {
            name,
            cash,
            position,
            jail_turns,
            properties,
            get_out_of_jail_cards,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub board: Vec<TileType>,
    pub players: Vec<Player>,
    pub current_player: u8,
    pub free_parking: u64,
    pub initialized: bool,
}

impl Pack for Game {
    const LEN: usize = 8192; // Large enough for game state

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut current = 0;
        
        // Write board length and tiles
        dst[current..current + 4].copy_from_slice(&(self.board.len() as u32).to_le_bytes());
        current += 4;
        for tile in &self.board {
            tile.pack_into_slice(&mut dst[current..current + TileType::LEN]);
            current += TileType::LEN;
        }
        
        // Write players length and players
        dst[current..current + 4].copy_from_slice(&(self.players.len() as u32).to_le_bytes());
        current += 4;
        for player in &self.players {
            player.pack_into_slice(&mut dst[current..current + Player::LEN]);
            current += Player::LEN;
        }
        
        // Write game state
        dst[current] = self.current_player;
        current += 1;
        dst[current..current + 8].copy_from_slice(&self.free_parking.to_le_bytes());
        current += 8;
        dst[current] = self.initialized as u8;
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut current = 0;
        
        let board_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;
        let mut board = Vec::with_capacity(board_len);
        for _ in 0..board_len {
            board.push(TileType::unpack_from_slice(&src[current..current + TileType::LEN])?);
            current += TileType::LEN;
        }
        
        let players_len = u32::from_le_bytes(src[current..current + 4].try_into().unwrap()) as usize;
        current += 4;
        let mut players = Vec::with_capacity(players_len);
        for _ in 0..players_len {
            players.push(Player::unpack_from_slice(&src[current..current + Player::LEN])?);
            current += Player::LEN;
        }
        
        let current_player = src[current];
        current += 1;
        
        let free_parking = u64::from_le_bytes(src[current..current + 8].try_into().unwrap());
        current += 8;
        
        let initialized = src[current] != 0;
        
        Ok(Game {
            board,
            players,
            current_player,
            free_parking,
            initialized,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Card {
    CollectMoney(u64),
    PayMoney(u64),
    Move(u8),
    GetOutOfJail,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_pack() {
        // Test packing and unpacking all color variants
        let colors = vec![
            Color::Brown,
            Color::LightBlue,
            Color::Pink,
            Color::Orange,
            Color::Red,
            Color::Yellow,
            Color::Green,
            Color::Blue,
            Color::DarkBlue,
        ];

        for (i, color) in colors.iter().enumerate() {
            let mut packed = vec![0; Color::LEN];
            color.pack_into_slice(&mut packed);
            
            // Verify packed value matches expected byte
            assert_eq!(packed[0], i as u8, "Color {:?} should pack to byte {}", color, i);
            
            // Verify unpacking returns original color
            let unpacked = Color::unpack_from_slice(&packed).unwrap();
            assert_eq!(&unpacked, color, "Unpacked color should match original");
        }

        // Test invalid byte value returns error
        let invalid_packed = vec![9]; // Invalid value (only 0-8 are valid)
        let result = Color::unpack_from_slice(&invalid_packed);
        assert!(result.is_err(), "Invalid byte should return error");
        assert_eq!(
            result.unwrap_err(),
            ProgramError::InvalidAccountData,
            "Invalid byte should return InvalidAccountData error"
        );
    }

    #[test]
    fn test_property_pack() {
        use solana_program::pubkey::Pubkey;

        // Test case 1: Property without owner
        let property1 = Property {
            name: String::from("Boardwalk"),
            color: Color::DarkBlue,
            cost: 400,
            rent: vec![50, 200, 600, 1400, 1700, 2000],
            house_cost: 200,
            hotel_cost: 200,
            house_rent: vec![200, 600, 1400, 1700, 2000],
            owner: None,
            houses: 0,
        };

        // Test case 2: Property with owner
        let owner = Pubkey::new_unique();
        let property2 = Property {
            name: String::from("Park Place"),
            color: Color::DarkBlue,
            cost: 350,
            rent: vec![35, 175, 500, 1100, 1300, 1500],
            house_cost: 200,
            hotel_cost: 200,
            owner: Some(owner),
            houses: 2,
        };

        for property in [property1, property2] {
            let mut packed = vec![0; Property::LEN];
            property.pack_into_slice(&mut packed);

            // Verify unpacking returns the same property
            let unpacked = Property::unpack_from_slice(&packed).unwrap();
            assert_eq!(unpacked.name, property.name);
            assert_eq!(unpacked.color, property.color);
            assert_eq!(unpacked.cost, property.cost);
            assert_eq!(unpacked.rent, property.rent);
            assert_eq!(unpacked.house_cost, property.house_cost);
            assert_eq!(unpacked.hotel_cost, property.hotel_cost);
            assert_eq!(unpacked.owner, property.owner);
            assert_eq!(unpacked.houses, property.houses);
        }

        // Test case 3: Error case - invalid UTF-8 in name
        let mut invalid_packed = vec![0; Property::LEN];
        // Write an invalid UTF-8 sequence for the name length
        invalid_packed[0..4].copy_from_slice(&(5u32).to_le_bytes());
        // Write invalid UTF-8 bytes
        invalid_packed[4..9].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        
        let result = Property::unpack_from_slice(&invalid_packed);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);

        // Verify LEN is sufficient for maximum property data
        let max_property = Property {
            name: "Very Long Property Name That Tests The Size Limit".to_string(),
            color: Color::DarkBlue,
            cost: u64::MAX,
            rent: vec![u64::MAX; 10], // Large rent array
            house_cost: u64::MAX,
            hotel_cost: u64::MAX,
            owner: Some(Pubkey::new_unique()),
            houses: u8::MAX,
        };

        let mut max_packed = vec![0; Property::LEN];
        max_property.pack_into_slice(&mut max_packed);
        
        // Verify we can unpack it successfully
        let unpacked = Property::unpack_from_slice(&max_packed).unwrap();
        assert_eq!(unpacked.name, max_property.name);
    }

    #[test]
    fn test_property_as_property() {
        let property = Property {
            name: String::from("Boardwalk"),
            color: Color::DarkBlue,
            cost: 400,
            rent: vec![50, 200, 600, 1400, 1700, 2000],
            house_cost: 200,
            hotel_cost: 200,
            house_rent: vec![200, 600, 1400, 1700, 2000],
            owner: None,
            houses: 0,
        };

        // Test that as_property returns Some containing a reference to the same property
        let property_ref = property.as_property();
        assert!(property_ref.is_some(), "as_property should return Some");
        
        let unwrapped = property_ref.unwrap();
        assert_eq!(unwrapped.name, property.name);
        assert_eq!(unwrapped.color, property.color);
        assert_eq!(unwrapped.cost, property.cost);
        assert_eq!(unwrapped.rent, property.rent);
        assert_eq!(unwrapped.house_cost, property.house_cost);
        assert_eq!(unwrapped.hotel_cost, property.hotel_cost);
        assert_eq!(unwrapped.owner, property.owner);
        assert_eq!(unwrapped.houses, property.houses);
    }

    #[test]
    fn test_special_tile_pack() {
        // Test all variants
        let tiles = vec![
            SpecialTile::Go,
            SpecialTile::Jail,
            SpecialTile::FreeParking,
            SpecialTile::GoToJail,
            SpecialTile::CommunityChest,
            SpecialTile::Chance,
            SpecialTile::IncomeTax,
            SpecialTile::LuxuryTax,
        ];

        for tile in tiles {
            let mut packed = vec![0; SpecialTile::LEN];
            tile.pack_into_slice(&mut packed);

            // Verify unpacking returns the same tile
            let unpacked = SpecialTile::unpack_from_slice(&packed).unwrap();
            assert_eq!(unpacked, tile, "Unpacked tile should match original");

            // Verify the packed byte matches expected value
            let expected_byte = match tile {
                SpecialTile::Go => 0,
                SpecialTile::Jail => 1,
                SpecialTile::FreeParking => 2,
                SpecialTile::GoToJail => 3,
                SpecialTile::CommunityChest => 4,
                SpecialTile::Chance => 5,
                SpecialTile::IncomeTax => 6,
                SpecialTile::LuxuryTax => 7,
            };
            assert_eq!(packed[0], expected_byte, "Packed byte should match expected value");
        }

        // Test error case - invalid byte value
        let mut invalid_packed = vec![0; SpecialTile::LEN];
        invalid_packed[0] = 8; // First invalid value
        let result = SpecialTile::unpack_from_slice(&invalid_packed);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);

        // Test error case - maximum invalid byte value
        invalid_packed[0] = u8::MAX;
        let result = SpecialTile::unpack_from_slice(&invalid_packed);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_tile_type_as_property() {
        use solana_program::pubkey::Pubkey;

        // Create a test property
        let property = Property {
            name: String::from("Boardwalk"),
            color: Color::DarkBlue,
            cost: 400,
            rent: vec![50, 200, 600, 1400, 1700, 2000],
            house_cost: 200,
            hotel_cost: 200,
            house_rent: vec![200, 600, 1400, 1700, 2000],
            owner: None,
            houses: 0,
        };

        // Test Property variant
        let tile_property = TileType::Property(property.clone());
        let property_ref = tile_property.as_property();
        assert!(property_ref.is_some(), "Property variant should return Some");
        let unwrapped = property_ref.unwrap();
        assert_eq!(unwrapped.name, property.name);
        assert_eq!(unwrapped.color, property.color);
        assert_eq!(unwrapped.cost, property.cost);
        assert_eq!(unwrapped.rent, property.rent);
        assert_eq!(unwrapped.house_cost, property.house_cost);
        assert_eq!(unwrapped.hotel_cost, property.hotel_cost);
        assert_eq!(unwrapped.owner, property.owner);
        assert_eq!(unwrapped.houses, property.houses);

        // Test Special variant
        let tile_special = TileType::Special(SpecialTile::Go);
        let special_ref = tile_special.as_property();
        assert!(special_ref.is_none(), "Special variant should return None");
    }

    #[test]
    fn test_tile_type_pack() {
        // Test Property variant
        let property = Property {
            name: String::from("Boardwalk"),
            color: Color::DarkBlue,
            cost: 400,
            rent: vec![50, 200, 600, 1400, 1700, 2000],
            house_cost: 200,
            hotel_cost: 200,
            house_rent: vec![200, 600, 1400, 1700, 2000],
            owner: None,
            houses: 0,
        };
        let tile_property = TileType::Property(property.clone());
        let mut packed = vec![0; TileType::LEN];
        tile_property.pack_into_slice(&mut packed);

        // Verify discriminator byte
        assert_eq!(packed[0], 0, "Property variant should have discriminator 0");

        // Verify property data is correctly packed after discriminator
        let mut expected_property_bytes = vec![0; Property::LEN];
        property.pack_into_slice(&mut expected_property_bytes);
        assert_eq!(&packed[1..Property::LEN+1], &expected_property_bytes[..], 
            "Property data should match after discriminator");

        // Test unpacking Property variant
        let unpacked = TileType::unpack_from_slice(&packed).unwrap();
        match unpacked {
            TileType::Property(unpacked_property) => {
                assert_eq!(unpacked_property.name, property.name);
                assert_eq!(unpacked_property.color, property.color);
                assert_eq!(unpacked_property.cost, property.cost);
                assert_eq!(unpacked_property.rent, property.rent);
                assert_eq!(unpacked_property.house_cost, property.house_cost);
                assert_eq!(unpacked_property.hotel_cost, property.hotel_cost);
                assert_eq!(unpacked_property.owner, property.owner);
                assert_eq!(unpacked_property.houses, property.houses);
            }
            _ => panic!("Unpacked wrong variant"),
        }

        // Test Special variant
        let special = SpecialTile::Go;
        let tile_special = TileType::Special(special);
        let mut packed = vec![0; TileType::LEN];
        tile_special.pack_into_slice(&mut packed);

        // Verify discriminator byte
        assert_eq!(packed[0], 1, "Special variant should have discriminator 1");

        // Verify special tile data is correctly packed after discriminator
        let mut expected_special_bytes = vec![0; SpecialTile::LEN];
        special.pack_into_slice(&mut expected_special_bytes);
        assert_eq!(&packed[1..SpecialTile::LEN+1], &expected_special_bytes[..],
            "Special tile data should match after discriminator");

        // Test unpacking Special variant
        let unpacked = TileType::unpack_from_slice(&packed).unwrap();
        match unpacked {
            TileType::Special(unpacked_special) => {
                assert_eq!(unpacked_special, special);
            }
            _ => panic!("Unpacked wrong variant"),
        }

        // Test error case - invalid discriminator
        let mut invalid_packed = vec![0; TileType::LEN];
        invalid_packed[0] = 2; // Invalid discriminator
        let result = TileType::unpack_from_slice(&invalid_packed);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);

        // Verify TileType::LEN is sufficient
        assert!(TileType::LEN >= Property::LEN + 1, 
            "TileType::LEN must accommodate Property variant plus discriminator");
        assert!(TileType::LEN >= SpecialTile::LEN + 1,
            "TileType::LEN must accommodate SpecialTile variant plus discriminator");
    }

    #[test]
    fn test_player_pack() {
        // Test basic player
        let player = Player {
            name: String::from("John Doe"),
            cash: 1500,
            position: 0,
            jail_turns: 0,
            properties: vec![1, 3, 6, 8, 9],
            get_out_of_jail_cards: 2,
        };
        let mut packed = vec![0; Player::LEN];
        player.pack_into_slice(&mut packed);

        // Test unpacking
        let unpacked = Player::unpack_from_slice(&packed).unwrap();
        assert_eq!(unpacked.name, player.name);
        assert_eq!(unpacked.cash, player.cash);
        assert_eq!(unpacked.position, player.position);
        assert_eq!(unpacked.jail_turns, player.jail_turns);
        assert_eq!(unpacked.properties, player.properties);
        assert_eq!(unpacked.get_out_of_jail_cards, player.get_out_of_jail_cards);

        // Test player with maximum values
        let max_player = Player {
            name: "A".repeat(256), // Large but reasonable name
            cash: u64::MAX,
            position: u8::MAX,
            jail_turns: u8::MAX,
            properties: (0..40).collect(), // Maximum possible properties (all board spaces)
            get_out_of_jail_cards: u8::MAX,
        };
        let mut packed = vec![0; Player::LEN];
        max_player.pack_into_slice(&mut packed);

        // Test unpacking maximum values
        let unpacked = Player::unpack_from_slice(&packed).unwrap();
        assert_eq!(unpacked.name, max_player.name);
        assert_eq!(unpacked.cash, max_player.cash);
        assert_eq!(unpacked.position, max_player.position);
        assert_eq!(unpacked.jail_turns, max_player.jail_turns);
        assert_eq!(unpacked.properties, max_player.properties);
        assert_eq!(unpacked.get_out_of_jail_cards, max_player.get_out_of_jail_cards);

        // Test error case - invalid UTF-8 in name
        let mut invalid_packed = vec![0; Player::LEN];
        // Write invalid UTF-8 sequence length
        invalid_packed[0..4].copy_from_slice(&(4u32).to_le_bytes());
        // Write invalid UTF-8 sequence
        invalid_packed[4] = 0xFF;
        invalid_packed[5] = 0xFF;
        invalid_packed[6] = 0xFF;
        invalid_packed[7] = 0xFF;
        let result = Player::unpack_from_slice(&invalid_packed);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);

        // Verify Player::LEN is sufficient for maximum data
        let required_len = 4 + // name length
            256 + // max reasonable name length
            8 + // cash
            1 + // position
            1 + // jail_turns
            4 + // properties length
            40 + // max properties
            1; // get_out_of_jail_cards
        assert!(Player::LEN >= required_len, 
            "Player::LEN must be at least {} for maximum data", required_len);
    }

    #[test]
    fn test_game_pack() {
        // Create test property for board
        let property = Property {
            name: String::from("Boardwalk"),
            color: Color::DarkBlue,
            cost: 400,
            rent: vec![50, 200, 600, 1400, 1700, 2000],
            house_cost: 200,
            hotel_cost: 200,
            house_rent: vec![200, 600, 1400, 1700, 2000],
            owner: None,
            houses: 0,
        };

        // Create test player
        let player = Player {
            name: String::from("John Doe"),
            cash: 1500,
            position: 0,
            jail_turns: 0,
            properties: vec![1, 3],
            get_out_of_jail_cards: 1,
        };

        // Test basic game
        let game = Game {
            board: vec![
                TileType::Special(SpecialTile::Go),
                TileType::Property(property.clone()),
                TileType::Special(SpecialTile::CommunityChest),
            ],
            players: vec![player.clone()],
            current_player: 0,
            free_parking: 500,
            initialized: true,
        };
        let mut packed = vec![0; Game::LEN];
        game.pack_into_slice(&mut packed);

        // Test unpacking basic game
        let unpacked = Game::unpack_from_slice(&packed).unwrap();
        assert_eq!(unpacked.board.len(), game.board.len());
        for (unpacked_tile, original_tile) in unpacked.board.iter().zip(game.board.iter()) {
            match (unpacked_tile, original_tile) {
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
                }
                (TileType::Special(up), TileType::Special(og)) => {
                    assert_eq!(up, og);
                }
                _ => panic!("Tile types don't match"),
            }
        }
        assert_eq!(unpacked.players.len(), game.players.len());
        assert_eq!(unpacked.players[0].name, game.players[0].name);
        assert_eq!(unpacked.players[0].cash, game.players[0].cash);
        assert_eq!(unpacked.players[0].position, game.players[0].position);
        assert_eq!(unpacked.players[0].jail_turns, game.players[0].jail_turns);
        assert_eq!(unpacked.players[0].properties, game.players[0].properties);
        assert_eq!(unpacked.players[0].get_out_of_jail_cards, game.players[0].get_out_of_jail_cards);
        assert_eq!(unpacked.current_player, game.current_player);
        assert_eq!(unpacked.free_parking, game.free_parking);
        assert_eq!(unpacked.initialized, game.initialized);

        // Test maximum size game
        let max_property = Property {
            name: "A".repeat(256),
            color: Color::DarkBlue,
            cost: u64::MAX,
            rent: vec![u64::MAX; 6],
            house_cost: u64::MAX,
            hotel_cost: u64::MAX,
            house_rent: vec![u64::MAX; 5],
            owner: Some(solana_program::pubkey::Pubkey::new_unique()),
            houses: 5,
        };
        let max_player = Player {
            name: "A".repeat(256),
            cash: u64::MAX,
            position: u8::MAX,
            jail_turns: u8::MAX,
            properties: (0..40).collect(),
            get_out_of_jail_cards: u8::MAX,
        };
        let max_game = Game {
            board: vec![TileType::Property(max_property); 40], // Maximum board size
            players: vec![max_player; 8], // Maximum players
            current_player: u8::MAX,
            free_parking: u64::MAX,
            initialized: true,
        };
        let mut packed = vec![0; Game::LEN];
        max_game.pack_into_slice(&mut packed);

        // Test unpacking maximum size game
        let unpacked = Game::unpack_from_slice(&packed).unwrap();
        assert_eq!(unpacked.board.len(), max_game.board.len());
        assert_eq!(unpacked.players.len(), max_game.players.len());
        assert_eq!(unpacked.current_player, max_game.current_player);
        assert_eq!(unpacked.free_parking, max_game.free_parking);
        assert_eq!(unpacked.initialized, max_game.initialized);

        // Verify Game::LEN is sufficient
        let required_len = 4 + // board length
            (40 * TileType::LEN) + // maximum board size
            4 + // players length
            (8 * Player::LEN) + // maximum players
            1 + // current_player
            8 + // free_parking
            1; // initialized
        assert!(Game::LEN >= required_len,
            "Game::LEN must be at least {} for maximum data", required_len);

        // Test error case - invalid board tile
        let mut invalid_packed = packed.clone();
        // Corrupt first tile's discriminator
        invalid_packed[4] = 2; // Invalid TileType discriminator
        let result = Game::unpack_from_slice(&invalid_packed);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);

        // Test error case - invalid player data
        let mut invalid_packed = packed.clone();
        let player_data_start = 4 + (40 * TileType::LEN) + 4; // After board length, board data, and players length
        // Corrupt first player's name with invalid UTF-8
        invalid_packed[player_data_start + 4] = 0xFF; // Invalid UTF-8 byte
        let result = Game::unpack_from_slice(&invalid_packed);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProgramError::InvalidAccountData);
    }

    #[test]
    fn test_card_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_game_is_initialized() {
        let game = Game {
            board: Vec::new(),
            players: Vec::new(),
            current_player: 0,
            free_parking: 0,
            initialized: false,
        };
        assert!(!game.is_initialized(), "Uninitialized game should return false");

        let game = Game {
            board: Vec::new(),
            players: Vec::new(),
            current_player: 0,
            free_parking: 0,
            initialized: true,
        };
        assert!(game.is_initialized(), "Initialized game should return true");
    }

    #[test]
    fn test_player_is_initialized() {
        let player = Player {
            name: String::from("Test Player"),
            cash: 1500,
            position: 0,
            jail_turns: 0,
            properties: Vec::new(),
            get_out_of_jail_cards: 0,
        };
        assert!(player.is_initialized(), "Player should always be initialized");
    }

    #[test]
    fn test_property_is_initialized() {
        let property = Property {
            name: String::from("Test Property"),
            color: Color::Blue,
            cost: 200,
            rent: 10,
            house_cost: 100,
            house_rent: vec![10, 20, 30, 40],
            houses: 0,
            owner: None,
        };
        assert!(property.is_initialized(), "Property should always be initialized");
    }
}

impl Pack for Card {
    const LEN: usize = 9; // 1 byte for type + 8 bytes for value

    fn pack_into_slice(&self, dst: &mut [u8]) {
        match self {
            Card::CollectMoney(amount) => {
                dst[0] = 0;
                dst[1..9].copy_from_slice(&amount.to_le_bytes());
            }
            Card::PayMoney(amount) => {
                dst[0] = 1;
                dst[1..9].copy_from_slice(&amount.to_le_bytes());
            }
            Card::Move(position) => {
                dst[0] = 2;
                dst[1] = *position;
            }
            Card::GetOutOfJail => {
                dst[0] = 3;
            }
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Ok(match src[0] {
            0 => Card::CollectMoney(u64::from_le_bytes(src[1..9].try_into().unwrap())),
            1 => Card::PayMoney(u64::from_le_bytes(src[1..9].try_into().unwrap())),
            2 => Card::Move(src[1]),
            3 => Card::GetOutOfJail,
            _ => return Err(ProgramError::InvalidAccountData),
        })
    }
}
