mod einigiri;
mod config;
mod luau_resolver;
mod wayland;
fn main() {
    let msg = einigiri::run().unwrap();
    println!("[DEBUG] Einigiri ran complete! Returned with message:\n{:#?}\n[DEBUG END]", msg);
    //println!("{:#?}", err);
}