local storage = require('app.storage')
local checks = require('checks')
local cartridge = require('cartridge')
local log = require('log')

local function init(opts)
    storage.create_space(opts.is_master)

    return true
end

local function get_weather_for_place(bucket_id, place_name)
    checks('number', 'string')

    local stored = storage.coordinates_get(place_name)
    if stored ~= nil then
        return { cached = true, coordinates = stored }
    end

    local coordinates, err = cartridge.rpc_call('app.roles.data_fetcher', 'get_coordinates', { place_name })
    if err ~= nil or coordinates == nil then
        log.error("Failed to perform an RPC call to the data_fetcher: %s", err)
        return nil
    end

    -- cache the response
    storage.coordinates_put(bucket_id, place_name, coordinates)

    return { cached = false, coordinates = coordinates }
end

storage_api = {
    get_weather_for_place = get_weather_for_place,
}

return {
    role_name = 'app.roles.storage',
    init = init,
    dependencies = {'cartridge.roles.vshard-storage'},
}