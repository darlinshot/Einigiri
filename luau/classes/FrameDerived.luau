local Frame = require("@core/./classes/Frame")

local Derived = {}
Derived.__index = Derived
Derived.Name = "Derived"
print(Frame.Name)

return Derived
