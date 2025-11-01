mod router;
mod dto_storage;
mod dto_api;

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

        let table = as_table! {
            "init_router" => tlua::Function::new(router::init_router)
        };
        let guard = (&lua).push(table);
        guard.forget()
    }
}
