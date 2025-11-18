local t = require('luatest')
local apply_config = t.group('data_fetcher_role.apply_config')
local validate_config = t.group('data_fetcher_role.validate_config')

local yaml = require('yaml')

local data_fetcher_role = require('app.roles.data_fetcher_role')

local OPTS_ARGUMENT = { is_master = true }

local REQUEST_TIMEOUT_IN_SECONDS_DEFAULT = 5
local get_request_timeout_in_seconds = function()
    return box.func['librust.get_request_timeout_in_seconds']:call()
end

t.before_suite(function()
    data_fetcher_role.init(OPTS_ARGUMENT)
end)

apply_config.test_default_on_start = function()
    local custom_config = { }

    data_fetcher_role.apply_config(custom_config, OPTS_ARGUMENT)

    t.assert_equals(get_request_timeout_in_seconds(), REQUEST_TIMEOUT_IN_SECONDS_DEFAULT)
end

apply_config.test_apply_config = function()
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            request_timeout_in_seconds: 15
    ]])

    data_fetcher_role.apply_config(custom_config, OPTS_ARGUMENT)

    t.assert_equals(get_request_timeout_in_seconds(), 15)
end

apply_config.test_remove_the_option_or_config = function()
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            # request_timeout_in_seconds option is removed
            some_other_option: 123
    ]])

    data_fetcher_role.apply_config(custom_config, OPTS_ARGUMENT)

    -- become default again
    t.assert_equals(get_request_timeout_in_seconds(), REQUEST_TIMEOUT_IN_SECONDS_DEFAULT)
end


validate_config.test_some_value = function()
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            request_timeout_in_seconds: 28
    ]])

    local result = data_fetcher_role.validate_config(custom_config, {})

    t.assert_eval_to_true(result)
end

validate_config.test_no_option = function()
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            # no request_timeout_in_seconds option
            some_other_option: 123
    ]])

    local result = data_fetcher_role.validate_config(custom_config, {})

    t.assert_eval_to_true(result)
end

validate_config.test_negative_value = function()
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            request_timeout_in_seconds: -1
    ]])

    local result = data_fetcher_role.validate_config(custom_config, {})

    t.assert_eval_to_false(result)
end

validate_config.test_not_a_number = function()
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            request_timeout_in_seconds: "abc"
    ]])

    local result = data_fetcher_role.validate_config(custom_config, {})

    t.assert_eval_to_false(result)
end

validate_config.test_non_integer = function()
    local custom_config = yaml.decode([[
    custom_config:
        open_meteo_api:
            request_timeout_in_seconds: 3.5
    ]])

    local result = data_fetcher_role.validate_config(custom_config, {})

    t.assert_eval_to_false(result)
end