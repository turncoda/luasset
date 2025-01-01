use mlua::prelude::*;
use unreal_asset::{engine_version::EngineVersion, Asset};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

fn load(_: &Lua, uasset_path_str: String) -> LuaResult<()> {
    let uasset_path = Path::new(&uasset_path_str);
    let uasset_file = File::open(uasset_path)?;
    let uexp_path = uasset_path.with_extension("uexp");
    let uexp_file = File::open(uexp_path)?;

    let asset = Asset::new(
        uasset_file,
        Some(uexp_file),
        EngineVersion::VER_UE5_1,
        None).unwrap();

    println!("{:#?}", asset);

    Ok(())
}

fn main() -> LuaResult<()> {
    let lua = Lua::new();

    let map_table = lua.create_table()?;
    map_table.set(1, "one")?;
    map_table.set("two", 2)?;

    let f = lua.create_function(load)?;

    lua.globals().set("map_table", map_table)?;
    lua.globals().set("load", f)?;

    lua.load("load('tests/ExampleLevel.umap')").exec()?;

    Ok(())
}


