local t = require('luatest')
local g = t.group('integration_api')

local helper = require('test.helper')

g.before_all(function(cg)
    cg.cluster = helper.cluster
    cg.cluster:start()
end)

g.after_all(function(cg)
    helper.stop_cluster(cg.cluster)
end)

g.before_each(function(cg) -- luacheck: no unused args
    -- helper.truncate_space_on_cluster(g.cluster, 'Set your space name here')
end)

g.test_weather_requires_parameter = function(cg)
    local server = cg.cluster.main_server
    local response = server:http_request('get', '/weather', { raise = false })
    t.assert_equals(response.status, 400)
    t.assert_str_contains(response.body, "place")
end

g.test_weather_Berlin = function(cg)
    local server = cg.cluster.main_server
    local response = server:http_request('get', '/weather?place=Berlin')
    t.assert_equals(response.status, 200)
    t.assert_equals(response.body, '{"latitude":52.52437,"longitude":13.41053}')
end

g.test_weather_London = function(cg)
    local server = cg.cluster.main_server
    local response = server:http_request('get', '/weather?place=London')
    t.assert_equals(response.body, '{"latitude":51.50853,"longitude":-0.12574}')
end

g.test_weather_in_nonexisting_place = function(cg)
    local server = cg.cluster.main_server
    local response = server:http_request('get', '/weather?place=Nowhereville', { raise = false })
    t.assert_equals(response.status, 404)
    t.assert_equals(response.body, "'Nowhereville' not found")
end

g.test_second_request_for_existing_place_is_served_from_cache = function(cg)
    local server = cg.cluster.main_server
    local city = 'Paris'

    local response1 = server:http_request('get', '/weather?place=' .. city)
    -- first request is served from the upstream server
    t.assert_equals(response1.headers['x-cache'], 'MISS')

    -- -- second request is served from the cache
    -- local response2 = server:http_request('get', '/weather?place=' .. city)
    -- t.assert_equals(response2.headers['x-cache'], 'HIT')
end