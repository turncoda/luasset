use mlua::prelude::*;
use unreal_asset::{engine_version::EngineVersion, Asset};
use unreal_asset::properties::Property;
use unreal_asset::exports::Export;
use unreal_asset::exports::ExportBaseTrait;
use unreal_asset::exports::ExportNormalTrait;
use std::{
    fs::File,
    path::Path,
};

fn create_uasset(lua: &Lua) -> LuaResult<mlua::Table> {
    let uasset = lua.create_table()?;
    let uasset_metatable: mlua::Table = lua.globals().get("uasset_metatable")?;
    uasset.set_metatable(Some(uasset_metatable));
    Ok(uasset)
}

fn load(lua: &Lua, uasset_path_str: String) -> LuaResult<mlua::Table> {
    let uasset_path = Path::new(&uasset_path_str);
    let uasset_file = File::open(uasset_path)?;
    let uexp_path = uasset_path.with_extension("uexp");
    let uexp_file = File::open(uexp_path)?;

    let asset = Asset::new(
        uasset_file,
        Some(uexp_file),
        EngineVersion::VER_UE5_1,
        None).unwrap();

    let uasset = create_uasset(lua)?;
    for (i, export) in asset.asset_data.exports.iter().enumerate() {
        let export_table = lua.create_table()?;
        let name = export.get_base_export().object_name.get_owned_content();
        export_table.set("_name", name)?;
        for prop in &export.get_normal_export().unwrap().properties {
            if let Property::StructProperty(prop) = prop {
                export_table.set(prop.name.get_owned_content(), 42)?;
            }
        }
        uasset.set(i+1, export_table)?;
    }
    Ok(uasset)
}

fn main() -> LuaResult<()> {
    let lua = Lua::new();

    // library module
    let uasset_lib = lua.create_table()?;
    uasset_lib.set("load", lua.create_function(load)?)?;

    // metatable to be attached to every object of type uasset
    let uasset_metatable = lua.create_table()?;
    uasset_metatable.set("__index", &uasset_lib)?;

    lua.globals().set("uasset", &uasset_lib)?;
    lua.globals().set("uasset_metatable", &uasset_metatable)?;

    // uasset prototype definition
    lua.load("
        uasset.add_actor = function(uasset, actor)
            uasset[#uasset+1] = actor
        end").exec()?;

    lua.load("
        uasset.get_actor = function(uasset, index)
            return uasset[index]
        end").exec()?;

    // sample script
    lua.load("
        local my_map = uasset.load('tests/ExampleLevel.umap')
        print(my_map:get_actor(1))
        print(my_map[1])
        print(my_map[1]._name)
        print(my_map[1].RelativeLocation)
        print(#my_map)
        print(my_map:add_actor(2))
        print(#my_map)
    ").exec()?;

    Ok(())
}


