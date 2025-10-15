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
    local config = conf_new[CFG_FILE_NAME]
    if (config ~= nil) then
        local section = config[CFG_SECTION_NAME]
        if section ~= nil then
            local timeout = section[CFG_REQUEST_TIMEOUT_OPTION_NAME]
            if timeout ~= nil then
                if (type(timeout) ~= 'number' or timeout < 0) then
                    log.info("Invalid %s.%s.%s value: %s", CFG_FILE_NAME, CFG_SECTION_NAME, CFG_REQUEST_TIMEOUT_OPTION_NAME, tostring(timeout))
                    return nil, string.format("'%s.%s.%s' must be a non-negative number", CFG_FILE_NAME, CFG_SECTION_NAME, CFG_REQUEST_TIMEOUT_OPTION_NAME)
                end
            end
        end
    end

    return true
end

local function apply_config(conf, opts) -- luacheck: no unused args
    local open_meteo_api_settings = { }

    local config = conf[CFG_FILE_NAME]
    if (config ~= nil) then
        local section = config[CFG_SECTION_NAME]
        if section ~= nil then
            -- it's guaranteed by validate_config that it's a number or nil
            open_meteo_api_settings.request_timeout = tonumber(section[CFG_REQUEST_TIMEOUT_OPTION_NAME])
        end
    end

    router.settings.open_meteo_api:set_request_timeout_in_seconds(open_meteo_api_settings.request_timeout)

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
