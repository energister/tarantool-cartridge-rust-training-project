local cartridge = require('cartridge')
local router = require('app.router')
local log = require('log')

local function init(opts) -- luacheck: no unused args
    local rust = require('librust')
    assert(rust.init_router(), "Failed to initialize librust router")

--     local httpd = assert(cartridge.service_get('httpd'), "Failed to get httpd service")
--
--     httpd:route({ method = 'GET', path = '/weather'}, router.http_get_weather)

    return true
end

return {
    role_name = 'app.roles.router',
    init = init,
    dependencies = {'cartridge.roles.vshard-router'},
}
