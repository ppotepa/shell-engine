//! Startup pipeline — runs ordered [`StartupCheck`]s before the game loop begins and collects the [`StartupReport`].

pub mod check;
pub mod checks;
pub mod context;
pub mod report;
pub mod runner;

pub use check::StartupCheck;
pub use context::{StartupContext, StartupSceneFile};
pub use report::{StartupIssue, StartupIssueLevel, StartupReport};
pub use runner::StartupRunner;
