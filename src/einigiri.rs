use std::error::Error;
use mlua::prelude::*;
use std::fs;

use crate::config;

pub fn run() -> Result<(), Box<dyn Error>> {
    let settings = config::load_config();
    for (i, v) in settings.fields { 
        println!("Key: {} Value: {}", i, v);
    }
    let projectPath = "";

    println!("Running Einigiri!");

    let luau = Lua::new();

    /*
    luau.create_function(move |_, modulePath: String| {
       let fileName = if modulePath.ends_with(".luau") { fileName }
       else {
           format!("{}.luau", modulePath)
       };
       let filePath = 
        
        Ok(())
    })?;
    */

    let script = fs::read_to_string("/home/ei/Projects/einigiri/luau/core.luau")?;
    /*
    let package: LuaTable = globals.get("package")?;
    let path: String = package.get("path")?;
    println!("{}", path);
    */
    luau.load(&script)
        .exec()?;

    Ok(())
}
