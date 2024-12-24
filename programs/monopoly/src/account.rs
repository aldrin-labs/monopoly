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
        // TODO: Implement test
    }

    #[test]
    fn test_player_account_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_property_account_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_card_deck_pack() {
        // TODO: Implement test
    }

    #[test]
    fn test_account_validation() {
        // TODO: Implement test
    }
}

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
