pub mod clock;
pub mod disk;
pub mod events;
pub mod journal;
pub mod mail;
pub mod modem;
pub mod mount;
pub mod network;
pub mod process;
pub mod resources;
pub mod unit_of_work;
pub mod users;

use crate::difficulty::MachineSpec;
use crate::hardware::HardwareProfile;
use crate::state::{FileStat, MachineState, QuestState};
use crate::vfs::Vfs;
use clock::SimulatedClock;
use disk::SimulatedDisk;
use events::KernelEventQueue;
use journal::Journal;
use mail::MailSpool;
use modem::SimulatedModem;
use mount::MountTable;
use network::SimulatedNetwork;
use process::SimulatedProcessTable;
use resources::ResourceState;
use users::UserDatabase;

pub struct Kernel {
    pub spec: MachineSpec,
    pub hw: HardwareProfile,
    pub clock: SimulatedClock,
    pub disk: SimulatedDisk,
    pub network: SimulatedNetwork,
    pub process: SimulatedProcessTable,
    pub users: UserDatabase,
    pub mail: MailSpool,
    pub mounts: MountTable,
    pub modem: SimulatedModem,
    pub journal: Journal,
    pub resources: ResourceState,
    pub events: KernelEventQueue,
    pub vfs: Vfs,
}

impl Kernel {
    pub fn new(spec: MachineSpec, vfs: Vfs) -> Self {
        let hw = HardwareProfile::from_spec(&spec);
        let resources = ResourceState::new(&spec);
        let clock = SimulatedClock::new();
        let disk = SimulatedDisk::new();
        let network = SimulatedNetwork::new();
        let mut process = SimulatedProcessTable::new(&spec);
        process.boot_system_processes();
        let users = UserDatabase::new();
        let mail = MailSpool::new();
        let mounts = MountTable::new(&spec);
        let modem = SimulatedModem::new(&hw);
        let journal = Journal::new();
        let events = KernelEventQueue::new();

        Self {
            spec,
            hw,
            clock,
            disk,
            network,
            process,
            users,
            mail,
            mounts,
            modem,
            journal,
            resources,
            events,
            vfs,
        }
    }

    /// Advance simulation clock and drain events.
    pub fn tick(&mut self, dt_ms: u64) {
        self.clock.advance(dt_ms);
        let now = self.clock.uptime_ms();
        self.resources.disk_ctrl.update_spindle_state(now, &self.hw);
        // Drain ready kernel events
        let ready = self.events.drain_ready(now);
        for ev in ready {
            (ev.action)();
        }
    }

    pub fn uptime_ms(&self) -> u64 {
        self.clock.uptime_ms()
    }
}

/// Per-command execution scope.
pub use unit_of_work::UnitOfWork;
