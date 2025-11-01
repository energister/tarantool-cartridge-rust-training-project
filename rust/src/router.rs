use shors::transport::http::route::Builder;
use shors::transport::http::{Request, Response};
use shors::transport::Context;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use tarantool::datetime::Datetime;
use tarantool::lua_state;
use tarantool::tlua::{as_table, LuaFunction};
use crate::{dto_storage, dto_api};
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
    let response: dto_storage::StorageResponse = call_storage(&bucket_id, &place_name)?;
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

fn call_storage(bucket_id: &u32, place_name: &String) -> Result<dto_storage::StorageResponse, FailureHttpResponse> {
    let lua = lua_state();

    // TODO: make permanent (see shors call_shard as example)
    let rpc_function: LuaFunction<_> = lua
        .eval(r#"
            return function(bucket_id, function_name, arguments_as_table)
                require('checks').checks('number', 'string', 'table')
                local res, err = require('vshard').router.callrw(bucket_id, function_name, arguments_as_table, {timeout = 2})
                -- require('log').error("⚠️: %s", type(err))

                assert(res, err)
                if err ~= nil then
                    -- TODO: instead try to simply return both values in order not to log emtpy errors (which have already been logged in storage or data_fetcher)
                    error(err)
                end

                -- require('log').error("👀: %s", require('json').encode(res))
                return res
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
        })
        .map(|resp| {
            Ok(resp)
        })?;
}

fn convert_to_http_response(place_name: &String, storage_response: &dto_storage::StorageResponse) -> Result<Response, FailureHttpResponse> {
    let coordinates = storage_response.coordinates.as_ref().ok_or_else(|| {
        FailureHttpResponse::new(404, &format!(r#"'{}' not found"#, &place_name))
    })?;
    let weather_ref = storage_response.weather.as_ref().ok_or_else(|| {
        let msg = format!(r#"No weather for '{}'"#, place_name);
        FailureHttpResponse::new(404, msg)
    })?;
    let http_response = dto_api::HttpResponse {
        coordinates: dto_api::HttpCoordinates {
            latitude: coordinates.latitude,
            longitude: coordinates.longitude,
        },
        point_in_time: format_date_time(&weather_ref.point_in_time),
        temperature_celsius: weather_ref.temperature_celsius,
    };
    Ok(Response {
        status: 200,
        headers: HashMap::from([
            ("content-type".to_string(), "application/json; charset=utf8".to_string()),
            ("x-cache".to_string(), (if storage_response.cached { "HIT" } else { "MISS" }).to_string()),
        ]),
        body: serde_json::to_vec(&http_response).unwrap(),
    })
}

fn format_date_time(dt: &Datetime) -> String {
    let offset: time::OffsetDateTime = dt.into_inner();
    offset.format(&Rfc3339).unwrap()
}