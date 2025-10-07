local http_client = require('http.client')
local vshard = require('vshard')
local json = require('json')
local log = require('log')

local REQUEST_TIMEOUT_IN_SECONDS = 1

local function query_open_meteo_site(place_name)
    local response = http_client.get('https://geocoding-api.open-meteo.com/v1/search?name=' .. place_name .. '&count=1&language=en&format=json', 
        {timeout = REQUEST_TIMEOUT_IN_SECONDS})
    local places = response:decode()['results']
    if places == nil or #places == 0 then
        return nil
    end

    local place = places[1]
    return { latitude = place['latitude'], longitude = place['longitude'] }
end

local function request_upstream(place_name)
    local place = query_open_meteo_site(place_name)
    if place == nil then
        return { status = 404, body = "'"..place_name.."' not found" }
    else
        return { status = 200, body = json.encode(place) }
    end
end

local function with_cache_header(http_response, x_cache_header_value)
    local http_response_with_cache = { headers = { ['x-cache'] = x_cache_header_value }}
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
        return with_cache_header(stored, 'HIT')
    end

    local response = request_upstream(place_name)

    -- store the response in the storage
    local _, err = vshard.router.callrw(bucket_id, 'storage_api.place_put', {place_name, response})
    if err ~= nil then
        log.error("Failed to perform a write request to the storage: %s", err)
        return { status = 500, body = 'Unexpected error while writing to storage' }
    end

    return with_cache_header(response, 'MISS')
end

return {
    http_get_weather = http_get_weather,
}