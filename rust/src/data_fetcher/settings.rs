use std::time::Duration;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Settings {
    pub open_meteo_api: OpenMeteoApiSettings,
}

pub struct OpenMeteoApiSettings {
    pub request_timeout: Duration,
}

pub static SETTINGS: Settings = Settings {
    open_meteo_api: OpenMeteoApiSettings {
        request_timeout: DEFAULT_REQUEST_TIMEOUT,
    },
};
