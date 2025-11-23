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
            "data_fetcher" => as_table! {
                "validate_request_timeout_in_seconds" => tlua::Function::new(data_fetcher::settings::validate_request_timeout_in_seconds)
            }
        };
        let guard = (&lua).push(api);
        guard.forget()
    }
}
