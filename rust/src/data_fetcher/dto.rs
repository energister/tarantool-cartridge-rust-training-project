use serde::{Deserialize, Serialize};
use tlua::{LuaRead, PushInto};
use tarantool::datetime::Datetime;

#[derive(Debug, Serialize, Deserialize, LuaRead, PushInto)]
pub struct CoordinatesResponse {
    pub coordinates: Option<Coordinates>,
}

#[derive(Debug, Serialize, Deserialize, LuaRead, PushInto)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize, Deserialize, LuaRead, PushInto)]
pub struct Weather {
    pub point_in_time: Datetime,
    pub expiration: Datetime,
    pub temperature_celsius: f64,
}