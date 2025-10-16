local vshard = require('vshard')
local log = require('log')
local cartridge = require('cartridge')
local fiber = require('fiber')

local function with_cache_header(x_cache_header_value, http_response)
    checks('string', 'table')
    local http_response_with_cache = { headers = { ['x-cache'] = x_cache_header_value }}
    -- copy http_response to the http_response_with_cache
    for k, v in pairs(http_response) do
        http_response_with_cache[k] = v
    end
    return http_response_with_cache
end

local function http_get_weather(req)
    local place_name = req:query_param().place
    if place_name == nil or place_name == "" then
        return { status = 400, body = "'place' parameter is required" }
    end

    local bucket_id = vshard.router.bucket_id_strcrc32(place_name)
    local stored, err = vshard.router.callro(bucket_id, 'storage_api.place_get', {place_name})
    if err ~= nil then
        log.error("Failed to perform a read request to the storage: %s", err)
        return { status = 500, body = 'Unexpected error while reading from storage' }
    end

    if stored ~= nil then
        return with_cache_header('HIT', stored)
    end

    local response, err = cartridge.rpc_call('app.roles.data_fetcher', 'request_upstream', { place_name })
    if err ~= nil then
        log.error("Failed to perform an RPC call to the data_fetcher: %s", err)
        return { status = 500, body = 'Unexpected error while fetching data from upstream server' }
    end

    -- store the response in the storage asynchronously
    fiber.create(function()
        local _, err = vshard.router.callrw(bucket_id, 'storage_api.place_put', { bucket_id, place_name, response })
        if err ~= nil then
            log.error("Failed to perform a write request to the storage: %s", err)
        end
    end)

    return with_cache_header('MISS', response)
end

return {
    http_get_weather = http_get_weather,
}