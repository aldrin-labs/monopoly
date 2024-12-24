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
    Purple,
    LightBlue,
    Pink,
    Orange,
    Red,
    Yellow,
    Green,
    DarkBlue,
}

impl Pack for Color {
    const LEN: usize = 1;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0] = match self {
            Color::Purple => 0,
            Color::LightBlue => 1,
            Color::Pink => 2,
            Color::Orange => 3,
            Color::Red => 4,
            Color::Yellow => 5,
            Color::Green => 6,
            Color::DarkBlue => 7,
        };
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Ok(match src[0] {
            0 => Color::Purple,
            1 => Color::LightBlue,
            2 => Color::Pink,
            3 => Color::Orange,
            4 => Color::Red,
            5 => Color::Yellow,
            6 => Color::Green,
            7 => Color::DarkBlue,
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
            owner,
            houses,
        })
    }
}

#[derive(Debug, Clone)]
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
        // TODO: Implement test
    }

    #[test]
    fn test_property_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_special_tile_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_tile_type_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_player_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_game_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_card_pack() {
        // TODO: Implement test
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
