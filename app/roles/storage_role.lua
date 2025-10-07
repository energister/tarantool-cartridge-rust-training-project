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

local fake_storage = {}

function storage_api.place_get(place_name)
    log.info("got request for %s place", place_name)
    return fake_storage[place_name]
end

function storage_api.place_put(place_name, place)
    log.info("storing place %s", place_name)
    fake_storage[place_name] = place
end

return {
    role_name = 'app.roles.storage',
    init = init,
    stop = stop,
    validate_config = validate_config,
    apply_config = apply_config,
    dependencies = {'cartridge.roles.vshard-storage'},
}