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

    let response = handle_errors(response_result)?;
    let geo_data: Option<GeocodingResponse> = response
        .map(|mut r| r.json())
        .transpose()?;

    return Ok(geo_data.map(convert));

    fn convert(geo_data: GeocodingResponse) -> dto::Coordinates {
        let first_result = geo_data.results.as_ref()
            .and_then(|results| results.first());

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
    }
}

/// Returns `None` in case of known transient errors
fn handle_errors(response: Result<Response, Box<fibreq::Error>>) -> Result<Option<Response>, Box<dyn std::error::Error>> {
    match response {
        Ok(resp) => {
            if resp.status() == 200 {
                Ok(Some(resp))
            } else {
                handle_fail("coordinates", &resp)
            }
        },
        Err(e) => {
            if matches!(*e, fibreq::Error::Timeout) {
                log::debug!("Timeout while fetching 'coordinates'");
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

fn handle_fail(context: &str, response: &Response) -> Result<Option<Response>, Box<dyn std::error::Error>> {
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
