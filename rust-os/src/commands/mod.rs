use std::collections::HashMap;
use crate::kernel::unit_of_work::UnitOfWork;
use crate::kernel::Kernel;

pub mod fs;
pub mod text;
pub mod net;
pub mod proc;
pub mod sys;
pub mod misc;

pub trait Command: Send + Sync {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel);
}

pub struct CommandRegistry {
    map: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn build() -> Self {
        let mut map: HashMap<String, Box<dyn Command>> = HashMap::new();

        // FS commands
        map.insert("ls".into(), Box::new(fs::LsCmd));
        map.insert("cat".into(), Box::new(fs::CatCmd));
        map.insert("cp".into(), Box::new(fs::CpCmd));
        map.insert("mv".into(), Box::new(fs::MvCmd));
        map.insert("rm".into(), Box::new(fs::RmCmd));
        map.insert("rmdir".into(), Box::new(fs::RmdirCmd));
        map.insert("mkdir".into(), Box::new(fs::MkdirCmd));
        map.insert("chmod".into(), Box::new(fs::ChmodCmd));
        map.insert("file".into(), Box::new(fs::FileCmd));

        // Text commands
        map.insert("echo".into(), Box::new(text::EchoCmd));
        map.insert("grep".into(), Box::new(text::GrepCmd));
        map.insert("head".into(), Box::new(text::HeadCmd));
        map.insert("tail".into(), Box::new(text::TailCmd));
        map.insert("wc".into(), Box::new(text::WcCmd));

        // Network commands
        map.insert("ping".into(), Box::new(net::PingCmd));
        map.insert("nslookup".into(), Box::new(net::NslookupCmd));
        map.insert("netstat".into(), Box::new(net::NetstatCmd));
        map.insert("ifconfig".into(), Box::new(net::IfconfigCmd));
        map.insert("ftp".into(), Box::new(net::FtpCmd));
        map.insert("finger".into(), Box::new(net::FingerCmd));
        map.insert("telnet".into(), Box::new(net::TelnetCmd));

        // Process commands
        map.insert("ps".into(), Box::new(proc::PsCmd));
        map.insert("kill".into(), Box::new(proc::KillCmd));

        // System commands
        map.insert("df".into(), Box::new(sys::DfCmd));
        map.insert("mount".into(), Box::new(sys::MountCmd));
        map.insert("date".into(), Box::new(sys::DateCmd));
        map.insert("uname".into(), Box::new(sys::UnameCmd));
        map.insert("hostname".into(), Box::new(sys::HostnameCmd));
        map.insert("whoami".into(), Box::new(sys::WhoamiCmd));
        map.insert("who".into(), Box::new(sys::WhoCmd));
        map.insert("w".into(), Box::new(sys::WCmd));
        map.insert("id".into(), Box::new(sys::IdCmd));
        map.insert("sync".into(), Box::new(sys::SyncCmd));
        map.insert("dmesg".into(), Box::new(sys::DmesgCmd));
        map.insert("last".into(), Box::new(sys::LastCmd));

        // Misc
        map.insert("fortune".into(), Box::new(misc::FortuneCmd));
        map.insert("man".into(), Box::new(misc::ManCmd));
        map.insert("help".into(), Box::new(misc::HelpCmd));
        map.insert("clear".into(), Box::new(misc::ClearCmd));
        map.insert("mail".into(), Box::new(misc::MailCmd));

        Self { map }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.map.get(name).map(|b| b.as_ref())
    }
}
