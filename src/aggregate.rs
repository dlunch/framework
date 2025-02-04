use alloc::vec::Vec;

use serde::{de::DeserializeOwned, Serialize};

use crate::{command::Command, event::Event, Result};

pub type AggregateTypeId = u32;

pub trait Aggregate: Sync + Send + Default + Serialize + DeserializeOwned {
    type Command: Command;
    type Event: Event + Serialize + DeserializeOwned;

    fn type_id() -> AggregateTypeId
    where
        Self: Sized;
    fn version(&self) -> u32;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>>;
    fn apply_events(&mut self, events: &[Self::Event]) -> Result<()>;
}
