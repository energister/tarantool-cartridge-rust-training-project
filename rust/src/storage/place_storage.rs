use serde::{Deserialize, Serialize};
use tarantool::space::{FieldType, IsNullable, Space};
use tarantool::tuple::Tuple;
use crate::storage::{dto, PlaceCoordinates};

const SPACE_NAME: &str = "place";

#[derive(Debug, Serialize, Deserialize)]
struct PlaceTuple {
    place_name: String,
    bucket_id: u32,
    // `None` means that the place is not listed in the geo database
    coordinates: Option<dto::Coordinates>,
}
impl tarantool::tuple::Encode for PlaceTuple {}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    create_place_space()
}

fn create_place_space() -> Result<(), Box<dyn std::error::Error>> {
    let places = Space::builder(SPACE_NAME)
        .field(("place_name", FieldType::String))
        .field(("bucket_id", FieldType::Unsigned))
        .field(("coordinates", FieldType::Array, IsNullable::Nullable)) // nullable field
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

pub fn coordinates_put(bucket_id: u32, place_name: String, coordinates: PlaceCoordinates) -> Result<Tuple, Box<dyn std::error::Error>> {
    let tuple = PlaceTuple {
        place_name,
        bucket_id,
        coordinates: match coordinates {
            PlaceCoordinates::Value(coords) => Some(coords),
            PlaceCoordinates::CouldNotBeFound(_) => None,
        },
    };

    Space::find(SPACE_NAME)
        .ok_or(format!("Can't find space '{SPACE_NAME}'"))?
        .put(&tuple)
        .map_err(|e| {
            log::error!("Error while storing into '{SPACE_NAME}': {e:?}");
            e.into()
        })
}

pub fn coordinates_get(place_name: String) -> Result<Option<PlaceCoordinates>, Box<dyn std::error::Error>> {
    let maybe_stored = Space::find(SPACE_NAME)
        .ok_or(format!("Can't find space '{SPACE_NAME}'"))?
        .get(&(place_name,))?
        .map(|record| record.decode::<PlaceTuple>())
        .transpose()?
        .map(|place|
            match place.coordinates {
                Some(coords) => PlaceCoordinates::Value(coords),
                None => PlaceCoordinates::CouldNotBeFound([]),
            }
        );
    Ok(maybe_stored)
}
