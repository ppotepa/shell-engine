use engine::ShellEngine;

fn main() {
    let mod_source = "mod/shell-quest/";

    let engine = ShellEngine::new(mod_source).unwrap_or_else(|error| {
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

