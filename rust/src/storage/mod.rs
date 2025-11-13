pub mod dto;
mod place_storage;
mod weather_storage;

use crate::data_fetcher;
use tarantool::lua_state;
use time::OffsetDateTime;
use tlua::{LuaFunction, LuaRead, PushInto};

#[derive(Debug, Clone, LuaRead, PushInto)]
pub enum PlaceCoordinates {
    CouldNotBeFound([(); 0]),
    Value(dto::Coordinates),
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    place_storage::init()?;
    weather_storage::init()?;
    Ok(())
}

pub fn get_weather_for_place(bucket_id: u32, place_name: String) -> Result<Option<dto::StorageResponse>, Box<dyn std::error::Error>> {

    let stored_weather: Option<data_fetcher::dto::Weather> = weather_storage::weather_get(place_name.clone())?;
    // Keep a copy of expiration for logging purposes
    let expiration_for_log = stored_weather.as_ref().map(|w| w.expiration);

    if let Some(weather) = stored_weather {
        let expiration: OffsetDateTime = weather.expiration.into();
        if OffsetDateTime::now_utc() < expiration {
            log::debug!("Cache HIT for weather of '{}' (will expire at {})", &place_name, weather.expiration);

            let stored_coordinates = place_storage::coordinates_get(&place_name)?
                .ok_or("Coordinates should be known if weather is cached")?;

            return match stored_coordinates {
                PlaceCoordinates::CouldNotBeFound(_) => {
                    Err("Coordinates should be known if weather is cached".into())
                },
                PlaceCoordinates::Value(coord) => {
                    Ok(Some(dto::StorageResponse {
                        coordinates: Some(coord),
                        weather: Some(weather),
                        cached: true,
                    }))
                }
            };
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
            Some(dto::StorageResponse {
                coordinates: None,
                weather: None,
                cached: true,
            })
        }
        Some(PlaceCoordinates::Value(coords)) => {
            let weather = fetch_weather(bucket_id, place_name, &coords)?;
            Some(dto::StorageResponse {
                coordinates: Some(coords),
                weather,
                cached: false,
            })
        }
    })
}

fn get_coordinates(bucket_id: u32, place_name: &String) -> Result<Option<PlaceCoordinates>, Box<dyn std::error::Error>> {
    let stored_coordinates = place_storage::coordinates_get(place_name)?;
    if stored_coordinates.is_some() {
        return Ok(stored_coordinates);
    }

    let response = make_remote_call_to_data_fetcher_for_coordinates(place_name)?;

    Ok(if response.is_none() {
        None
    } else {
        // cache the response
        let coordinates = response.unwrap();
        place_storage::coordinates_put(bucket_id, &place_name, coordinates.clone())?;
        Some(coordinates)
    })
}

fn fetch_weather(bucket_id: u32, place_name: String, coordinates: &dto::Coordinates) -> Result<Option<data_fetcher::dto::Weather>, Box<dyn std::error::Error>> {
    let weather = make_remote_call_to_data_fetcher_for_weather(&coordinates)?;

    // cache the response
    if let Some(ref w) = weather {
        weather_storage::weather_upsert(bucket_id, &place_name, w.point_in_time, w.expiration, w.clone())?;
    }

    Ok(weather)
}

fn make_remote_call_to_data_fetcher_for_coordinates(place_name: &String) -> Result<Option<PlaceCoordinates>, Box<dyn std::error::Error>> {

    let lua = lua_state();

    // TODO: use shors
    let rpc_function: LuaFunction<_> = lua
        .eval(r#"
            return function(place_name)
                require('checks').checks('string')

                local response, err = require('cartridge').rpc_call('app.roles.data_fetcher', 'get_coordinates', { place_name })
                if err ~= nil then
                    log.error("Failed to perform an RPC call to the data_fetcher.get_coordinates: %s", err)
                    error("Failed to perform an RPC call to the data_fetcher.get_coordinates")
                end

                return response
            end"#,
        )
        .map_err(|e| {
            log::error!("Failed to generate call function: {}", e);
            "Unexpected error while fetching coordinates"
        })?;

    let response: Result<Option<data_fetcher::dto::CoordinatesResponse>, Box<dyn std::error::Error>> = rpc_function
        .call_with_args(&place_name)
        .map_err(|e| {
            log::error!("Failed to request data fetcher: {}", e);
            "Unexpected error while fetching coordinates".into()
        });

    Ok(response?.map(|c| {
        match c.coordinates {
            None => PlaceCoordinates::CouldNotBeFound([]),
            Some(coords) => PlaceCoordinates::Value(dto::Coordinates {
                latitude: coords.latitude,
                longitude: coords.longitude,
            })
        }
    }))
}

fn make_remote_call_to_data_fetcher_for_weather(coordinates: &dto::Coordinates) -> Result<Option<data_fetcher::dto::Weather>, Box<dyn std::error::Error>> {

    let lua = lua_state();

    // TODO: use shors
    let rpc_function: LuaFunction<_> = lua
        .eval(r#"
            return function(coordinates_latitude, coordinates_longitude)
                require('checks').checks('number', 'number')

                local arguments = { coordinates_latitude, coordinates_longitude }
                local response, err = require('cartridge').rpc_call('app.roles.data_fetcher', 'get_weather', arguments)
                if err ~= nil then
                    log.error("Failed to perform an RPC call to the data_fetcher.get_weather: %s", err)
                    error("Failed to perform an RPC call to the data_fetcher.get_weather")
                end

                return response
            end"#,
        )
        .map_err(|e| {
            log::error!("Failed to generate call function: {}", e);
            "Unexpected error while fetching weather"
        })?;

    let response = rpc_function
        .call_with_args((coordinates.latitude, coordinates.longitude))
        .map_err(|e| {
            log::error!("Failed to request data fetcher: {}", e);
            "Unexpected error while fetching weather"
        })?;

    Ok(response)
}