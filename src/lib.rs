//! Who Stole My Focus. The library half exists so the tests can reach the same
//! code the executable runs, rather than a copy of it.

pub mod app;
pub mod config;
pub mod guard;
pub mod history;
pub mod i18n;
pub mod input;
pub mod journal;
pub mod panel;
pub mod policy;
pub mod run;
pub mod system;
pub mod tray;
pub mod viewer;
pub mod window;

pub use run::run_guard;
