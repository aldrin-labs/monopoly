use bolt_lang::prelude::*;
use bolt_component::{BoltComponent, BoltMetadata, Component};
use bolt_lang::{BoltContext, BoltError};
use common::{Color, Property as PropertyData};

#[derive(Component)]
pub struct Property {
    pub data: PropertyData,
    pub bolt_metadata: BoltMetadata,
}

impl BoltComponent for Property {
    fn init_space() -> usize {
        std::mem::size_of::<Self>()
    }
}

#[bolt_program(Property)]
pub mod property_component {
    use super::*;

    pub fn initialize(ctx: &mut BoltContext) -> Result<(), BoltError> {
        Ok(())
    }

    pub fn create_property(
        ctx: &mut BoltContext,
        name: String,
        color: Color,
        cost: u64,
        rent: Vec<u64>,
        house_cost: u64,
        hotel_cost: u64,
    ) -> Result<(), BoltError> {
        let property = PropertyData {
            name,
            color,
            cost,
            rent,
            house_cost,
            hotel_cost,
            owner: None,
            houses: 0,
            bolt_metadata: BoltMetadata::default(),
        };
        ctx.world.add_component(Property { data: property })?;
        Ok(())
    }
}
