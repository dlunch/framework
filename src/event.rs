use alloc::vec::Vec;
use core::future::Future;

use crate::{aggregate::Aggregate, as_any::AsAny, Result};

pub type EventTypeId = u32;

pub trait Event: Sync + Send + AsAny {
    fn type_id(&self) -> EventTypeId;
    fn version(&self) -> u32;
}

pub trait EventStore {
    fn read<A>(
        &self,
        aggregate_id: u64,
        from_version: u32,
    ) -> impl Future<Output = Result<Vec<A::Event>>> + Send
    where
        A: Aggregate;

    fn save<A>(
        &self,
        aggregate_id: u64,
        events: &[A::Event],
    ) -> impl Future<Output = Result<()>> + Send
    where
        A: Aggregate;
}
