use crate::data_fetcher;
use serde::{Deserialize, Serialize};
use tarantool::datetime::Datetime;
use tarantool::space::{FieldType, Space};
use tarantool::tuple::Tuple;

const SPACE_NAME: &str = "weather";

#[derive(Debug, Serialize, Deserialize)]
struct WeatherTuple {
    place_name: String,
    bucket_id: u32,
    point_in_time: Datetime,
    expiration: Datetime,
    weather_data: data_fetcher::dto::Weather
}
impl tarantool::tuple::Encode for WeatherTuple {}

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

pub fn weather_upsert(bucket_id: u32, place_name: &String, point_in_time: Datetime, expiration: Datetime, weather: data_fetcher::dto::Weather) -> Result<Tuple, Box<dyn std::error::Error>> {
    let tuple = WeatherTuple {
        place_name: place_name.clone(),
        bucket_id,
        point_in_time,
        expiration,
        weather_data: weather,
    };

    Space::find(SPACE_NAME)
        .ok_or(format!("Can't find space '{SPACE_NAME}'"))?
        .put(&tuple)
        .map_err(|e| {
            log::error!("Error while storing into '{SPACE_NAME}': {e:?}");
            e.into()
        })
}

pub fn weather_get(place_name: String) -> Result<Option<data_fetcher::dto::Weather>, Box<dyn std::error::Error>> {
    let maybe_stored = Space::find(SPACE_NAME)
        .ok_or(format!("Can't find space '{SPACE_NAME}'"))?
        .get(&(place_name,))?
        .map(|record| record.decode::<WeatherTuple>())
        .transpose()? // Option<Result<WeatherTuple, _>> -> Result<Option<WeatherTuple>, _>
        .map(|tuple| {
            let mut weather = tuple.weather_data;
            weather.expiration = tuple.expiration;
            weather
        });
    Ok(maybe_stored)
}