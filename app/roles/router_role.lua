local rust = require('librust')

local function init(opts) -- luacheck: no unused args
    assert(rust.init_router(), "Failed to initialize router")

    return true
end

return {
    role_name = 'app.roles.router',
    init = init,
    dependencies = {'cartridge.roles.vshard-router'},
}
