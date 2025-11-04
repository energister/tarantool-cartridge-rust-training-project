use serde::{Deserialize, Serialize};
use tlua::{LuaRead, PushInto};

#[derive(Debug, Serialize, Deserialize, LuaRead, PushInto)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}