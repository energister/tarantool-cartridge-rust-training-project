use tarantool::ffi::lua as ffi_lua;
use tarantool::tlua;
use tarantool::tlua::AsLua;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_librust(l: *mut ffi_lua::lua_State) -> i32 {
    unsafe {
        let lua = tlua::StaticLua::from_static(l);

        shors::init_lua_functions(&lua).unwrap();

        let table = tlua::AsTable((
            ("init_router", tlua::Function::new(init_router)),
            ("foo", "bar"),
        ));
        let guard = (&lua).push(table);
        guard.forget()
    }
}

fn init_router() -> bool {
    true
}
