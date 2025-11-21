--- Module provides convenient wrappers over `box.lib` for calling Rust functions

local checks = require('checks')

local api = {}

--- Rust is only available after box.cfg
api.is_available = function()
	return type(box.cfg) ~= "function"
end

local function register_function(func_name, func)
	api[func_name] = function(...)
		local ok, result = pcall(func, ...)
		if not ok then
			return nil, result
		end

		return result
	end
end

api.load_function = function(libname, rust_func_name)
	checks('string', 'string')

	if not api.is_available() then
		error("Method must be called after box.cfg()")
	end

	local module = box.lib.load(libname)
	local func = module:load(rust_func_name)
	register_function(rust_func_name, func)
end

---Load rust lib
---@return boolean?, table?
api.load = function(libname, rust_funcs)
	checks('string', 'table')

    if not api.is_available() then
        error("Method must be called after box.cfg()")
    end

	local module = box.lib.load(libname)

	for _, func_name in ipairs(rust_funcs) do
		local func = module:load(func_name)
		register_function(func_name, func)
	end
end

return api