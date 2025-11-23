-- Role: data_fetcher
-- Purpose: Encapsulate interactions with the remote server API (https://open-meteo.com/)
local luaopen_rust = require('librust')
local rust = require('app.rust')
local log = require('log')

local CFG_FILE_NAME = 'custom_config' -- custom_config.yml
local CFG_SECTION_NAME = 'open_meteo_api'
local CFG_REQUEST_TIMEOUT_OPTION_NAME = 'request_timeout_in_seconds'

local function init(opts)
    rust.load("librust", { "init_rpc_server", "rpc_handler", "set_request_timeout_in_seconds" })

    assert(rust.init_rpc_server())
end

local function validate_config(conf_new, conf_old) -- luacheck: no unused args
    local timeout = ((conf_new[CFG_FILE_NAME] or {})[CFG_SECTION_NAME] or {})[CFG_REQUEST_TIMEOUT_OPTION_NAME]

    --[[ validate_config() is called too early during lifecycle,
    so it's impossible to call Rust via `box.func['<fn>']:call()`
    because it leads to the "Please call box.cfg{} first" error ]]
    local valid = luaopen_rust.data_fetcher.validate_request_timeout_in_seconds(timeout)

    if not valid then
        local option_path = CFG_FILE_NAME .. '.' .. CFG_SECTION_NAME .. '.' .. CFG_REQUEST_TIMEOUT_OPTION_NAME
        log.info("Invalid %s value: %s", option_path, tostring(timeout))
        return nil, option_path .. " must be a non-negative number"
    end

    return true
end

local function apply_config(conf, opts) -- luacheck: no unused args
    local timeout = ((conf[CFG_FILE_NAME] or {})[CFG_SECTION_NAME] or {})[CFG_REQUEST_TIMEOUT_OPTION_NAME]

    assert(rust.set_request_timeout_in_seconds(timeout or box.NULL))

    return true
end

return {
    role_name = 'app.roles.data_fetcher',
    init = init,
    validate_config = validate_config,
    apply_config = apply_config,
    rpc_handler = function(path, ctx, mp_request)
        return rust.rpc_handler(path, ctx, mp_request)
    end
}