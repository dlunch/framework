#![no_std]
extern crate alloc;

mod aggregate;
mod as_any;
mod command;
mod error;
mod event;
mod event_listener;
mod framework;
mod query;
mod read_model;
mod repository;
mod snapshot;

pub use self::{
    aggregate::{Aggregate, AggregateTypeId},
    command::Command,
    error::FrameworkError,
    event::{Event, EventStore, EventTypeId},
    framework::Framework,
    query::{Query, QueryHandler},
    read_model::{ReadModel, ReadModelStore},
    snapshot::{DummySnapshotStore, SnapshotStore},
};

pub type Result<T> = core::result::Result<T, FrameworkError>;
