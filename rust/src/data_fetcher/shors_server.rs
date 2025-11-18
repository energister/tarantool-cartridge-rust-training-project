use once_cell::unsync::Lazy;
use shors::tarantool::tuple::{FunctionArgs, FunctionCtx};
use shors::transport::rpc::server::Server;
use shors::transport::{rpc, Context};
use shors::shors_error;
use std::error::Error;
use std::os::raw::c_int;
use tarantool::log::TarantoolLogger;
use crate::data_fetcher::{coordinates, weather};

thread_local! {
    pub static RPC_SERVER: Lazy<Server> = Lazy::new(Server::new);
}

#[tarantool::proc]
fn init_rpc_server() -> tarantool::Result<()> {
    // setup logging
    static LOGGER: TarantoolLogger = TarantoolLogger::new();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let routes = rpc::route::Builder::new()
        .with_error_handler(|ctx, err| {
            shors_error!(ctx: ctx, "rpc error {}", err);
        })
        .with_middleware(|route| {
            log::debug!("got new rpc request!");
            route
        })
        .group();

    let get_coordinates_route = routes.builder().with_path("/get_coordinates").build(
        |_ctx: &mut Context, req: rpc::Request| -> Result<_, Box<dyn Error>> {
            return coordinates::get_coordinates(req.parse::<String>()?);
        },
    );
    let get_weather_route = routes.builder().with_path("/get_weather").build(
        |_ctx: &mut Context, req: rpc::Request| -> Result<_, Box<dyn Error>> {
            let (latitude, longitude) = req.parse::<(f64, f64)>()?;
            return weather::get_weather(latitude, longitude)
        },
    );

    RPC_SERVER.with(|srv| {
        srv.register(Box::new(get_coordinates_route));
        srv.register(Box::new(get_weather_route));
    });

    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_handler(ctx: FunctionCtx, args: FunctionArgs) -> c_int {
    RPC_SERVER.with(|srv| srv.handle(ctx, args))
}
