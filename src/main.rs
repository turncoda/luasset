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

    let uasset_table = lua.create_table()?;
    for (i, export) in asset.asset_data.exports.iter().enumerate() {
        let export_table = lua.create_table()?;
        let name = export.get_base_export().object_name.get_owned_content();
        export_table.set("_name", name)?;
        for prop in &export.get_normal_export().unwrap().properties {
            if let Property::StructProperty(prop) = prop {
                export_table.set(prop.name.get_owned_content(), 42)?;
            }
        }
        // TODO set to actor handle/table
        uasset_table.set(i+1, export_table)?;
    }
    Ok(uasset_table)
}

fn main() -> LuaResult<()> {
    let lua = Lua::new();

    let map_table = lua.create_table()?;
    map_table.set(1, "one")?;
    map_table.set("two", 2)?;

    let f = lua.create_function(load)?;

    lua.globals().set("map_table", map_table)?;
    lua.globals().set("load", f)?;

    lua.load("
        local my_map = load('tests/ExampleLevel.umap')
        print(my_map[1])
        print(my_map[1]._name)
        print(my_map[1].RelativeLocation)
    ").exec()?;

    Ok(())
}


