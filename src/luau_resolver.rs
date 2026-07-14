// Some types are explicitly stated: mlua::Result, mlua::Value
// This is to avoid some confusion

use mlua::prelude::*;
use std::{
    path::{Path, PathBuf},
    cell::{RefCell},
    collections::{HashMap},
};

struct RequireState {
    stack: Vec<PathBuf>, // Currently executing chunk
    cache: HashMap<PathBuf, mlua::Value> // Path and result
}

fn resolve_module_path(caller_dir: &PathBuf, req: &str) -> mlua::Result<PathBuf>{
    let full_path = caller_dir.join(req);
    let candidates = [
        full_path.clone(),
        full_path.with_extension("luau"),
        full_path.with_extension("lua"),
        full_path.join("init.luau"),
        full_path.join("init.lua"),
    ];

    for c in candidates {
        if c.is_file() {
            return c.canonicalize().map_err(mlua::Error::external);
        }
    }

    Err(
        mlua::Error::RuntimeError(
            format!("Module {req} not found relative to '{}'.", caller_dir.display())
        )
    )
}

fn resolve_and_run(luau: &Lua, req: &str) -> mlua::Result<mlua::Value>{
    // Get RequireState -> Borrow it -> Get most recently added PathBuf -> Clone it -> Fall back if
    // failed
    /*
    let caller_dir = luau.app_data_ref::<RefCell<RequireState>>()
        .and_then(|state| state.borrow())
        .and_then(|state_ref| state_ref.stack.last().cloned())
        .unwrap_or_else(|| PathBuf::from("."));
    */
    let caller_dir = {
        let state = luau.app_data_ref::<RefCell<RequireState>>().unwrap();
        state.borrow().stack.last().cloned().unwrap_or_else(|| PathBuf::from("."))
    };

    let resolved = resolve_module_path(&caller_dir, req)?;

    // This module has already been loaded (cached)
    if let Some(lua_val) = luau
        .app_data_ref::<RefCell<RequireState>>()
        .unwrap()
        .borrow()
        .cache.get(&resolved)
    {
        return Ok(lua_val.clone());
    }

    let src = std::fs::read_to_string(&resolved).map_err(mlua::Error::external)?;
    let chunk_name = resolved.to_string_lossy().to_string();
    let module_dir = resolved.parent().unwrap_or(Path::new(".")).to_path_buf();

    // Push in, so if this module contains require(), it will continue from where it's at.
    luau.app_data_ref::<RefCell<RequireState>>()
        .unwrap()
        .borrow_mut()
        .stack.push(module_dir);

    let result = luau.load(&src).set_name(chunk_name).eval();

    // Pop, since we are done with this current module.
    luau.app_data_ref::<RefCell<RequireState>>()
        .unwrap()
        .borrow_mut()
        .stack.pop();

    let value: mlua::Value = result?;

    // Cache the returned value, so other modules can use later if needed.
    luau.app_data_ref::<RefCell<RequireState>>()
        .unwrap()
        .borrow_mut()
        .cache.insert(resolved, value.clone());

    Ok(value)
}

pub fn setup_require(luau: &Lua, entry_dir: PathBuf) -> mlua::Result<()> {
    // Only 1 object of each type
    // Type = RefCell
    luau.set_app_data(
        RefCell::new(
            RequireState {
                stack:vec![entry_dir],
                cache: HashMap::new(),
            }
        )
    );

    let require_fn = luau.create_function(|luau, path: String| {
        resolve_and_run(luau, &path)
    })?;
    luau.globals().set("require", require_fn);

    Ok(())
}
