local storage = require('app.storage')
local checks = require('checks')
local cartridge = require('cartridge')
local log = require('log')

local function init(opts) -- luacheck: no unused args
    if opts.is_master then
        storage.create_space()
    end

    return true
end

local function get_weather_for_place(bucket_id, place_name)
    checks('number', 'string')

    local coordinates = storage.place_get(place_name)
    if coordinates ~= nil then
        return { cached = true, coordinates = coordinates }
end

    local response, err = cartridge.rpc_call('app.roles.data_fetcher', 'request_upstream', { place_name })
    if err ~= nil or response == nil then
        log.error("Failed to perform an RPC call to the data_fetcher: %s", err)
        return nil
    end

    -- cache the response
    storage.place_put(bucket_id, place_name, response)

    return { cached = false, coordinates = response }
end

storage_api = {
    get_weather_for_place = get_weather_for_place,
}

return {
    role_name = 'app.roles.storage',
    init = init,
    dependencies = {'cartridge.roles.vshard-storage'},
}