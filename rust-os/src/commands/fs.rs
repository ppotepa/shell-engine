use crate::commands::Command;
use crate::kernel::unit_of_work::UnitOfWork;
use crate::kernel::Kernel;

pub struct LsCmd;
impl Command for LsCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let long = args.contains(&"-l") || args.contains(&"-la") || args.contains(&"-al");
        let all = args.contains(&"-a")
            || args.contains(&"-la")
            || args.contains(&"-al")
            || args.contains(&"-A");
        let one = args.contains(&"-1");

        // Determine target path
        let path_arg = args.iter().skip(1).find(|a| !a.starts_with('-')).copied();
        let target = uow.session.resolve_path(path_arg);

        if !kernel.vfs.exists(&target) {
            let t = target.clone();
            uow.print(format!("ls: {t}: No such file or directory"));
            return;
        }
        if kernel.vfs.is_file(&target) {
            if long {
                if let Some(stat) = kernel.vfs.stat(&target) {
                    let name = target.split('/').next_back().unwrap_or(&target);
                    uow.print(format!(
                        "{} {:2} {:8} {:8} {:6} {} {}",
                        stat.permissions,
                        stat.links,
                        stat.owner,
                        stat.group,
                        stat.size,
                        stat.modified,
                        name
                    ));
                }
            } else {
                let name = target.split('/').next_back().unwrap_or(&target);
                uow.print(name.to_string());
            }
            return;
        }

        let mut entries = kernel.vfs.readdir(&target);
        if all {
            entries.insert(0, "..".to_string());
            entries.insert(0, ".".to_string());
        } else {
            entries.retain(|e| !e.starts_with('.'));
        }

        if long {
            // total line
            uow.print(format!("total {}", entries.len() * 2));
            for name in &entries {
                let full = if target == "/" {
                    format!("/{name}")
                } else {
                    format!("{target}/{name}")
                };
                let (perms, links, owner, group, size, modified) =
                    if let Some(stat) = kernel.vfs.stat(&full) {
                        (
                            stat.permissions.clone(),
                            stat.links,
                            stat.owner.clone(),
                            stat.group.clone(),
                            stat.size,
                            stat.modified.clone(),
                        )
                    } else if name == "." || name == ".." {
                        (
                            "drwxr-xr-x".to_string(),
                            2,
                            uow.session.user.clone(),
                            "staff".to_string(),
                            512u64,
                            "Sep 17 21:12".to_string(),
                        )
                    } else {
                        (
                            "-rw-r--r--".to_string(),
                            1,
                            uow.session.user.clone(),
                            "staff".to_string(),
                            0u64,
                            "Sep 17 21:12".to_string(),
                        )
                    };
                uow.print(format!(
                    "{} {:2} {:8} {:8} {:6} {} {}",
                    perms, links, owner, group, size, modified, name
                ));
            }
        } else if one {
            for name in &entries {
                uow.print(name.clone());
            }
        } else {
            // Compact multi-column
            let line = entries.join("  ");
            if !line.is_empty() {
                uow.print(line);
            }
        }
    }
}

pub struct CatCmd;
impl Command for CatCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let files: Vec<&str> = args
            .iter()
            .skip(1)
            .filter(|a| !a.starts_with('-'))
            .copied()
            .collect();
        if files.is_empty() {
            uow.print("cat: no file specified");
            return;
        }
        let decay = uow.quest.decay_tier();
        for f in files {
            let path = uow.session.resolve_path(Some(f));
            match kernel.vfs.read_file(&path) {
                Some(content) => {
                    let content = content.to_string();
                    for line in content.lines() {
                        uow.print(line.to_string());
                    }
                    // Tier 3 decay: 5% chance of data bleed
                    if decay >= 3 {
                        let tick = kernel.uptime_ms();
                        if tick.is_multiple_of(20) {
                            uow.print("".to_string());
                            // Extra line from another file (bleed)
                            uow.print("[data bleed from adjacent block]".to_string());
                        }
                    }
                }
                None => {
                    let p = path.clone();
                    uow.print(format!("cat: {p}: No such file or directory"));
                }
            }
        }
    }
}

pub struct CpCmd;
impl Command for CpCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        if args.len() < 3 {
            uow.print("usage: cp source dest");
            return;
        }
        let src = uow.session.resolve_path(Some(args[1]));
        let dst = uow.session.resolve_path(Some(args[2]));
        if kernel.vfs.copy_file(&src, &dst) {
            uow.quest.backup_made = true;
        } else {
            let s = src.clone();
            uow.print(format!("cp: {s}: No such file or directory"));
        }
    }
}

pub struct MvCmd;
impl Command for MvCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        if args.len() < 3 {
            uow.print("usage: mv source dest");
            return;
        }
        let src = uow.session.resolve_path(Some(args[1]));
        let dst = uow.session.resolve_path(Some(args[2]));
        if !kernel.vfs.move_file(&src, &dst) {
            let s = src.clone();
            uow.print(format!("mv: {s}: No such file or directory"));
        }
    }
}

