local cartridge = require('cartridge')
local log = require('log')
local http_client = require('http.client').new()

local function http_weather(req)
    
    local place_name = req:query_param().place
    if place_name == nil or place_name == "" then
        return { status = 400, body = "'place' parameter is required" }
    end

    -- TODO Does this block TX fiber?
    local response = http_client:get('https://geocoding-api.open-meteo.com/v1/search?name=' .. place_name .. '&count=1&language=en&format=json')
    local places = response:decode()['results']
    if places == nil or #places == 0 then
        return { status = 404, body = "'"..place_name.."' not found" }
    end
    local place = places[1]
    local resp = req:render({json = { latitude = place['latitude'], longitude = place['longitude'] } })
    resp.status = 200
    return resp
end

local function init(opts) -- luacheck: no unused args
    -- if opts.is_master then
    -- end

    local httpd = assert(cartridge.service_get('httpd'), "Failed to get httpd service")
    httpd:route({method = 'GET', path = '/hello'}, function()
        log.error("Hello!")
        return {body = 'Hello world!'}
    end)

    httpd:route(
        { method = 'GET', path = '/weather'},
        http_weather
    )   

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

return {
    role_name = 'app.roles.weather',
    init = init,
    stop = stop,
    validate_config = validate_config,
    apply_config = apply_config,
    -- dependencies = {'cartridge.roles.vshard-router'},
}
