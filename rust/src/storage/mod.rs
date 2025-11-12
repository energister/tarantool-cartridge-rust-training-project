pub mod dto;
pub mod place_storage;
pub mod weather_storage;

use tlua::{LuaRead, PushInto};
use crate::data_fetcher;

#[derive(Debug, LuaRead, PushInto)]
pub enum PlaceCoordinates {
    CouldNotBeFound([(); 0]),
    Value(dto::Coordinates),
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    place_storage::init()?;
    weather_storage::init()?;
    Ok(())
}
