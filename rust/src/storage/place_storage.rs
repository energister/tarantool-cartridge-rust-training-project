use crate::storage::api;
use serde::{Deserialize, Serialize};
use tarantool::space::{FieldType, Space};
use tarantool::tuple::Tuple;

const SPACE_NAME: &str = "place";

#[derive(Debug, Serialize, Deserialize)]
struct PlaceTuple {
    place_name: String,
    bucket_id: u32,
    coordinates: api::Coordinates,
}
impl tarantool::tuple::Encode for PlaceTuple {}

pub fn create_space() -> Result<(), Box<dyn std::error::Error>> {
    let places = Space::builder(SPACE_NAME)
        .field(("place_name", FieldType::String))
        .field(("bucket_id", FieldType::Unsigned))
        .field(("coordinates", FieldType::Array))
        // create space only if it does not exist
        .if_not_exists(true)
        .create()?;

    places.index_builder("primary")
        .parts(["place_name"])
        .if_not_exists(true)
        .create()?;

    // required for vshard
    places.index_builder("bucket_id")
        .parts(["bucket_id"])
        .unique(false)
        .if_not_exists(true)
        .create()?;

    Ok(())
}

pub fn coordinates_put(bucket_id: u32, place_name: &str, coordinates: api::Coordinates) -> Result<Tuple, Box<dyn std::error::Error>> {
    let tuple = PlaceTuple {
        place_name: place_name.to_owned(),
        bucket_id,
        coordinates,
    };

    Space::find(SPACE_NAME)
        .ok_or(format!("Can't find space '{SPACE_NAME}'"))?
        .put(&tuple)
        .map_err(|e| {
            log::error!("Error while storing into '{SPACE_NAME}': {e:?}");
            e.into()
        })
}

pub fn coordinates_get(place_name: &str) -> Result<Option<api::Coordinates>, Box<dyn std::error::Error>> {
    let maybe_stored = Space::find(SPACE_NAME)
        .ok_or(format!("Can't find space '{SPACE_NAME}'"))?
        .get(&(place_name,))?
        .map(|record| record.decode::<PlaceTuple>())
        .transpose()?
        .map(|place| place.coordinates);
    Ok(maybe_stored)
}
