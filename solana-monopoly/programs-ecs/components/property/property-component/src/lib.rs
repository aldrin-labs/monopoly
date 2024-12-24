use bolt_lang::prelude::*;
use anchor_lang::prelude::*;
use common::{Color, Property as PropertyData};

#[derive(Component)]
pub struct Property {
    pub data: PropertyData,
}

#[program]
pub mod property_component {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_property(
        ctx: Context<Initialize>,
        name: String,
        color: Color,
        cost: u64,
        rent: Vec<u64>,
        house_cost: u64,
        hotel_cost: u64,
    ) -> Result<()> {
        let property = PropertyData {
            name,
            color,
            cost,
            rent,
            house_cost,
            hotel_cost,
            owner: None,
            houses: 0,
        };
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
