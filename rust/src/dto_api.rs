use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HttpResponse {
    pub coordinates: HttpCoordinates,
    pub point_in_time: String,
    pub temperature_celsius: f64,
}

#[derive(Debug, Serialize)]
pub struct HttpCoordinates {
    pub longitude: f64,
    pub latitude: f64,
}
