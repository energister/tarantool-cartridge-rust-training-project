use crate::dto_data_fetcher;

pub fn get_coordinates(place_name: String) -> dto_data_fetcher::Coordinates {
    dto_data_fetcher::Coordinates {
        latitude: 52.52437,
        longitude: 13.41053,
    }
}