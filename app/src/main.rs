use engine::ShellEngine;

fn main() {
    let mod_source =
        std::env::var("SHELL_QUEST_MOD_SOURCE").unwrap_or_else(|_| "mods/shell-quest/".to_string());

    let engine = ShellEngine::new(&mod_source).unwrap_or_else(|error| {
        eprintln!("Failed to initialize ShellEngine: {error}");
        std::process::exit(1);
    });

    println!(
        "ShellEngine initialized with mod source: {}",
        engine.mod_source().display()
    );

    if let Err(error) = engine.run() {
        eprintln!("Engine error: {error}");
        std::process::exit(1);
    }
}
