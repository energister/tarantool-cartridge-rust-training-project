local cartridge = require('cartridge')
local router = require('app.router')
local log = require('log')

local function init(opts) -- luacheck: no unused args
    -- if opts.is_master then
    -- end

    local httpd = assert(cartridge.service_get('httpd'), "Failed to get httpd service")

    httpd:route({ method = 'GET', path = '/weather'}, router.http_get_weather)

    return true
end

local function stop()
    return true
end

local CFG_FILE_NAME = 'custom_config' -- custom_config.yml
local CFG_SECTION_NAME = 'open_meteo_api'
local CFG_REQUEST_TIMEOUT_OPTION_NAME = 'request_timeout_in_seconds'

local function validate_config(conf_new, conf_old) -- luacheck: no unused args
    local timeout = conf_new[CFG_FILE_NAME]
        and conf_new[CFG_FILE_NAME][CFG_SECTION_NAME]
        and conf_new[CFG_FILE_NAME][CFG_SECTION_NAME][CFG_REQUEST_TIMEOUT_OPTION_NAME]

    if timeout ~= nil
        and (type(timeout) ~= 'number' or timeout < 0) then

        local option_path = CFG_FILE_NAME .. '.' .. CFG_SECTION_NAME .. '.' .. CFG_REQUEST_TIMEOUT_OPTION_NAME
        log.info("Invalid %s value: %s", option_path, tostring(timeout))
        return nil, option_path .. " must be a non-negative number"
    end

    return true
end

local function apply_config(conf, opts) -- luacheck: no unused args
    local timeout = conf[CFG_FILE_NAME]
        and conf[CFG_FILE_NAME][CFG_SECTION_NAME]
        and conf[CFG_FILE_NAME][CFG_SECTION_NAME][CFG_REQUEST_TIMEOUT_OPTION_NAME]

    router.settings.open_meteo_api:set_request_timeout_in_seconds(timeout)

    return true
end

return {
    role_name = 'app.roles.router',
    init = init,
    stop = stop,
    validate_config = validate_config,
    apply_config = apply_config,
    dependencies = {'cartridge.roles.vshard-router'},
}
