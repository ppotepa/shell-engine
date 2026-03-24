use crate::commands::Command;
use crate::kernel::unit_of_work::UnitOfWork;
use crate::kernel::Kernel;

static FORTUNES: &[&str] = &[
    "RFC 1149: \"A Standard for the Transmission of IP Datagrams on Avian Carriers.\"",
    "The best programs are the ones written when the programmer is supposed to be doing something else. -- Cargill",
    "Debugging is twice as hard as writing the code in the first place. -- Kernighan",
    "There are two ways to write error-free programs; only the third one works. -- Hoare",
    "C is quirky, flawed, and an enormous success. -- Ritchie",
    "Unix is simple. It just takes a genius to understand its simplicity. -- Ritchie",
    "A language that doesn't affect the way you think about programming is not worth knowing. -- Perlis",
    "Simplicity does not precede complexity, but follows it. -- Perlis",
    "I think most of you know MINIX. I've done MINIX. -- Tanenbaum",
    "Good judgment comes from experience, and experience comes from bad judgment. -- Covey",
];

static SPOOKY_FORTUNE: &str =
    "The best programs are the ones that haven't been written yet. -- ????";

pub struct FortuneCmd;
impl Command for FortuneCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let anomaly_count = uow.quest.anomaly_count();
        // Deterministic "random" based on uptime
        let tick = kernel.uptime_ms();
        let show_spooky = anomaly_count >= 2 && (tick % 10 == 0);

        if show_spooky {
            uow.print(SPOOKY_FORTUNE.to_string());
        } else {
            let idx = (tick / 1000) as usize % FORTUNES.len();
            uow.print(FORTUNES[idx].to_string());
        }
    }
}

pub struct ManCmd;
impl Command for ManCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        let topic = args.get(1).copied().unwrap_or("");
        match topic {
            "ftp" => {
                uow.print("FTP(1)                   User Commands                  FTP(1)");
                uow.print("".to_string());
                uow.print("NAME");
                uow.print("       ftp - ARPANET file transfer program");
                uow.print("".to_string());
                uow.print("SYNOPSIS");
                uow.print("       ftp [host]");
                uow.print("".to_string());
                uow.print("COMMANDS");
                uow.print("       open host      Connect to remote host");
                uow.print("       ascii          Set transfer type to ASCII (default)");
                uow.print("       binary         Set transfer type to BINARY");
                uow.print("       put file       Upload local file");
                uow.print("       ls             List remote directory");
                uow.print("       bye            Close connection and exit");
                uow.print("".to_string());
                uow.print("NOTES");
                uow.print("       Compressed files (.Z) MUST be transferred in binary mode.");
                uow.print("       ASCII mode will corrupt compressed archive headers.");
            }
            "ping" => {
                uow.print("PING(8)                System Manager's Manual               PING(8)");
                uow.print("".to_string());
                uow.print("NAME");
                uow.print("       ping - send ICMP ECHO_REQUEST to network hosts");
                uow.print("".to_string());
                uow.print("SYNOPSIS");
                uow.print("       ping host");
                uow.print("".to_string());
                uow.print("DESCRIPTION");
                uow.print("       ping uses the ICMP protocol to elicit a response from a host.");
            }
            "" => {
                uow.print("What manual page do you want?");
            }
            _ => {
                uow.print(format!("No manual entry for {topic}"));
            }
        }
    }
}

pub struct HelpCmd;
impl Command for HelpCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        uow.print("Available commands (MINIX 1.1):");
        uow.print("".to_string());
        uow.print("  File:    ls  cat  cp  mv  rm  rmdir  mkdir  chmod  file");
        uow.print("  Text:    echo  grep  head  tail  wc");
        uow.print("  Network: ping  nslookup  netstat  ifconfig  ftp  finger  telnet");
        uow.print("  Process: ps  kill");
        uow.print("  System:  df  mount  date  uname  hostname  whoami  who  w  id  dmesg  last");
        uow.print("  Shell:   cd  pwd  exit  history");
        uow.print("  Other:   fortune  man  help  clear  mail  sync");
        uow.print("".to_string());
        uow.print("Type 'man <command>' for details.");
    }
}

pub struct ClearCmd;
impl Command for ClearCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        // Signal to the AppHost to clear the screen
        uow.print("\x1b[2J\x1b[H".to_string());
    }
}

pub struct MailCmd;
impl Command for MailCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let messages = kernel.mail.list();
        if messages.is_empty() {
            uow.print("No mail.");
            return;
        }
        uow.print(format!("Mail version 2.12 6/28/83. Type ? for help."));
        uow.print(format!("\"/var/spool/mail/{}\": {} messages",
            uow.session.user, messages.len()));
        for (i, m) in messages.iter().enumerate() {
            let status = if m.read { " " } else { "N" };
            uow.print(format!("{}{:3} {}  {:<20} {}", status, i + 1, m.date, m.from, m.subject));
        }
        uow.print("& ".to_string());
        uow.print("(use mail application for interactive reading)".to_string());
    }
}
