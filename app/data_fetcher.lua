local log = require('log')
local http_client = require('http.client')

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

local function get_coordinates(place_name)
    checks('string')
    local response = http_client.get('https://geocoding-api.open-meteo.com/v1/search?name=' .. place_name .. '&count=1&language=en&format=json',
        {timeout = settings.open_meteo_api.request_timeout_in_seconds})
    local places = response:decode()['results']
    if places == nil or #places == 0 then
        return {}
    end

    local place = places[1]
    return { latitude = place['latitude'], longitude = place['longitude'] }
end


return {
    settings = settings,
    get_coordinates = get_coordinates,
}