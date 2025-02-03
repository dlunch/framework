use alloc::{boxed::Box, vec::Vec};

use crate::{aggregate::Aggregate, as_any::AsAny, Result};

pub type EventTypeId = u32;

pub trait Event: AsAny + Sync + Send {
    fn type_id(&self) -> EventTypeId;
    fn version(&self) -> u32;
}

#[async_trait::async_trait]
pub trait EventStore: Sync + Send {
    async fn read<A>(&self, aggregate_id: u64, from_version: u32) -> Result<Vec<A::Event>>
    where
        A: Aggregate;

    async fn save<A>(&self, aggregate_id: u64, events: &[A::Event]) -> Result<()>
    where
        A: Aggregate;
}
