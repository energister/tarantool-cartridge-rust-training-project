use tarantool::space::{FieldType, Space};

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    create_weather_space()
}


fn create_weather_space() -> Result<(), Box<dyn std::error::Error>> {
    let weather = Space::builder("weather")
        .format([
            ("place_name", FieldType::String),
            ("bucket_id", FieldType::Unsigned),
            ("point_in_time", FieldType::Datetime),
            ("expiration", FieldType::Datetime),
            ("weather_data", FieldType::Map),
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