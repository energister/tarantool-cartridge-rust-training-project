pub mod dto;
mod place_storage;
pub mod weather_storage;

use crate::data_fetcher;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    place_storage::init()?;
    weather_storage::init()?;
    Ok(())
}
