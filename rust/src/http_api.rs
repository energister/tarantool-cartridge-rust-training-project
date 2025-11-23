use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HttpResponse {
    pub coordinates: HttpCoordinates,
    pub point_in_time: Option<String>,
    pub temperature_celsius: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct HttpCoordinates {
    pub longitude: f64,
    pub latitude: f64,
}
