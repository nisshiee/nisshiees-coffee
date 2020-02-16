extern crate failure;
extern crate serde;
extern crate uuid;

#[cfg(test)]
extern crate serde_json;
#[cfg(test)]
extern crate simulacrum;

pub mod projector;
pub mod store;

pub mod aggregate;
pub use aggregate::Aggregate;

pub mod id;
pub use id::Id;

pub mod event;
pub use event::Event;

pub mod command;
pub use command::{Command, CommandError};

#[cfg(test)]
mod tests;
