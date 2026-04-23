use crate::cargo::CargoCommand;
use crate::cli::BenchArgs;
use anyhow::Result;
use std::path::Path;

pub fn run(workspace_root: &Path, args: &BenchArgs) -> Result<()> {
    let scenario = args.scenario.as_deref().unwrap_or("standard");
    let duration = args.duration.unwrap_or(match scenario {
        "quick" => 2.0,
        "extended" => 10.0,
        _ => 5.0,
    });

    let mod_source = format!("mods/{}/", args.mod_name);

    println!(
        "Benchmark: {} scenario ({:.1}s per combo)",
        scenario, duration
    );
    println!("Mod: {}", args.mod_name);

    if let Some(ref combo) = args.combo {
        println!("Single combo: {}", combo);
        run_single_combo(workspace_root, &mod_source, duration, combo)?;
    } else {
        println!("Running all 9 flag combinations...\n");
        run_all_combos(workspace_root, &mod_source, duration)?;
    }

    Ok(())
}

fn run_single_combo(
    workspace_root: &Path,
    mod_source: &str,
    duration: f32,
    combo: &str,
) -> Result<()> {
    let mut cmd = CargoCommand::new("app")
        .profile("release")
        .feature("app/software-backend")
        .app_arg("--mod-source")
        .app_arg(mod_source)
        .app_arg("--bench")
        .app_arg(duration.to_string())
        .app_arg("--skip-splash");

    for flag in combo.split_whitespace() {
        cmd = cmd.app_arg(format!("--{}", flag));
    }

    let status = cmd.exec(workspace_root)?;

    if !status.success() {
        anyhow::bail!("benchmark failed");
    }

    Ok(())
}

fn run_all_combos(workspace_root: &Path, mod_source: &str, duration: f32) -> Result<()> {
    let combos = vec![
        ("baseline", vec![]),
        ("opt-comp", vec!["opt-comp"]),
        ("opt-present", vec!["opt-present"]),
        ("opt-diff", vec!["opt-diff"]),
        ("opt-skip", vec!["opt-skip"]),
        ("opt-async", vec!["opt-async"]),
        ("opt-comp+diff", vec!["opt-comp", "opt-diff"]),
        ("opt-comp+present", vec!["opt-comp", "opt-present"]),
        ("all-opt", vec!["opt"]),
    ];

    println!("Building release binary first...");
    let build_status = std::process::Command::new("cargo")
        .args([
            "build",
            "--release",
            "-p",
            "app",
            "--features",
            "app/software-backend",
        ])
        .current_dir(workspace_root)
        .status()?;

    if !build_status.success() {
        anyhow::bail!("build failed");
    }

    println!("\nRunning benchmarks:\n");

    for (name, flags) in &combos {
        print!("{:20} ... ", name);
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut cmd = CargoCommand::new("app")
            .profile("release")
            .feature("app/software-backend")
            .app_arg("--mod-source")
            .app_arg(mod_source)
            .app_arg("--bench")
            .app_arg(duration.to_string())
            .app_arg("--skip-splash");

        for flag in flags {
            cmd = cmd.app_arg(format!("--{}", flag));
        }

        let status = cmd.exec(workspace_root)?;

        if status.success() {
            println!("done");
        } else {
            println!("FAILED");
        }
    }

    println!("\nBenchmark reports saved to reports/benchmark/");

    Ok(())
}
