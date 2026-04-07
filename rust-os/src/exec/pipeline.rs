use crate::commands::CommandRegistry;
use crate::exec::builtins;
use crate::exec::tokenizer::tokenize;
use crate::kernel::unit_of_work::UnitOfWork;
use crate::kernel::Kernel;
use crate::session::UserSession;
use crate::state::QuestState;

pub struct MinixPipeline {
    registry: CommandRegistry,
}

impl Default for MinixPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl MinixPipeline {
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::build(),
        }
    }

    /// Execute a raw input line. Returns true if the shell should exit.
    pub fn execute(
        &self,
        input: &str,
        session: &mut UserSession,
        quest: &mut QuestState,
        kernel: &mut Kernel,
    ) -> (bool, Vec<crate::kernel::unit_of_work::ScheduledLine>) {
        session.push_history(input);
        let history_snapshot = session.command_history.clone();
        let base_time = kernel.uptime_ms();
        let mut uow = UnitOfWork::new(session, quest, base_time);

        let commands = tokenize(input);
        for cmd in &commands {
            if cmd.tokens.is_empty() {
                continue;
            }

            // Try builtins first
            if let Some(_code) = builtins::try_handle(&cmd.tokens, &mut uow, &history_snapshot) {
                if uow.exit_requested {
                    let lines = uow.drain();
                    return (true, lines);
                }
                continue;
            }

            // Try registered commands
            let name = &cmd.tokens[0];
            if let Some(handler) = self.registry.get(name) {
                let args: Vec<&str> = cmd.tokens.iter().map(|s| s.as_str()).collect();
                handler.execute(&args, &mut uow, kernel);
            } else {
                // Easter eggs / one-liners
                if !try_one_liner(name, &cmd.tokens, &mut uow, kernel) {
                    let n = name.clone();
                    uow.print(format!("{n}: command not found"));
                }
            }
        }

        let lines = uow.drain();
        (false, lines)
    }
}

fn try_one_liner(name: &str, args: &[String], uow: &mut UnitOfWork, kernel: &Kernel) -> bool {
    let difficulty = &kernel.spec.difficulty;
    let quest = &*uow.quest;

    match name {
        "emacs" => {
            uow.print("emacs: not installed. only vi available.");
            true
        }
        "vi" => {
            uow.print("vi: insufficient memory");
            true
        }
        "vim" => {
            uow.print("vim: not installed. try vi. (insufficient memory anyway)");
            true
        }
        "nano" => {
            uow.print("nano: not installed");
            true
        }
        "gcc" => {
            uow.print("gcc: not installed. try Amsterdam Compiler Kit (ack)");
            true
        }
        "cc" => {
            uow.print("cc: no input files");
            true
        }
        "make" => {
            uow.print("make: no targets. nothing to do.");
            true
        }
        "sed" => {
            uow.print("sed: not installed");
            true
        }
        "awk" => {
            uow.print("awk: not installed");
            true
        }
        "python" | "python3" | "perl" | "ruby" | "node" => {
            uow.print(format!("{name}: command not found"));
            true
        }
        "ssh" => {
            uow.print("ssh: command not found (not yet invented)");
            true
        }
        "scp" | "sftp" => {
            uow.print(format!("{name}: command not found"));
            true
        }
        "wget" | "curl" => {
            uow.print(format!("{name}: command not found. use ftp."));
            true
        }
        "sudo" => {
            uow.print("sudo: command not found. this is MINIX.");
            true
        }
        "apt" | "apt-get" | "yum" | "brew" => {
            uow.print(format!(
                "{name}: command not found. software is installed from tarballs here."
            ));
            true
        }
        "git" => {
            uow.print("git: command not found (not yet)");
            true
        }
        "docker" | "kubectl" | "terraform" => {
            uow.print(format!("{name}: command not found"));
            true
        }
        "init" => {
            uow.print("init: must be run as PID 1");
            true
        }
        "reboot" => {
            uow.print("reboot: permission denied");
            true
        }
        "shutdown" => {
            uow.print("shutdown: permission denied");
            true
        }
        "halt" => {
            uow.print("halt: permission denied");
            true
        }
        "linux" => {
            uow.print("linux: command not found (not yet)");
            true
        }
        "minix" => {
            // Stateful — handled by quest counter
            let count = quest.anomaly_count();
            if count >= 2 {
                uow.print("minix: I know.");
            }
            true
        }
        "hello" => true, // silent
        "su" => {
            use crate::difficulty::Difficulty;
            if matches!(difficulty, Difficulty::Su) {
                uow.print("su: you chose this name, didn't you?");
            } else {
                uow.print("su: permission denied");
            }
            true
        }
        "rm" if args
            .get(1)
            .map(|s| s == "-rf" || s == "-fr")
            .unwrap_or(false) =>
        {
            if args.get(2).map(|s| s == "/").unwrap_or(false) {
                uow.print("rm: '/': Operation not permitted");
            } else {
                return false; // let rm command handle it
            }
            true
        }
        _ => false,
    }
}
