use serde::{Deserialize, Serialize};
use tarantool::datetime::Datetime;
use tarantool::tlua::LuaRead;

#[derive(Debug, Serialize, Deserialize, LuaRead)]
pub struct StorageResponse {
    pub coordinates: Option<Coordinates>,
    pub weather: Option<Weather>,
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize, LuaRead)]
pub struct Coordinates {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Debug, Serialize, Deserialize, LuaRead)]
pub struct Weather {
    pub point_in_time: Datetime,
    pub expiration: Datetime,
    pub temperature_celsius: f64,
}
