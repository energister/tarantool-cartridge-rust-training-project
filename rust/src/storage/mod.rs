pub mod api;
mod place_storage;
mod weather_storage;

use crate::data_fetcher;
use serde::{Deserialize, Serialize};
use shors::transport;
use tarantool;
use tarantool::datetime::Datetime;
use time::OffsetDateTime;
use tlua::{LuaRead, PushInto};

#[derive(Debug, Clone, LuaRead, PushInto)]
pub enum PlaceCoordinates {
    CouldNotBeFound([(); 0]),
    Value(api::Coordinates),
}

#[derive(Debug, Serialize, Deserialize)]
struct WeatherTuple {
    place_name: String,
    bucket_id: u32,
    point_in_time: Datetime,
    expiration: Datetime,
    weather_data: data_fetcher::api::Weather
}
impl tarantool::tuple::Encode for WeatherTuple {}

#[tarantool::proc]
pub fn create_spaces(is_master: bool) -> Result<(), Box<dyn std::error::Error>> {
    if is_master {
        place_storage::create_space()?;
        weather_storage::create_space()?;
    }
    Ok(())
}

#[tarantool::proc]
pub fn get_weather_for_place(bucket_id: u32, place_name: &str) -> Result<Option<api::StorageResponse>, Box<dyn std::error::Error>> {
    let stored_weather = weather_storage::weather_get(place_name)?;
    // Keep a copy of expiration for logging purposes
    let expiration_for_log = stored_weather.as_ref().map(|w| w.expiration);

    if let Some(stored) = stored_weather {
        let expiration: OffsetDateTime = stored.expiration.into();
        if OffsetDateTime::now_utc() < expiration {
            log::debug!("Cache HIT for weather of '{}' (will expire at {})", &place_name, stored.expiration);

            let stored_coordinates = place_storage::coordinates_get(&place_name)?
                .ok_or("Coordinates should be known if weather is cached")?;

            return Ok(Some(api::StorageResponse {
                coordinates: Some(stored_coordinates),
                weather: Some(stored.weather_data),
                cached: true,
            }))
        }
    }

    log::debug!("Cache MISS for weather of '{}' (expiration was at {:?})", place_name, expiration_for_log);

    let coordinates: Option<PlaceCoordinates> = get_coordinates(bucket_id, &place_name)?;
    Ok(match coordinates {
        None => {
            // failed because of a known error (e.g., network issue)
            None
        },
        Some(PlaceCoordinates::CouldNotBeFound(_)) => {
            // place is not listed in the geo database
            Some(api::StorageResponse {
                coordinates: None,
                weather: None,
                cached: true,
            })
        }
        Some(PlaceCoordinates::Value(coords)) => {
            let weather = fetch_weather(bucket_id, place_name, &coords)?;
            Some(api::StorageResponse {
                coordinates: Some(coords),
                weather,
                cached: false,
            })
        }
    })
}

fn get_coordinates(bucket_id: u32, place_name: &str) -> Result<Option<PlaceCoordinates>, Box<dyn std::error::Error>> {
    let stored_coordinates = place_storage::coordinates_get(place_name)?;
    if let Some(coords) = stored_coordinates {
        return Ok(Some(PlaceCoordinates::Value(coords)));
    }

    match make_remote_call_to_data_fetcher_for_coordinates(place_name)? {
        None => Ok(None),
        Some(coordinates) => {
            if let PlaceCoordinates::Value(ref coords) = coordinates {
                // cache the response
                place_storage::coordinates_put(bucket_id, &place_name, coords.clone())?;
            }

            Ok(Some(coordinates))
        }
    }
}

fn fetch_weather(bucket_id: u32, place_name: &str, coordinates: &api::Coordinates) -> Result<Option<data_fetcher::api::Weather>, Box<dyn std::error::Error>> {
    let weather = make_remote_call_to_data_fetcher_for_weather(&coordinates)?;

    // cache the response
    if let Some(ref w) = weather {
        weather_storage::weather_upsert(&WeatherTuple {
            place_name: place_name.to_owned(),
            bucket_id,
            point_in_time: w.point_in_time,
            expiration: w.expiration,
            weather_data: w.clone(),
        })?;
    }

    Ok(weather)
}

fn make_remote_call_to_data_fetcher_for_coordinates(place_name: &str) -> Result<Option<PlaceCoordinates>, Box<dyn std::error::Error>> {
    let lua = tarantool::lua_state();

    let response: Option<data_fetcher::api::CoordinatesResponse> = transport::rpc::client::Builder::new(&lua)
        .role_endpoint("app.roles.data_fetcher", "/get_coordinates")
        .call(&mut transport::Context::background(), place_name)
        .map_err(|e| {
            log::error!("Failed to request data fetcher: {}", e);
            "Unexpected error while fetching coordinates"
        })?
        .get(0)
        .ok_or_else(|| {
            log::error!("Failed to extract response from tuple at index 0");
            "Unexpected error while fetching coordinates"
        })?;

    Ok(response.map(|c| {
        match c.coordinates {
            None => PlaceCoordinates::CouldNotBeFound([]),
            Some(coords) => PlaceCoordinates::Value(api::Coordinates {
                latitude: coords.latitude,
                longitude: coords.longitude,
            })
        }
    }))
}

fn make_remote_call_to_data_fetcher_for_weather(coordinates: &api::Coordinates) -> Result<Option<data_fetcher::api::Weather>, Box<dyn std::error::Error>> {
    let lua = tarantool::lua_state();

    let response: Option<data_fetcher::api::Weather> = transport::rpc::client::Builder::new(&lua)
        .role_endpoint("app.roles.data_fetcher", "/get_weather")
        .call(&mut transport::Context::background(), (coordinates.latitude, coordinates.longitude))
        .map_err(|e| {
            log::error!("Failed to request data fetcher: {}", e);
            "Unexpected error while fetching weather"
        })?
        .get(0)
        .ok_or_else(|| {
            log::error!("Failed to extract response from tuple at index 0");
            "Unexpected error while fetching weather"
        })?;

    Ok(response)
}
