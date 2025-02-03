use alloc::boxed::Box;

use crate::{aggregate::Aggregate, Result};

#[async_trait::async_trait]
pub trait SnapshotStore: Sync + Send {
    async fn read<A>(&self, aggregate_id: u64) -> Result<Option<A>>
    where
        A: Aggregate;
    async fn save<A>(&self, aggregate: &A) -> Result<()>
    where
        A: Aggregate;
}

pub struct DummySnapshotStore;

#[async_trait::async_trait]
impl SnapshotStore for DummySnapshotStore {
    async fn read<A>(&self, _aggregate_id: u64) -> Result<Option<A>>
    where
        A: Aggregate,
    {
        Ok(None)
    }

    async fn save<A>(&self, _aggregate: &A) -> Result<()>
    where
        A: Aggregate,
    {
        Ok(())
    }
}
