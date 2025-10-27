use serde::Serialize;
use shors::transport::Context;
use shors::transport::http::route::Builder;
use shors::transport::http::{Request, Response};
use std::collections::HashMap;
use std::error::Error;
use tarantool::lua_state;

pub fn init_router() -> bool {
    let get_hello_route = Builder::new()
        .with_method("GET")
        .with_path("/hello")
        .build(http_hello_handler);

    let get_weather_route = Builder::new()
        .with_method("GET")
        .with_path("/weather")
        .build(http_weather_handler);

    let server = shors::transport::http::server::Server::new();
    server.register(Box::new(get_hello_route));
    server.register(Box::new(get_weather_route));

    log::error!("👋");

    true
}

fn http_hello_handler(_ctx: &mut Context, _request: Request) -> Result<Response, Box<dyn Error>> {
    Ok(Response {
        status: 200,
        headers: HashMap::from([]),
        body: "Hello world!".as_bytes().to_vec(),
    })
}


#[derive(Serialize)]
struct WeatherResponse {
    coordinates: Coordinates,
    temperature_celsius: f64,
}
#[derive(Serialize)]
struct Coordinates {
    latitude: f64,
    longitude: f64,
}

fn http_weather_handler(_ctx: &mut Context, request: Request) -> Result<Response, Box<dyn Error>> {
    let place = form_urlencoded::parse(request.query.as_bytes())
        .find(|(key, _)| key == "place")
        .map(|(_, value)| value.to_string());

    if place.is_none() {
        return Ok(Response {
            status: 400,
            headers: HashMap::from([]),
            body: "'place' parameter is required".as_bytes().to_vec(),
        });
    }

    let place_value = place.unwrap();

    let lua = lua_state();
    lua.exec(&format!(
        "require('log').error('Hello from {}')",
        place_value
    ))
    .unwrap();

    let response = WeatherResponse {
        coordinates: Coordinates {
            latitude: 52.52437,
            longitude: 13.41053,
        },
        temperature_celsius: 20.0,
    };

    return Ok(Response::from(response))
}
