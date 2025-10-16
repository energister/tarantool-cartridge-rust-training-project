local checks = require('checks')

local function create_space()
    local places = box.schema.create_space(
        'place',
        {
            format = {
                {'place_name', 'string'},
                {'bucket_id', 'unsigned'},
                {'coordinates', 'map'},
            },

            -- create space only if it does not exist
            if_not_exists = true,
        }
    )

    places:create_index('primary', {
        parts = {'place_name'},
        if_not_exists = true,
    })

    -- required for vshard
    places:create_index('bucket_id', {
        parts = {'bucket_id'},
        unique = false,
        if_not_exists = true,
    })
end

local function place_get(place_name)
    local place = box.space.place:get(place_name)
    if place then
        return place.coordinates
    else
        return nil
    end
end

local function place_put(bucket_id, place_name, coordinates)
    checks('number', 'string', 'table')
    local storable_coordinates = setmetatable(coordinates, { __serialize = "map" })
    box.space.place:insert({ place_name, bucket_id, storable_coordinates })
    return true
end


return {
    create_space = create_space,
    place_get = place_get,
    place_put = place_put,
}