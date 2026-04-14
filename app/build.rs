use std::process::Command;

fn main() {
    // Re-run when git state changes (new commit, staged files, etc.)
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/index");

    // Embed git describe so the binary knows its version.
    // `--always --dirty` produces e.g. "a1b2c3d" or "a1b2c3d-dirty".
    let hash = Command::new("git")
        .args(["describe", "--always", "--dirty"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| String::from("unknown"));

    println!("cargo:rustc-env=BUILD_GIT_HASH={}", hash.trim());
}
