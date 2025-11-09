use shors::transport::http::route::Builder;
use shors::transport::http::{Request, Response};
use shors::transport::Context;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use tarantool::datetime::Datetime;
use tarantool::lua_state;
use tarantool::tlua::{as_table, LuaFunction};
use crate::storage;
use crate::dto_api;
use time::format_description::well_known::Rfc3339;

pub fn init_router() -> bool {
    let get_weather_route = Builder::new()
        .with_method("GET")
        .with_path("/weather")
        .build(
            |_ctx: &mut Context, request: Request| -> Result<Response, Box<dyn Error>> {
                do_handle_request(&request)
                    .or_else(|e| {
                        Ok(Response {
                            status: e.http_code,
                            headers: HashMap::new(),
                            body: e.body.into_bytes(),
                        })
                    })
            });

    let server = shors::transport::http::server::Server::new();
    server.register(Box::new(get_weather_route));

    true
}

#[derive(Debug)]
struct FailureHttpResponse {
    pub http_code: u32,
    pub body: String,
}
impl FailureHttpResponse {
    pub fn new(http_code: u32, body: impl Into<String>) -> Self {
        FailureHttpResponse {
            http_code,
            body: body.into(),
        }
    }
}

fn do_handle_request(request: &Request) -> Result<Response, FailureHttpResponse> {
    let place_name = extract_place_parameter(request)?;
    let bucket_id = calculate_bucket_id(&place_name)?;
    let response: Option<storage::dto::StorageResponse> = call_storage(&bucket_id, &place_name)?;
    log::debug!("Response from storage: {:#?}", &response);
    Ok(convert_to_http_response(&place_name, &response))?
}

fn extract_place_parameter(request: &Request) -> Result<String, FailureHttpResponse> {
    let place = form_urlencoded::parse(request.query.as_bytes())
        .find(|(key, _)| key == "place")
        .map(|(_, value)| value.to_string());

    place.ok_or(FailureHttpResponse::new(400, "'place' parameter is required"))
}

fn calculate_bucket_id(place_name: &String) -> Result<u32, FailureHttpResponse> {
    lua_state().eval_with(
        "return require('vshard').router.bucket_id_strcrc32('...')",
        place_name,
    ).map_err(
        |e| {
            log::error!("Error while calculating bucket_id: {}", e);
            FailureHttpResponse::new(500, "Unexpected error while querying cache")
        }
    )
}

fn call_storage(bucket_id: &u32, place_name: &String) -> Result<Option<storage::dto::StorageResponse>, FailureHttpResponse> {
    let lua = lua_state();

    // TODO: make permanent (see shors call_shard as example)
    let rpc_function: LuaFunction<_> = lua
        .eval(r#"
            return function(bucket_id, function_name, arguments_as_table)
                require('checks').checks('number', 'string', 'table')
                local storage_response, err = require('vshard').router.callrw(bucket_id, function_name, arguments_as_table, {timeout = 5})
                -- require('log').error("⚠️: %s", type(err))

                --[[ Don't use assert() here, because storage_response might be nil,
                which should be processed as a special case (an error that is already handled in the storage)
                ]]
                if err ~= nil then
                    error(err)
                end

                -- require('log').error("👀: %s", require('json').encode(storage_response))
                return storage_response
            end"#,
        )
        .map_err(|e| {
            log::error!("Failed to generate vshard router call function: {}", e);
            FailureHttpResponse::new(500, "Unexpected error while querying cache")
        })?;

    let params = as_table! { bucket_id, &place_name };
    return rpc_function
        .call_with_args((bucket_id, "storage_api.get_weather_for_place", params))
        .map_err(|e| {
            log::error!("Failed to request the storage: {}", e);
            FailureHttpResponse::new(500, "Unexpected error while querying cache")
        });
}

fn convert_to_http_response(place_name: &String, storage_response: &Option<storage::dto::StorageResponse>) -> Result<Response, FailureHttpResponse> {
    let response = storage_response.as_ref().ok_or_else(||
        // got Lua nil from storage
        FailureHttpResponse::new(503, "Open Meteo API is temporarily unavailable")
    )?;

    let coordinates = response.coordinates.as_ref().ok_or_else(|| {
        FailureHttpResponse::new(404, &format!(r#"'{}' not found"#, &place_name))
    })?;

    let weather = response.weather.as_ref().ok_or_else(|| {
        let http_response = dto_api::HttpResponse {
            coordinates: dto_api::HttpCoordinates {
                latitude: coordinates.latitude,
                longitude: coordinates.longitude,
            },
            point_in_time: None,
            temperature_celsius: None,
        };
        let json = serde_json::to_string(&http_response).unwrap();
        FailureHttpResponse::new(503, json)
    })?;

    let http_response = dto_api::HttpResponse {
        coordinates: dto_api::HttpCoordinates {
            latitude: coordinates.latitude,
            longitude: coordinates.longitude,
        },
        point_in_time: Some(format_date_time(&weather.point_in_time)),
        temperature_celsius: Some(weather.temperature_celsius),
    };
    Ok(Response {
        status: 200,
        headers: HashMap::from([
            ("content-type".to_string(), "application/json; charset=utf8".to_string()),
            ("x-cache".to_string(), (if response.cached { "HIT" } else { "MISS" }).to_string()),
        ]),
        body: serde_json::to_vec(&http_response).unwrap(),
    })
}

fn format_date_time(dt: &Datetime) -> String {
    let offset: time::OffsetDateTime = dt.into_inner();
    offset.format(&Rfc3339).unwrap()
}