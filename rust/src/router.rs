use std::collections::HashMap;
use shors::transport::http::route::Builder;
use shors::transport::http::{Request, Response};
use shors::transport::Context;
use tarantool::{lua_state};

pub fn init_router() -> bool {
    let get_hello_route = Builder::new()
        .with_method("GET")
        .with_path("/hello")
        .build(
            |_ctx: &mut Context, _request: Request| -> Result<_, Box<dyn std::error::Error>> {
                Ok(Response {
                    status: 200,
                    headers: HashMap::from([]),
                    body: "Hello world!".as_bytes().to_vec(),
                })
            },
        );

    shors::transport::http::server::Server::new().register(Box::new(get_hello_route));

    let lua = lua_state();
    lua.exec("require('log').error('Hello')").unwrap();

    true
}

