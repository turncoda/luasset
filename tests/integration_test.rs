use std::fs::File;
use std::path::Path;
use tempdir::TempDir;
use unreal_asset::engine_version::EngineVersion;
use unreal_asset::Asset;

fn bytes_match(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (a_v, b_v) in a.iter().zip(b.iter()) {
        if a_v != b_v {
            return false;
        }
    }
    true
}

fn open_uasset(p: &str) -> Asset<File> {
    let uasset_path = Path::new(p);
    let uasset_file = File::open(uasset_path).unwrap();
    let uexp_path = uasset_path.with_extension("uexp");
    let uexp_file = File::open(uexp_path).unwrap();

    Asset::new(uasset_file, Some(uexp_file), EngineVersion::VER_UE5_1, None).unwrap()
}

fn save_uasset(a: Asset<File>, p: &Path) {
    let uasset_path = Path::new(p);
    let uexp_path = uasset_path.with_extension("uexp");
    let mut uasset_file = File::create(uasset_path).unwrap();
    let mut uexp_file = File::create(uexp_path).unwrap();
    a.write_data(&mut uasset_file, Some(&mut uexp_file))
        .unwrap();
}

#[test]
fn test_noop() {
    // Rust version
    let mut expected_uasset_bytes = vec![];
    let mut expected_uexp_bytes = vec![];
    {
        let uasset = open_uasset("tests/ExampleLevel.umap");
        let mut out_uasset = std::io::Cursor::new(&mut expected_uasset_bytes);
        let mut out_uexp = std::io::Cursor::new(&mut expected_uexp_bytes);
        uasset
            .write_data(&mut out_uasset, Some(&mut out_uexp))
            .unwrap();
    }

    // Lua version
    {
        let tmp_dir = TempDir::new("luasset_test_noop").unwrap();
        let file_path = tmp_dir.path().join("ExampleLevel_noop_lua.umap");
        let out_uasset_path = file_path.clone();
        let out_uexp_path = file_path.with_extension("uexp");
        if let Err(e) = luasset::run_script(mlua::chunk! {
            my_asset = uasset("tests/ExampleLevel.umap")
            my_asset:save($file_path)
        }) {
            println!("{}", e);
            assert!(false);
        }

        let out_uasset_bytes = std::fs::read(out_uasset_path).unwrap();
        let out_uexp_bytes = std::fs::read(out_uexp_path).unwrap();

        assert!(bytes_match(&out_uasset_bytes, &expected_uasset_bytes));
        assert!(bytes_match(&out_uexp_bytes, &expected_uexp_bytes));
    }
}
