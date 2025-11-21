use std::sync::RwLock;
use std::time::Duration;
use tlua::AnyLuaValue;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Settings {
    pub open_meteo_api: OpenMeteoApiSettings,
}

pub struct OpenMeteoApiSettings {
    request_timeout: RwLock<Duration>,
}

impl OpenMeteoApiSettings {
    pub fn get_request_timeout(&self) -> Duration {
        match self.request_timeout.read() {
            Ok(guard) => *guard,
            Err(_) => DEFAULT_REQUEST_TIMEOUT,
        }
    }

    /// If `None`, the default timeout will be set.
    pub fn set_request_timeout(&self, timeout: Option<Duration>) {
        if let Ok(mut guard) = self.request_timeout.write() {
            *guard = timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT);
            log::info!("Set Open Meteo API request timeout to {:?}", *guard)
        }
    } 
}

pub static SETTINGS: Settings = Settings {
    open_meteo_api: OpenMeteoApiSettings {
        request_timeout: RwLock::new(DEFAULT_REQUEST_TIMEOUT),
    },
};

pub fn validate_request_timeout_in_seconds(seconds: Option<AnyLuaValue>) -> tarantool::Result<bool> {
    match seconds {
        Some(AnyLuaValue::LuaNumber(value)) if value >= 0.0 && value.fract() == 0.0 => Ok(true),
        None => Ok(true),  // default value will be used
        _ => Ok(false)
    }
}

/// If `None`, the default timeout will be set.
#[tarantool::proc]
// TODO: replace parameter with struct
pub fn set_request_timeout_in_seconds(seconds: Option<u64>) -> tarantool::Result<()> {
    SETTINGS.open_meteo_api.set_request_timeout(
        seconds.map(|d| Duration::from_secs(d))
    );
    Ok(())
}

#[tarantool::proc]
pub fn get_request_timeout_in_seconds() -> tarantool::Result<u64> {
    Ok(SETTINGS.open_meteo_api.get_request_timeout().as_secs())
}