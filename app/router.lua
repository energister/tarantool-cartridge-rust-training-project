local http_client = require('http.client').new()
local vshard = require('vshard')
local json = require('json')
local log = require('log')

local REQUEST_TIMEOUT_IN_SECONDS = 1

local function request_upstream(place_name)
    -- TODO Does this block TX fiber?
    local response = http_client:get('https://geocoding-api.open-meteo.com/v1/search?name=' .. place_name .. '&count=1&language=en&format=json', 
        {timeout = REQUEST_TIMEOUT_IN_SECONDS})
    local places = response:decode()['results']
    if places == nil or #places == 0 then
        return nil
    end

    local place = places[1]
    return { latitude = place['latitude'], longitude = place['longitude'] }
end

local function http_get_weather(req)

    local place_name = req:query_param().place
    if place_name == nil or place_name == "" then
        return { status = 400, body = "'place' parameter is required" }
    end

    local bucket_id = vshard.router.bucket_id(place_name)
    local stored, err = vshard.router.callro(bucket_id, 'storage_api.place_get', {place_name})
    if err ~= nil then
        log.error("Failed to perform a read request to the storage: %s", err)
        return { status = 500, body = 'Unexpected error while reading from storage' }
    end

    if stored ~= nil then
        error("TODO: remove this error after implementing caching")
        return { status = stored['status'], body = stored['body'], headers = { ['x-cache'] = 'HIT' } }
    end
    local miss_headers = { ['x-cache'] = 'MISS' }

    local place = request_upstream(place_name)
    if place == nil then
        return { status = 404, headers = miss_headers, body = "'"..place_name.."' not found" }
    else
        return { status = 200, headers = miss_headers, body = json.encode(place) }
    end
end

return {
    http_get_weather = http_get_weather,
}