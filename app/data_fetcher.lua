local log = require('log')
local http_client = require('http.client')
local json = require('json')

local settings = {
    open_meteo_api = {
        REQUEST_TIMEOUT_IN_SECONDS_DEFAULT = 1
    }
}
settings.open_meteo_api.request_timeout_in_seconds = settings.open_meteo_api.REQUEST_TIMEOUT_IN_SECONDS_DEFAULT
function settings.open_meteo_api:set_request_timeout_in_seconds(timeout)
    checks('?', 'number|nil')
    self.request_timeout_in_seconds = timeout or self.REQUEST_TIMEOUT_IN_SECONDS_DEFAULT
    log.info("Set open_meteo_api.request_timeout_in_seconds to %s", tostring(self.request_timeout_in_seconds))
end

local function query_open_meteo_site(place_name)
    local response = http_client.get('https://geocoding-api.open-meteo.com/v1/search?name=' .. place_name .. '&count=1&language=en&format=json', 
        {timeout = settings.open_meteo_api.request_timeout_in_seconds})
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


return {
    settings = settings,
    request_upstream = request_upstream,
}