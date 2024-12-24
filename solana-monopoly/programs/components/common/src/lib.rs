use bolt_lang::prelude::*;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Card {
    CollectMoney(u64),
    PayMoney(u64),
    Move(u8),
    GetOutOfJail,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
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

#[derive(Component)]
pub enum TileType {
    Property(Property),
    Special(SpecialTile),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
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
