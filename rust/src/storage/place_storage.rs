use serde::Deserialize;
use tarantool::space::{Field, FieldType, Space};

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    create_place_space()
}


fn create_place_space() -> Result<(), Box<dyn std::error::Error>> {
    let places = Space::builder("place")
        .field(("place_name", FieldType::String))
        .field(("bucket_id", FieldType::Unsigned))
        .field(("coordinates", FieldType::Map))
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
