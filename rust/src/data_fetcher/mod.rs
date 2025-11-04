use fibreq::Response;
use serde::Deserialize;

pub mod dto;
pub mod settings;

pub use settings::SETTINGS;

pub fn get_coordinates(place_name: String) -> Result<Option<dto::Coordinates>, Box<dyn std::error::Error>> {
    let url = format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json", place_name);

    #[derive(Deserialize)]
    struct GeocodingResponse {
        results: Option<Vec<GeocodingResult>>,
    }

    #[derive(Deserialize)]
    struct GeocodingResult {
        latitude: f64,
        longitude: f64,
    }

    let client = fibreq::ClientBuilder::new().build();
    let http_req = client.get(&url)?;
    let mut response = http_req
        .request_timeout(SETTINGS.open_meteo_api.request_timeout)
        .send()?;

    if response.status() != 200 {
        return handle_fail("coordinates", &url, &response);
    }

    let json = response.text()?;
    let geo_data: GeocodingResponse = serde_json::from_str(&json)?;

    let first_result = geo_data.results.as_ref()
        .and_then(|results| results.first());

    match first_result {
        Some(result) =>
            Ok(Some(dto::Coordinates {
                latitude: Some(result.latitude),
                longitude: Some(result.longitude),
            })),
        None =>  // place not found
            Ok(Some(dto::Coordinates {
                latitude: None,
                longitude: None,
            }))
    }
}

fn handle_fail(context: &str, url: &str, response: &Response) -> Result<Option<dto::Coordinates>, Box<dyn std::error::Error>> {
    if response.status() == 408 /* Request Timeout */ ||
        response.status() == 503 /* Service Unavailable */ {
        log::debug!("Failed to fetch '{}': HTTP_status_code={}", context, response.status());
        Ok(None)
    } else {
        log::error!("Failed to fetch '{}': HTTP_status_code={}, URL={}", context, response.status(), url);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to fetch '{}' from Open Meteo API: HTTP {}", context, response.status()),
        )))
    }
}
