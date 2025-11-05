//! data_fetcher module
//!
//! Purpose:
//! Encapsulate interactions with the remote server API (https://open-meteo.com/)

use fibreq::Response;

pub mod dto;
pub mod settings;
pub mod coordinates;

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
