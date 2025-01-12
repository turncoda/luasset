use mlua::prelude::*;
fn main() -> LuaResult<()> {
    luasset::run_script(mlua::chunk! {
        local my_map = uasset("tests/ExampleLevel.umap")
        print(my_map)
        print(my_map._userdata:num_exports())
        print(my_map[1])
        assert(my_map[9999] == nil) --[[ OOB access ]]
        print(my_map[1].RelativeLocation)
        print(my_map[1].RelativeLocation.RelativeLocation)
        print(my_map[1].RelativeLocation.RelativeLocation.w)
        print(my_map[1].RelativeLocation.RelativeLocation.x)
        print(my_map[1].RelativeLocation.RelativeLocation.y)
        print(my_map[1].RelativeLocation.RelativeLocation.z)
        print(my_map[1].RelativeLocation.notfound)
        print(my_map[1].notfound)
        --[[ intended use case: duplicate actor 2 ]]
        --[===[ my_map[#my_map+1] = my_map[2] ]===]
        --[[ intended use case: transplant actor 3 from donor uasset ]]
        --[===[ my_map[#my_map+1] = donor[3] ]===]
        --[[ intended use case: update property values in actor 2 ]]
        --[===[ my_map[2].RelativeLocation.RelativeLocation.x = 5 ]===]
        --[[ invalid ]]
        --[===[ my_map[1].RelativeLocation = 1 ]===]
    })
}
