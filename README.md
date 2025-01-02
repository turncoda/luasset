(work-in-progress)

goal is to be able to do something like this:

```lua
-- open uasset file
my_map = uasset("original/Zone_Theatre")
donor = uasset("original/Zone_Lab")

-- get actor by index
-- automatic transplant/duplicate
actor1 = my_map:add_actor(
    donor:get_actor(6)
)

-- find any actor that passes condition lambda
-- get property via table lookup
actor_bigbox = my_map:add_actor(
    donor:find_any_actor(function(actor)
        return
            actor
            ["StaticMeshComponent"]
            ["StaticMesh"]
            .name == "blockoutBigbox"
    end)
)

-- set property value by assignment
actor_bigbox["RelativeScale3D"] = { 1, 0.9, 1.3 }

my_map:find_any_import(function(import)
    return import.name == "DT_SampleWaypointTable"
end).name = "DT_th1"

-- find all actors that pass condition lambda
for actor in my_map:find_all_actors(function(actor)
    return actor.name == "BP_SavePoint_C"
end) do
    actor:disable()
end

my_map:save("Zone_Theatre_th1")
```
