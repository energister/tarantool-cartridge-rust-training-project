local rust = require('app.rust')

local function init(opts)
    rust.load("librust", { "create_spaces", "get_weather_for_place" })

    assert(rust.create_spaces(opts.is_master))

    return true
end

storage_api = {
    get_weather_for_place = function(bucket_id, place_name)
        return rust.get_weather_for_place(bucket_id, place_name)
    end
}

return {
    role_name = 'app.roles.storage',
    init = init,
    dependencies = {'cartridge.roles.vshard-storage'},
}