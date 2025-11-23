use crate::storage::WeatherTuple;
use tarantool::space::{FieldType, Space};

const SPACE_NAME: &str = "weather";

pub fn create_space() -> Result<(), Box<dyn std::error::Error>> {
    let weather = Space::builder(SPACE_NAME)
        .format([
            ("place_name", FieldType::String),
            ("bucket_id", FieldType::Unsigned),
            ("point_in_time", FieldType::Datetime),
            ("expiration", FieldType::Datetime),
            ("weather_data", FieldType::Array),
        ])
        .if_not_exists(true)
        .create()?;

    weather.index_builder("primary")
        .parts(["place_name"])
        .if_not_exists(true)
        .create()?;

    // required for vshard
    weather.index_builder("bucket_id")
        .parts(["bucket_id"])
        .unique(false)
        .if_not_exists(true)
        .create()?;

    Ok(())
}

pub fn weather_upsert(tuple: &WeatherTuple) -> Result<(), Box<dyn std::error::Error>> {
    Space::find(SPACE_NAME)
        .ok_or(format!("Can't find space '{SPACE_NAME}'"))?
        .put(&tuple)
        .map_err(|e| {
            log::error!("Error while storing into '{SPACE_NAME}': {e:?}");
            e.into()
        })
        .map(|_| ())
}

pub fn weather_get(place_name: &str) -> Result<Option<WeatherTuple>, Box<dyn std::error::Error>> {
    let maybe_stored = Space::find(SPACE_NAME)
        .ok_or(format!("Can't find space '{SPACE_NAME}'"))?
        .get(&(place_name,))?
        .map(|record| record.decode::<WeatherTuple>())
        .transpose()?;
    Ok(maybe_stored)
}