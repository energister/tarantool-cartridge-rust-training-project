local rust = require('librust')
--local router = require('app.router')

local function init(opts) -- luacheck: no unused args
    assert(rust.init_router(), "Failed to initialize router")

    --assert(router.init(), "Failed to initialize router")

    return true
end

return {
    role_name = 'app.roles.router',
    init = init,
    dependencies = {'cartridge.roles.vshard-router'},
}
