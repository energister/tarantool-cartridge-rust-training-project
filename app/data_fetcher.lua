local log = require('log')
local checks = require('checks')

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

return {
    settings = settings,
}