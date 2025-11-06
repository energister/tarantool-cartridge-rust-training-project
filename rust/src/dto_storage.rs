use serde::{Deserialize, Serialize};
use tarantool::tlua::LuaRead;
use crate::data_fetcher::dto::Weather;

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
