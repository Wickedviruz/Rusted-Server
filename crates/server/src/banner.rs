use anyhow::Result;

pub fn print_banner() -> Result<()> {
    let server_name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let build_date = std::env::var("BUILD_DATE").unwrap_or_else(|_| "unknown".into());
    let target = std::env::var("BUILD_TARGET").unwrap_or_else(|_| "unknown".into());
    let git_hash = option_env!("GIT_HASH").unwrap_or("unknown");
    let rustc_version = rustc_version_runtime::version_meta().short_version_string;
    let lua = mlua::Lua::new();
    let lua_version: String = lua.load("return _VERSION").eval()?;

    println!("{} - Version {}", server_name, version);
    println!("Git commit: {}", git_hash);
    println!("Compiled with Rust {}", rustc_version);
    println!("Compiled on {} for platform {}", build_date, target);
    println!("Linked with {}", lua_version);
    println!();
    println!("A server developed by Wickedviruz & contributors");
    println!("Visit our repo for updates: https://github.com/Wickedviruz/Rusted-Server");
    println!();

    Ok(())
}
