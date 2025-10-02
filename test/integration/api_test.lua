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

g.test_sample = function(cg)
    local server = cg.cluster.main_server
    local response = server:http_request('post', '/admin/api', {json = {query = '{ cluster { self { alias } } }'}})
    t.assert_equals(response.json, {data = { cluster = { self = { alias = 'api' } } }})
    t.assert_equals(server.net_box:eval('return box.cfg.memtx_dir'), server.workdir)
end

g.test_metrics = function(cg)
    local server = cg.cluster.main_server
    local response = server:http_request('get', '/metrics')
    t.assert_equals(response.status, 200)
    t.assert_equals(response.reason, "Ok")
end

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