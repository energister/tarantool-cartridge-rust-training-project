use std::cell::Cell;
use std::time::Duration;
use once_cell::unsync::Lazy;
use tlua::AnyLuaValue;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Settings {
    pub(crate) open_meteo_api: OpenMeteoApiSettings,
}

pub struct OpenMeteoApiSettings {
    pub(crate) request_timeout: Cell<Duration>,
}

thread_local! {
    pub static SETTINGS: Lazy<Settings> = Lazy::new(||
        Settings {
            open_meteo_api: OpenMeteoApiSettings {
                request_timeout: Cell::new(DEFAULT_REQUEST_TIMEOUT)
            }
        }
    )
}

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
    let duration = seconds.map(Duration::from_secs).unwrap_or(DEFAULT_REQUEST_TIMEOUT);
    SETTINGS.with(|s| s.open_meteo_api.request_timeout.set(duration));
    log::info!("Set Open Meteo API request timeout to {:?}", duration);
    Ok(())
}

#[tarantool::proc]
pub fn get_request_timeout_in_seconds() -> tarantool::Result<u64> {
    Ok(SETTINGS.with(|s| s.open_meteo_api.request_timeout.get().as_secs()))
}
