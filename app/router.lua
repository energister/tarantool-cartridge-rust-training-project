local http_client = require('http.client').new()

local REQUEST_TIMEOUT_IN_SECONDS = 1

local function http_get_weather(req)

    local place_name = req:query_param().place
    if place_name == nil or place_name == "" then
        return { status = 400, body = "'place' parameter is required" }
    end

    -- TODO Does this block TX fiber?
    local response = http_client:get('https://geocoding-api.open-meteo.com/v1/search?name=' .. place_name .. '&count=1&language=en&format=json', 
        {timeout = REQUEST_TIMEOUT_IN_SECONDS})
    local places = response:decode()['results']
    if places == nil or #places == 0 then
        return { status = 404, body = "'"..place_name.."' not found" }
    end

    local place = places[1]
    return req:render({json = { latitude = place['latitude'], longitude = place['longitude'] } })
end

return {
    http_get_weather = http_get_weather,
}