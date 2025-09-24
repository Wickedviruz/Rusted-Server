pub mod script_manager;
pub mod hooks;

use mlua::Lua;

pub fn run_lua_snippet() {
    let lua = Lua::new();
    lua.load(r#"print("Hello from Lua in RFS!")"#).exec().unwrap();
}
