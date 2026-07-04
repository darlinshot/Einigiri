use std::error::Error;
use mlua::prelude::*;
use std::fs;
use std::env;

slint::slint! {
    import { Button, VerticalBox } from "std-widgets.slint";
    export component App inherits Window {
        // Properties that Luau will manipulate
        in-out property <string> status_text: "Initial State";
        in-out property <bool> is_danger: false;

        width: 300px;
        height: 150px;

        background: is_danger ? #ffcccc : #e6f7ff;

        VerticalLayout {
            padding: 20px;
            spacing: 15px;
            alignment: center;

            Text {
                text: status_text;
                font-size: 18px;
                horizontal-alignment: center;
            }

            Button {
                text: "Print thing";
                clicked => {
                    // This calls the Rust callback
                    root.mouse_clicked();
                }
            }
        }
        
        // Define the callback hook
        callback mouse_clicked();
    }
}

fn main () -> Result<(), Box<dyn Error>>{
    let config = "/home/ei/.config/einigiri/einigiri.luau";

    let luau = Lua::new();
    let lua_fn = luau.create_function(|_, name: String| {
        println!("{}", name);
        Ok(())
    })?;

    let globals = luau.globals();
    globals.set("PrintRust", lua_fn)?;

    // Run the Luau script
    let script = fs::read_to_string(config)?;
    luau.load(&script).exec()?;

    let app = App::new().unwrap();
    let app_weak = app.as_weak();

    app.on_mouse_clicked(move || {
        println!("Printing thing");
    });

    app.run();
    Ok(())
}
