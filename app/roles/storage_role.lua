local rust = require('librust')

local function init(opts)
    assert(rust.storage.create_spaces(opts.is_master))

    return true
end

storage_api = {
    get_weather_for_place = rust.storage.get_weather_for_place,
}

return {
    role_name = 'app.roles.storage',
    init = init,
    dependencies = {'cartridge.roles.vshard-storage'},
}