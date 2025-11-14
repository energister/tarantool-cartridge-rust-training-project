mod router;
mod dto_api;
mod data_fetcher;
mod storage;

use tarantool::ffi::lua as ffi_lua;
use tarantool::tlua::AsLua;
use tarantool::tlua;
use tarantool::log::TarantoolLogger;
use tlua::as_table;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_librust(l: *mut ffi_lua::lua_State) -> i32 {
    // setup logging
    static LOGGER: TarantoolLogger = TarantoolLogger::new();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    unsafe {
        let lua = tlua::StaticLua::from_static(l);

        shors::init_lua_functions(&lua).unwrap();

        let api = as_table! {
            "init_router" => tlua::Function::new(router::init_router),
            "data_fetcher" => as_table! {
                "get_coordinates" => tlua::Function::new(data_fetcher::coordinates::get_coordinates),
                "get_weather" => tlua::Function::new(data_fetcher::weather::get_weather),
                "set_request_timeout_in_seconds" => tlua::Function::new(data_fetcher::settings::set_request_timeout),
            },
            "storage" => as_table! {
                "create_spaces" => tlua::Function::new(storage::create_spaces),
                "get_weather_for_place" => tlua::Function::new(storage::get_weather_for_place),
            }
        };
        let guard = (&lua).push(api);
        guard.forget()
    }
}
