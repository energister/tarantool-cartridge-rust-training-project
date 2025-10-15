local t = require('luatest')
local g = t.group('router_role.apply_config')

local yaml = require('yaml')

local router_role = require('app.roles.router_role')

local OPTS_ARGUMENT = { is_master = true }

local settings = require('app.router').settings.open_meteo_api

g.test_default_on_start = function() -- luacheck: no unused args
    local custom_config = { }

    router_role.apply_config(custom_config, OPTS_ARGUMENT)

    t.assert_equals(settings.request_timeout_in_seconds, settings.REQUEST_TIMEOUT_IN_SECONDS_DEFAULT)
end

g.test_apply_config = function() -- luacheck: no unused args
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            request_timeout_in_seconds: 15
    ]])

    router_role.apply_config(custom_config, OPTS_ARGUMENT)

    t.assert_equals(settings.request_timeout_in_seconds, 15)
end

g.test_remove_the_option_or_config = function() -- luacheck: no unused args
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            # request_timeout_in_seconds option is removed
            some_other_option: 123
    ]])

    router_role.apply_config(custom_config, OPTS_ARGUMENT)

    -- become default again
    t.assert_equals(settings.request_timeout_in_seconds, settings.REQUEST_TIMEOUT_IN_SECONDS_DEFAULT)
end