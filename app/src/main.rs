use engine::ShellEngine;

fn main() {
    let mod_source = "mod/shell-quest/";

    match ShellEngine::new(mod_source) {
        Ok(engine) => {
            println!(
                "ShellEngine initialized with mod source: {}",
                engine.mod_source().display()
            );
        }
        Err(error) => {
            eprintln!("Failed to initialize ShellEngine: {error}");
            std::process::exit(1);
        }
    }
}

