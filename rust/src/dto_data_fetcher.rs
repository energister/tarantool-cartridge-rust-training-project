use serde::{Deserialize, Serialize};
use tlua::{LuaRead, PushInto};

#[derive(Debug, Serialize, Deserialize, LuaRead, PushInto)]
pub struct Coordinates {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}