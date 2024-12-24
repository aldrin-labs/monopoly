use bolt_lang::prelude::*;
use bolt_lang::{component, bolt_program, ComponentTraits, BoltMetadata, World, BoltError, Entity};
use bolt_component::ID;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq)]
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

impl bolt_lang::Space for Color {
    const INIT_SPACE: usize = 8; // Number of variants in Color enum
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum Card {
    CollectMoney(u64),
    PayMoney(u64),
    Move(u8),
    GetOutOfJail,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
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

pub struct TileType {
    pub property: Option<Property>,
    pub special: Option<SpecialTile>,
}

#[component]
pub struct Property {
    #[max_len(32)]
    pub name: String,
    pub color: Color,
    pub cost: u64,
    #[max_len(6)]
    pub rent: Vec<u64>,
    pub house_cost: u64,
    pub hotel_cost: u64,
    pub owner: Option<Pubkey>,
    pub houses: u8,
}

impl Default for Property {
    fn default() -> Self {
        Self {
            name: String::new(),
            color: Color::Purple,
            cost: 0,
            rent: Vec::new(),
            house_cost: 0,
            hotel_cost: 0,
            owner: None,
            houses: 0,
            bolt_metadata: BoltMetadata::default(),
        }
    }
}
