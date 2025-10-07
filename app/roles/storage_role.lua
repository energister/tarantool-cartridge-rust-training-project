local storage = require('app.storage')
local log = require('log')

local function init(opts) -- luacheck: no unused args
    -- if opts.is_master then
    -- end

    return true
end

local function stop()
    return true
end

local function validate_config(conf_new, conf_old) -- luacheck: no unused args
    return true
end

local function apply_config(conf, opts) -- luacheck: no unused args
    -- if opts.is_master then
    -- end

    return true
end

storage_api = {}

function storage_api.place_get(place_name)
    log.info("got request for %s place", place_name)
    return nil
end

return {
    role_name = 'app.roles.storage',
    init = init,
    stop = stop,
    validate_config = validate_config,
    apply_config = apply_config,
    dependencies = {'cartridge.roles.vshard-storage'},
}