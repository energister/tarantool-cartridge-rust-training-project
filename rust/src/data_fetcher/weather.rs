use serde::Deserialize;
use crate::data_fetcher;
use crate::data_fetcher::dto;
use crate::data_fetcher::settings::SETTINGS;

#[derive(Deserialize, Debug)]
struct MeteoApiWeatherResponse {
    utc_offset_seconds: i32,
    current: MeteoApiCurrentWeather,
}

#[derive(Deserialize, Debug)]
struct MeteoApiCurrentWeather {
    time: String,
    interval: i64,
    temperature: f64,
}

pub fn get_weather(latitude: f64, longitude: f64) -> Result<Option<dto::Weather>, Box<dyn std::error::Error>> {
    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature", latitude, longitude);

    let client = fibreq::ClientBuilder::new().build();
    let http_req = client.get(&url)?;
    let response_result = http_req
        .request_timeout(SETTINGS.open_meteo_api.get_request_timeout())
        .send();

    let response = data_fetcher::handle_errors("weather", &url, response_result)?;
    // TODO: refactor: reduce copy-paste by moving the following code into data_fetcher::handle_errors()
    let weather_data: Option<MeteoApiWeatherResponse> = response
        .map(|mut r| r.json())
        .transpose()
        .inspect_err(
            |e| log::error!("Failed to deserialize 'weather' response: {}. URL={}", e, &url)
        )?;

    weather_data.map(convert).transpose()
}

fn convert(data: MeteoApiWeatherResponse) -> Result<dto::Weather, Box<dyn std::error::Error>> {
    let offset = time::UtcOffset::from_whole_seconds(data.utc_offset_seconds)?;

    let point_in_time =
        // TODO: add &timeformat=unixtime to the URL to simplify parsing. See https://open-meteo.com/en/docs#api_documentation
        time::PrimitiveDateTime::parse(&data.current.time, &time::format_description::well_known::Iso8601::DATE_TIME)
            .map_err(|e: time::error::Parse| {
                log::error!("Unexpected time format in Meteo API response ({}): {}", data.current.time, e);
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unexpected error while parsing Open Meteo API response".to_string(),
                )
            })
            .map(|primitive|
                primitive.assume_offset(offset)
            )?;

    let ttl = time::Duration::seconds(data.current.interval);

    Ok(dto::Weather {
        point_in_time: point_in_time.into(),
        expiration: (point_in_time + ttl).into(),
        temperature_celsius: data.current.temperature,
    })
}