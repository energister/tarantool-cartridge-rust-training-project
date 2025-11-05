use serde::Deserialize;
use crate::data_fetcher;
use crate::data_fetcher::dto;
use crate::data_fetcher::settings::SETTINGS;

#[derive(Deserialize, Debug)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Deserialize, Debug)]
struct GeocodingResult {
    latitude: f64,
    longitude: f64,
}

pub fn get_coordinates(place_name: String) -> Result<Option<dto::Coordinates>, Box<dyn std::error::Error>> {
    let url = format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json", place_name);

    let client = fibreq::ClientBuilder::new().build();
    let http_req = client.get(&url)?;
    let response_result = http_req
        .request_timeout(SETTINGS.open_meteo_api.get_request_timeout())
        .send();

    let response = data_fetcher::handle_errors("coordinates", &url, response_result)?;
    let geo_data: Option<GeocodingResponse> = response
        .map(|mut r| r.json())
        .transpose()
        .inspect_err(
            |e| log::error!("Failed to deserialize 'coordinates' response: {}. URL={}", e, &url)
        )?;

    Ok(geo_data.map(convert))
}

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