pub struct RmCmd;
impl Command for RmCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let recursive = args.contains(&"-r") || args.contains(&"-rf") || args.contains(&"-fr");
        let files: Vec<&str> = args
            .iter()
            .skip(1)
            .filter(|a| !a.starts_with('-'))
            .copied()
            .collect();
        if files.is_empty() {
            uow.print("usage: rm [-r] file...");
            return;
        }
        for f in files {
            let path = uow.session.resolve_path(Some(f));
            if !kernel.vfs.exists(&path) {
                let p = path.clone();
                uow.print(format!("rm: {p}: No such file or directory"));
            } else if kernel.vfs.is_dir(&path) && !recursive {
                let p = path.clone();
                uow.print(format!("rm: {p}: is a directory"));
            } else {
                kernel.vfs.delete(&path);
            }
        }
    }
}

pub struct RmdirCmd;
impl Command for RmdirCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let dirs: Vec<&str> = args.iter().skip(1).copied().collect();
        if dirs.is_empty() {
            uow.print("usage: rmdir dir...");
            return;
        }
        for d in dirs {
            let path = uow.session.resolve_path(Some(d));
            if !kernel.vfs.is_dir(&path) {
                let p = path.clone();
                uow.print(format!("rmdir: {p}: Not a directory"));
            } else if !kernel.vfs.readdir(&path).is_empty() {
                let p = path.clone();
                uow.print(format!("rmdir: {p}: Directory not empty"));
            } else {
                kernel.vfs.delete(&path);
            }
        }
    }
}

pub struct MkdirCmd;
impl Command for MkdirCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let dirs: Vec<&str> = args
            .iter()
            .skip(1)
            .filter(|a| !a.starts_with('-'))
            .copied()
            .collect();
        if dirs.is_empty() {
            uow.print("usage: mkdir dir...");
            return;
        }
        for d in dirs {
            let path = uow.session.resolve_path(Some(d));
            if kernel.vfs.exists(&path) {
                let p = path.clone();
                uow.print(format!("mkdir: {p}: File exists"));
            } else {
                kernel.vfs.mkdir(&path);
            }
        }
    }
}

pub struct ChmodCmd;
impl Command for ChmodCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        if args.len() < 3 {
            uow.print("usage: chmod mode file...");
            return;
        }
        let mode = args[1];
        for f in args.iter().skip(2) {
            let path = uow.session.resolve_path(Some(f));
            if !kernel.vfs.exists(&path) {
                let p = path.clone();
                uow.print(format!("chmod: {p}: No such file or directory"));
            } else if let Some(stat) = kernel.vfs.stat_mut(&path) {
                // Apply numeric or symbolic mode
                stat.permissions = apply_chmod(&stat.permissions, mode);
            }
        }
    }
}

fn apply_chmod(current: &str, mode: &str) -> String {
    // Very simplified: just accept numeric octal
    if mode.len() == 3 && mode.chars().all(|c| c.is_ascii_digit()) {
        let is_dir = current.starts_with('d');
        let mut chars: Vec<char> = vec!['?'; 10];
        chars[0] = if is_dir { 'd' } else { '-' };
        let digits: Vec<u8> = mode.chars().map(|c| c as u8 - b'0').collect();
        for (i, &d) in digits.iter().enumerate() {
            let base = 1 + i * 3;
            chars[base] = if d & 4 != 0 { 'r' } else { '-' };
            chars[base + 1] = if d & 2 != 0 { 'w' } else { '-' };
            chars[base + 2] = if d & 1 != 0 { 'x' } else { '-' };
        }
        chars.iter().collect()
    } else {
        current.to_string()
    }
}

pub struct FileCmd;
impl Command for FileCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let files: Vec<&str> = args.iter().skip(1).copied().collect();
        if files.is_empty() {
            uow.print("usage: file file...");
            return;
        }
        for f in files {
            let path = uow.session.resolve_path(Some(f));
            let desc = if kernel.vfs.is_dir(&path) {
                "directory".to_string()
            } else if let Some(content) = kernel.vfs.read_file(&path) {
                if content.contains("[COMPRESSED ARCHIVE") {
                    "compress'd data".to_string()
                } else if content.contains("[binary")
                    || content.contains("[core dump")
                    || content.contains("[mail spool")
                {
                    "data".to_string()
                } else if content.starts_with("#!/bin/sh") || content.starts_with("#!/bin") {
                    "Bourne shell script text".to_string()
                } else if content.starts_with("/*") || content.contains("#include") {
                    "C source, ASCII text".to_string()
                } else {
                    "ASCII text".to_string()
                }
            } else {
                "No such file or directory".to_string()
            };
            let fname = f;
            uow.print(format!("{fname}: {desc}"));
        }
    }
}
