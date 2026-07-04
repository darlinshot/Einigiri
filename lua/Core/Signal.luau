-- TYPES

export type Connection = {
    Disconnect: (self: Connection) -> (),
}
export type Signal<T...> = {
    Connect: (self: Signal<T...>, callback: (T...) -> ()) -> Connection,
    Once: (self: Signal<T...>, callback: (T...) -> ()) -> Connection,
    Fire: (self: Signal<T...>, T...) -> (),
    Wait: (self: Signal<T...>) -> T...
}

-- CONNECTION CLASS
local Connection = {}
Connection.__index = {}

function Connection.New()
    local self = {}

    return setmetatable(self, Connection)
end

-- Make strict
setmetatable(Connection, {
	__index = function(_, key)
		error(("Attempt to get Connection::%s (not a valid member)"):format(tostring(key)), 2)
	end,
	__newindex = function(_, key)
		error(("Attempt to set Connection::%s (not a valid member)"):format(tostring(key)), 2)
	end,
})

-- SIGNAL CLASS
local Signal = {}
Signal.__index = Signal

function Signal.New<T...>(): Signal<T...>
    local self = {}
    self.Callbacks = {}

    return setmetatable(self, Signal) :: any
end

function Signal:Connect(callback: () -> any)

end

-- Make strict
setmetatable(Signal, {
	__index = function(_, key)
		error(("Attempt to get Signal::%s (not a valid member)"):format(tostring(key)), 2)
	end,
	__newindex = function(_, key)
		error(("Attempt to set Signal::%s (not a valid member)"):format(tostring(key)), 2)
	end,
})

return Signal
