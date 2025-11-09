use serde::{Deserialize, Serialize};
use tarantool::tlua::LuaRead;
use tlua::PushInto;
use crate::data_fetcher;

#[derive(Debug, Serialize, Deserialize, LuaRead, PushInto)]
pub struct StorageResponse {
    pub coordinates: Option<Coordinates>,
    pub weather: Option<data_fetcher::dto::Weather>,
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize, LuaRead, PushInto)]
pub struct Coordinates {
    pub longitude: f64,
    pub latitude: f64,
}
