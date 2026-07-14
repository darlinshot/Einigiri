use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use mlua::prelude::*;

use crate::config;
use crate::luau_resolver;

pub fn run() -> Result<(), Box<dyn Error>> {
    let app_config = config::load_config();
    for (i, v) in &app_config.fields { 
        println!("Key: {} Value: {}", i, v);
    }

    println!("Running Einigiri!");

    let luau = Lua::new();

    let test_fn = luau.create_function(move |ctx, to_print: String| -> Result<(), mlua::Error>{
        println!("{}", to_print);
        Ok(())
    })?;
    luau.globals().set("test", test_fn)?;

    let script_path = PathBuf::from("./luau/core.luau");
    let src = fs::read_to_string(&script_path)?;
    let dir = script_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    luau_resolver::setup_require(&luau, dir);
    luau.load(&src)
        .exec()?;

    Ok(())
}
