use std::sync::RwLock;
use std::time::Duration;

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

/// If `None`, the default timeout will be set.
pub fn set_request_timeout(seconds: Option<u64>) {
    SETTINGS.open_meteo_api.set_request_timeout(
        seconds.map(|d| Duration::from_secs(d))
    );
}