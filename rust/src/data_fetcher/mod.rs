//! data_fetcher module
//!
//! Purpose:
//! Encapsulate interactions with the remote server API (https://open-meteo.com/)

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
    let response_result = http_req
        .request_timeout(SETTINGS.open_meteo_api.get_request_timeout())
        .send();

    let mut response = match response_result {
        Ok(resp) => resp,
        Err(e) => {
            return if matches!(*e, fibreq::Error::Timeout) {
                log::debug!("Timeout while fetching 'coordinates'");
                Ok(None)
            } else {
                Err(Box::new(e))
            }
        }
    };

    if response.status() != 200 {
        return handle_fail("coordinates", &response);
    }

    let geo_data: GeocodingResponse = response.json()?;

    let first_result = geo_data.results.as_ref()
        .and_then(|results| results.first());

    Ok(Some(
        first_result
            .map(|result|
                dto::Coordinates {
                    latitude: Some(result.latitude),
                    longitude: Some(result.longitude),
                }
            )
            .unwrap_or(
                // place not found
                dto::Coordinates {
                    latitude: None,
                    longitude: None,
                }
            )
    ))
}

fn handle_fail(context: &str, response: &Response) -> Result<Option<dto::Coordinates>, Box<dyn std::error::Error>> {
    if response.status() == 408 /* Request Timeout */ ||
        response.status() == 503 /* Service Unavailable */ {
        log::debug!("Timeout while fetching '{}': HTTP_status_code={}", context, response.status());
        Ok(None)
    } else {
        log::error!("Failed to fetch '{}': HTTP_status_code={}, URL={}", context, response.status(), response.url());
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to fetch '{}' from Open Meteo API: HTTP {}", context, response.status()),
        )))
    }
}
