use mlua::prelude::*;
use unreal_asset::{engine_version::EngineVersion, Asset};
use unreal_asset::properties::Property;
use unreal_asset::types::PackageIndex;
//use unreal_asset::exports::Export;
//use unreal_asset::exports::ExportBaseTrait;
use unreal_asset::exports::ExportNormalTrait;
use std::{
    fs::File,
    path::Path,
};

struct AssetUserData(Asset<File>);

impl mlua::UserData for AssetUserData {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("num_exports", |_, this, ()| {
            Ok(this.0.asset_data.exports.len())
        });
        methods.add_method("index_is_valid", |_, this, (index,): (i32,)| {
            match this.0.get_export(PackageIndex::new(index)) {
                Some(_) => Ok(true),
                None => Ok(false),
            }
        });
        methods.add_method("prop_names_are_valid", |_, this, (index, prop_names): (i32, mlua::Table)| {
            let Some(export) = this.0.get_export(PackageIndex::new(index)) else {
                return Ok(false);
            };
            let Some(export) = export.get_normal_export() else {
                return Ok(false);
            };
            let first_prop_name : String = prop_names.get(1)?;
            let rest_prop_names = table_to_vec(&prop_names, 2, prop_names.len()?)?;
            for prop in &export.properties {
                if prop_name(&prop) != first_prop_name {
                    continue;
                }
                if prop_path_validation_helper(&rest_prop_names, prop) {
                    return Ok(true);
                }
            }
            Ok(false)
        });
    }
}

fn table_to_vec(table: &mlua::Table, start: i64, end: i64) -> LuaResult<Vec<String>> {
    let mut result = vec![];
    for i in start..end+1 {
        result.push(table.get(i)?);
    }
    Ok(result)
}

fn prop_name(prop: &Property) -> String {
    match prop {
        Property::StructProperty(prop) => &prop.name,
        Property::VectorProperty(prop) => &prop.name,
        _ => panic!("unhandled type"),
    }.get_owned_content()
}

fn prop_path_validation_helper(names: &[String], prop: &Property) -> bool {
    if names.len() == 0 {
        return true;
    }
    let cur_name = &names[0];
    match prop {
        Property::StructProperty(prop) => {
            for prop in &prop.value {
                if prop_name(prop).eq(cur_name) {
                    return prop_path_validation_helper(&names[1..], prop);
                }
            }
        },
        Property::VectorProperty(_) => {
            if cur_name == "x" { return true };
            if cur_name == "y" { return true };
            if cur_name == "z" { return true };
        },
        _ => panic!("unhandled type"),
    }
    false
}

fn create_uasset(lua: &Lua, name: String, userdata: AssetUserData) -> LuaResult<mlua::Table> {
    let uasset = lua.create_table()?;
    let uasset_mt: mlua::Table = lua.globals().get("uasset_mt")?;
    uasset.set_metatable(Some(uasset_mt));
    uasset.set("_userdata", userdata)?;
    uasset.set("_name", name)?;
    Ok(uasset)
}

fn uasset_ctor(lua: &Lua, (_, uasset_path_str): (mlua::Table, String)) -> LuaResult<mlua::Table> {
    let uasset_path = Path::new(&uasset_path_str);
    let uasset_file = File::open(uasset_path)?;
    let uexp_path = uasset_path.with_extension("uexp");
    let uexp_file = File::open(uexp_path)?;

    let asset = Asset::new(
        uasset_file,
        Some(uexp_file),
        EngineVersion::VER_UE5_1,
        None).unwrap();

    let name = String::from(uasset_path.file_stem().unwrap().to_string_lossy());
    create_uasset(lua, name, AssetUserData(asset))
}

fn main() -> LuaResult<()> {
    let lua = Lua::new();

    // library module
    let uasset_lib = lua.create_table()?;

    // make library module callable as a constructor
    let uasset_lib_mt = lua.create_table()?;
    uasset_lib_mt.set("__call", lua.create_function(uasset_ctor)?)?;
    uasset_lib.set_metatable(Some(uasset_lib_mt));

    // metatable to be attached to every object of type uasset
    // methods are defined on the library module
    let uasset_mt = lua.create_table()?;
    uasset_mt.set("__index", &uasset_lib)?;

    let export_mt = lua.create_table()?;
    let prop_mt = lua.create_table()?;

    lua.globals().set("uasset", &uasset_lib)?;
    lua.globals().set("uasset_mt", &uasset_mt)?;
    lua.globals().set("export_mt", &export_mt)?;
    lua.globals().set("prop_mt", &prop_mt)?;

    // prop prototype
    lua.load("
        prop_mt.__tostring = function(self)
            result = tostring(self._export)
            for i=1,#self._keys do
                result = result .. '.' .. self._keys[i]
            end
            return result
        end
    ").exec()?;

    lua.load("
        prop_mt.__index = function(self, key)
            -- make a copy of the existing keys
            local tmp_keys = {}
            for i=1,#self._keys do
                tmp_keys[i] = self._keys[i]
            end

            -- append key to copy
            tmp_keys[#tmp_keys+1] = key

            -- check if key path exists in asset userdata
            if not self._export._uasset._userdata:prop_names_are_valid(self._export._index, tmp_keys) then
                return nil
            end

            -- update self
            self._keys = tmp_keys
            return self
        end
    ").exec()?;

    // export prototype
    lua.load("
        export_mt.__tostring = function(self)
            return string.format('%s.%d', self._uasset, self._index)
        end
    ").exec()?;

    lua.load("
        export_mt.__index = function(self, key)
            if not self._uasset._userdata:prop_names_are_valid(self._index, {key}) then
                return nil
            end
            return setmetatable({_export=self, _keys={key}}, prop_mt)
        end
    ").exec()?;

    // uasset prototype
    lua.load("
        uasset.get_export = function(self, index)
            if not self._userdata:index_is_valid(index) then
                return nil
            end
            return setmetatable({_uasset=self, _index=index}, export_mt)
        end
    ").exec()?;

    lua.load("
        uasset_mt.__index = function(self, key)
            if type(key) == 'number' then
                return uasset.get_export(self, key)
            else
                return function(...) uasset[key](self, table.unpack(arg)) end
            end
        end").exec()?;

    lua.load("
        uasset_mt.__tostring = function(self)
            return string.format('%s', self._name)
        end").exec()?;

    // sample script
    lua.load("
        local my_map = uasset('tests/ExampleLevel.umap')
        print(my_map)
        print(my_map._userdata:num_exports())
        print(my_map[1])
        assert(my_map[9999] == nil) -- OOB access
        print(my_map[1].RelativeLocation)
        print(my_map[1].RelativeLocation.RelativeLocation)
        print(my_map[1].RelativeLocation.RelativeLocation.w)
        print(my_map[1].RelativeLocation.RelativeLocation.x)
        print(my_map[1].RelativeLocation.RelativeLocation.y)
        print(my_map[1].RelativeLocation.RelativeLocation.z)
        print(my_map[1].RelativeLocation.notfound)
        print(my_map[1].notfound)
    ").exec()?;

    Ok(())
}


