use crate::commands::Command;
use crate::kernel::unit_of_work::UnitOfWork;
use crate::kernel::Kernel;

pub struct EchoCmd;
impl Command for EchoCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        let text = args.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
        uow.print(text);
    }
}

pub struct GrepCmd;
impl Command for GrepCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        if args.len() < 3 {
            uow.print("usage: grep pattern file...");
            return;
        }
        let pattern = args[1].to_lowercase();
        let files: Vec<&str> = args
            .iter()
            .skip(2)
            .filter(|a| !a.starts_with('-'))
            .copied()
            .collect();
        let show_filename = files.len() > 1;
        for f in &files {
            let path = uow.session.resolve_path(Some(f));
            match kernel.vfs.read_file(&path) {
                Some(content) => {
                    let content = content.to_string();
                    for line in content.lines() {
                        if line.to_lowercase().contains(&pattern) {
                            if show_filename {
                                uow.print(format!("{f}:{line}"));
                            } else {
                                uow.print(line.to_string());
                            }
                        }
                    }
                }
                None => {
                    uow.print(format!("grep: {f}: No such file or directory"));
                }
            }
        }
    }
}

pub struct HeadCmd;
impl Command for HeadCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let n: usize = args
            .windows(2)
            .find(|w| w[0] == "-n")
            .and_then(|w| w[1].parse().ok())
            .unwrap_or(10);
        let files: Vec<&str> = args
            .iter()
            .skip(1)
            .filter(|a| !a.starts_with('-'))
            .copied()
            .collect();
        for f in &files {
            let path = uow.session.resolve_path(Some(f));
            match kernel.vfs.read_file(&path) {
                Some(content) => {
                    let content = content.to_string();
                    for line in content.lines().take(n) {
                        uow.print(line.to_string());
                    }
                }
                None => {
                    uow.print(format!("head: {f}: No such file or directory"));
                }
            }
        }
    }
}

pub struct TailCmd;
impl Command for TailCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let n: usize = args
            .windows(2)
            .find(|w| w[0] == "-n")
            .and_then(|w| w[1].parse().ok())
            .unwrap_or(10);
        let files: Vec<&str> = args
            .iter()
            .skip(1)
            .filter(|a| !a.starts_with('-'))
            .copied()
            .collect();
        for f in &files {
            let path = uow.session.resolve_path(Some(f));
            match kernel.vfs.read_file(&path) {
                Some(content) => {
                    let content = content.to_string();
                    let lines: Vec<&str> = content.lines().collect();
                    let start = lines.len().saturating_sub(n);
                    for line in &lines[start..] {
                        uow.print(line.to_string());
                    }
                }
                None => {
                    uow.print(format!("tail: {f}: No such file or directory"));
                }
            }
        }
    }
}

pub struct WcCmd;
impl Command for WcCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let files: Vec<&str> = args
            .iter()
            .skip(1)
            .filter(|a| !a.starts_with('-'))
            .copied()
            .collect();
        if files.is_empty() {
            uow.print("usage: wc file...");
            return;
        }
        let count_lines = !args.contains(&"-w") && !args.contains(&"-c") || args.contains(&"-l");
        let count_words = !args.contains(&"-l") && !args.contains(&"-c") || args.contains(&"-w");
        let count_bytes = !args.contains(&"-l") && !args.contains(&"-w") || args.contains(&"-c");

        for f in &files {
            let path = uow.session.resolve_path(Some(f));
            match kernel.vfs.read_file(&path) {
                Some(content) => {
                    let content = content.to_string();
                    let lines = content.lines().count();
                    let words: usize = content.split_whitespace().count();
                    let bytes = content.len();
                    let mut parts = Vec::new();
                    if count_lines {
                        parts.push(format!("{lines:6}"));
                    }
                    if count_words {
                        parts.push(format!("{words:6}"));
                    }
                    if count_bytes {
                        parts.push(format!("{bytes:6}"));
                    }
                    parts.push(f.to_string());
                    uow.print(parts.join(" "));
                }
                None => {
                    uow.print(format!("wc: {f}: No such file or directory"));
                }
            }
        }
    }
}
